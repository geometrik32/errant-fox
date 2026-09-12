#!/usr/bin/env python3
"""
compress_adaptive_vfr.py - Адаптивное сжатие спортивного видео (VFR + Dynamic Quality).

Скрипт разбивает видео на сегменты 'Action' (сходы) и 'Pause' (паузы/обсуждения):
- Action: исходное разрешение, 100 fps, высокое качество (CRF 18-20 / CQ 20).
- Pause: холст исходного разрешения, прореживание до 25 fps, даунскейл 720p -> растяжение, высокое сжатие (CRF 32-35 / CQ 34).
- Аудио: исходное качественное AAC 192 kbps по всему видео.
- Склейка: Concat Demuxer VFR + генерация оглавления глав (Chapters).
"""

import argparse
import json
import os
import shutil
import sqlite3
import subprocess
import sys
import tempfile
import time
from pathlib import Path

# Обеспечение корректного вывода UTF-8 в Windows консоли
if sys.platform == "win32":
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    if hasattr(sys.stderr, "reconfigure"):
        sys.stderr.reconfigure(encoding="utf-8", errors="replace")



def run_command(cmd, verbose=False):
    """Выполнить команду в консоли и проверить статус."""
    if verbose:
        print(f"[CMD] {' '.join(cmd)}", file=sys.stderr)
    result = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    if result.returncode != 0:
        print(f"[ERROR] Команда завершилась с ошибкой:\n{' '.join(cmd)}", file=sys.stderr)
        print(f"[STDERR]\n{result.stderr}", file=sys.stderr)
        raise RuntimeError(f"FFmpeg error: {result.stderr.strip()[:300]}")
    return result


def probe_video(video_path):
    """Получить технические характеристики видео через ffprobe."""
    cmd = [
        "ffprobe", "-v", "error",
        "-show_entries", "format=duration,size,bit_rate:stream=index,codec_type,codec_name,width,height,r_frame_rate,avg_frame_rate,sample_rate,channels",
        "-of", "json",
        str(video_path)
    ]
    res = run_command(cmd)
    data = json.loads(res.stdout)
    
    video_stream = None
    audio_stream = None
    for s in data.get("streams", []):
        if s.get("codec_type") == "video" and not video_stream:
            video_stream = s
        elif s.get("codec_type") == "audio" and not audio_stream:
            audio_stream = s

    if not video_stream:
        raise ValueError(f"Не удалось найти видеодорожку в файле {video_path}")

    # Определение FPS
    r_fps = video_stream.get("r_frame_rate", "30/1")
    if "/" in r_fps:
        num, den = map(float, r_fps.split("/"))
        fps = num / den if den != 0 else 30.0
    else:
        fps = float(r_fps)

    duration = float(data.get("format", {}).get("duration", 0.0))
    width = int(video_stream.get("width", 1920))
    height = int(video_stream.get("height", 1080))
    size_bytes = int(data.get("format", {}).get("size", 0))

    return {
        "duration": duration,
        "width": width,
        "height": height,
        "fps": fps,
        "size_bytes": size_bytes,
        "has_audio": audio_stream is not None,
        "audio_sample_rate": audio_stream.get("sample_rate", "48000") if audio_stream else "48000",
        "audio_channels": audio_stream.get("channels", 2) if audio_stream else 2
    }


def get_bouts_from_db(video_id, db_path):
    """Извлечь интервалы сходов для video_id из SQLite."""
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()
    cursor.execute(
        "SELECT order_index, time_start_ms, time_end_ms FROM bouts WHERE video_id = ? ORDER BY order_index",
        (video_id,)
    )
    rows = cursor.fetchall()
    conn.close()
    
    bouts = []
    for r in rows:
        bouts.append({
            "order": r[0],
            "start": r[1] / 1000.0,
            "end": r[2] / 1000.0
        })
    return bouts


def build_segments(bouts, total_duration):
    """
    Разбивает таймлайн на непрерывную последовательность сегментов (Action и Pause).
    """
    # Сортировка и фильтрация интервалов
    sorted_bouts = sorted(bouts, key=lambda x: x["start"])
    merged_bouts = []
    for b in sorted_bouts:
        s = max(0.0, float(b["start"]))
        e = min(total_duration, float(b["end"]))
        if e <= s:
            continue
        if merged_bouts and s <= merged_bouts[-1]["end"]:
            # Слияние перекрывающихся
            merged_bouts[-1]["end"] = max(merged_bouts[-1]["end"], e)
        else:
            merged_bouts.append({"start": s, "end": e})

    segments = []
    current_time = 0.0

    for i, b in enumerate(merged_bouts):
        # Пауза перед сходом
        if b["start"] > current_time + 0.05:
            segments.append({
                "type": "pause",
                "index": len(segments),
                "start": current_time,
                "end": b["start"],
                "duration": b["start"] - current_time
            })
        
        # Сам сход (Action)
        segments.append({
            "type": "action",
            "index": len(segments),
            "bout_index": i + 1,
            "start": b["start"],
            "end": b["end"],
            "duration": b["end"] - b["start"]
        })
        current_time = b["end"]

    # Хвостовая пауза после последнего схода
    if current_time + 0.05 < total_duration:
        segments.append({
            "type": "pause",
            "index": len(segments),
            "start": current_time,
            "end": total_duration,
            "duration": total_duration - current_time
        })

    return segments


def encode_segment(segment, input_video, output_path, info, encoder="hevc_nvenc",
                   action_crf=19, pause_crf=33, pause_fps=25, verbose=False):
    """Кодирование отдельного сегмента (Action или Pause)."""
    seg_type = segment["type"]
    start_sec = segment["start"]
    duration_sec = segment["duration"]
    orig_w = info["width"]
    orig_h = info["height"]
    orig_fps = max(1, int(round(info["fps"])))
    actual_pause_fps = min(pause_fps, orig_fps)

    cmd = [
        "ffmpeg", "-y",
        "-ss", f"{start_sec:.3f}",
        "-t", f"{duration_sec:.3f}",
        "-i", str(input_video),
    ]

    # Видеофильтры
    if seg_type == "action":
        # Сохраняем исходный фреймрейт и геометрию
        vfilter = f"fps={orig_fps},setsar=1"
        gop = max(1, orig_fps * 2)
        if encoder == "hevc_nvenc":
            cmd += [
                "-c:v", "hevc_nvenc",
                "-preset", "p5",
                "-cq", str(action_crf),
                "-b:v", "0",
                "-g", str(gop),
            ]
        else:
            cmd += [
                "-c:v", "libx265",
                "-preset", "fast",
                "-crf", str(action_crf),
                "-g", str(gop),
            ]
    else:
        # Pause: прореживаем кадры и сжимаем картинку до 720p -> растягиваем обратно на холст
        # Если исходник больше 720p (1080p, 2K, 4K), сжимаем до 720p и растягиваем на orig_w:orig_h
        # Если исходник сам 720p или меньше, просто жмем с высоким CRF
        if orig_w > 1280 or orig_h > 720:
            vfilter = f"fps={actual_pause_fps},scale=1280:720:flags=bicubic,scale={orig_w}:{orig_h}:flags=bilinear,setsar=1"
        else:
            vfilter = f"fps={actual_pause_fps},setsar=1"

        gop = max(1, actual_pause_fps * 2)
        if encoder == "hevc_nvenc":
            cmd += [
                "-c:v", "hevc_nvenc",
                "-preset", "p5",
                "-cq", str(pause_crf),
                "-b:v", "0",
                "-g", str(gop),
            ]
        else:
            cmd += [
                "-c:v", "libx265",
                "-preset", "fast",
                "-crf", str(pause_crf),
                "-g", str(gop),
            ]

    cmd += [
        "-vf", vfilter,
        "-pix_fmt", "yuv420p",
        "-tag:v", "hvc1",
        "-avoid_negative_ts", "make_zero",
    ]

    if info.get("has_audio", True):
        cmd += [
            "-c:a", "aac",
            "-b:a", "192k",
            "-ar", "48000",
            "-ac", "2",
        ]
    else:
        cmd += ["-an"]

    cmd.append(str(output_path))
    run_command(cmd, verbose=verbose)



def generate_metadata_file(segments, metadata_path):
    """Генерация файла глав FFMETADATAFILE."""
    with open(metadata_path, "w", encoding="utf-8") as f:
        f.write(";FFMETADATA1\n")
        f.write("title=Errant Fox Fencing Sparring (Adaptive VFR)\n\n")

        for seg in segments:
            if seg["type"] == "action":
                start_ms = int(round(seg["start"] * 1000))
                end_ms = int(round(seg["end"] * 1000))
                bout_idx = seg.get("bout_index", 1)
                f.write("[CHAPTER]\n")
                f.write("TIMEBASE=1/1000\n")
                f.write(f"START={start_ms}\n")
                f.write(f"END={end_ms}\n")
                f.write(f"title=Сход #{bout_idx} ({seg['start']:.1f}s - {seg['end']:.1f}s)\n\n")


def compress_video(input_video, bouts, output_video, encoder="hevc_nvenc",
                   action_crf=19, pause_crf=33, pause_fps=25, keep_temp=False, verbose=False):
    """Главная функция адаптивного сжатия."""
    input_path = Path(input_video).resolve()
    output_path = Path(output_video).resolve()
    output_path.parent.mkdir(parents=True, exist_ok=True)

    t0 = time.time()
    info = probe_video(input_path)
    
    print(f"🎬 Исходное видео: {input_path.name}")
    print(f"   Длительность: {info['duration']:.2f} сек | Разрешение: {info['width']}x{info['height']} | FPS: {info['fps']:.1f}")
    print(f"   Размер: {info['size_bytes'] / (1024*1024):.2f} МБ")
    print(f"   Энкодер: {encoder} (Action CRF/CQ={action_crf}, Pause CRF/CQ={pause_crf}, Pause FPS={pause_fps})")

    segments = build_segments(bouts, info["duration"])
    action_count = sum(1 for s in segments if s["type"] == "action")
    pause_count = sum(1 for s in segments if s["type"] == "pause")
    total_action_sec = sum(s["duration"] for s in segments if s["type"] == "action")
    print(f"📊 Сегментация: {len(segments)} участков (Сходов: {action_count} [{total_action_sec:.1f}с], Пауз: {pause_count} [{info['duration'] - total_action_sec:.1f}с])")

    temp_dir = Path(tempfile.mkdtemp(prefix="errant_fox_vfr_"))
    try:
        segment_files = []
        for seg in segments:
            seg_name = f"seg_{seg['index']:03d}_{seg['type']}.mp4"
            seg_path = temp_dir / seg_name
            print(f"   ⚙️ Кодирование [{seg['index']+1}/{len(segments)}] {seg['type'].upper()}: {seg['start']:.2f}с -> {seg['end']:.2f}с ({seg['duration']:.2f}с)...")
            encode_segment(
                segment=seg,
                input_video=input_path,
                output_path=seg_path,
                info=info,
                encoder=encoder,
                action_crf=action_crf,
                pause_crf=pause_crf,
                pause_fps=pause_fps,
                verbose=verbose
            )
            segment_files.append(seg_path)

        # Подготовка списка конкатенации
        concat_list_path = temp_dir / "concat_list.txt"
        with open(concat_list_path, "w", encoding="utf-8") as f:
            for s_file in segment_files:
                # Concat demuxer требует прямые слеши
                escaped_path = str(s_file.resolve()).replace("\\", "/")
                f.write(f"file '{escaped_path}'\n")

        # Генерация файла глав (Chapters)
        metadata_path = temp_dir / "ffmetadata.txt"
        generate_metadata_file(segments, metadata_path)

        # Склейка через FFmpeg Concat Demuxer
        print("🔗 Склейка сегментов в итоговый VFR MP4 с главами...")
        final_cmd = [
            "ffmpeg", "-y",
            "-f", "concat",
            "-safe", "0",
            "-i", str(concat_list_path),
            "-i", str(metadata_path),
            "-map_metadata", "1",
            "-c", "copy",
            "-fps_mode", "vfr",
            "-movflags", "+faststart",
            str(output_path)
        ]
        run_command(final_cmd, verbose=verbose)

    finally:
        if not keep_temp:
            shutil.rmtree(temp_dir, ignore_errors=True)
        else:
            print(f"[INFO] Временные файлы сохранены в: {temp_dir}")

    elapsed = time.time() - t0
    final_info = probe_video(output_path)
    orig_mb = info["size_bytes"] / (1024 * 1024)
    final_mb = final_info["size_bytes"] / (1024 * 1024)
    ratio = (1.0 - (final_mb / orig_mb)) * 100.0

    print("\n✅ СЖАТИЕ УСПЕШНО ЗАВЕРШЕНО!")
    print(f"📁 Итоговый файл: {output_path}")
    print(f"📉 Исходный размер: {orig_mb:.2f} МБ")
    print(f"📦 Финальный размер: {final_mb:.2f} МБ (экономия: {ratio:.1f}%)")
    print(f"⏱️ Время обработки: {elapsed:.1f} сек ({elapsed/info['duration']:.2f}x от реального времени)")

    return {
        "input_size_mb": orig_mb,
        "output_size_mb": final_mb,
        "saved_percent": ratio,
        "elapsed_sec": elapsed,
        "output_path": str(output_path)
    }


def main():
    parser = argparse.ArgumentParser(description="Адаптивное VFR сжатие спортивного видео (Action 100fps vs Pause 25fps)")
    parser.add_argument("input_video", help="Путь к исходному видеофайлу")
    parser.add_argument("-o", "--output", help="Путь для сохранения сжатого видео", default=None)
    parser.add_argument("--video-id", help="UUID видео из SQLite базы errant_fox.db для автоматической загрузки сходов")
    parser.add_argument("--db-path", help="Путь к SQLite базе данных", default="data/db/errant_fox.db")
    parser.add_argument("--bouts-json", help="JSON строка или путь к JSON файлу со списком интервалов [{'start': 10.0, 'end': 15.0}]")
    parser.add_argument("--intervals", help="Строка интервалов через запятую: '13.18-16.18,33.54-36.54'")
    parser.add_argument("--encoder", choices=["hevc_nvenc", "libx265"], default="hevc_nvenc", help="Энкодер H.265 (по умолчанию: hevc_nvenc)")
    parser.add_argument("--action-crf", type=int, default=19, help="CRF/CQ для сегментов Action (по умолчанию: 19)")
    parser.add_argument("--pause-crf", type=int, default=33, help="CRF/CQ для сегментов Pause (по умолчанию: 33)")
    parser.add_argument("--pause-fps", type=int, default=25, help="Частота кадров для пауз (по умолчанию: 25)")
    parser.add_argument("--keep-temp", action="store_true", help="Не удалять временные файлы сегментов")
    parser.add_argument("-v", "--verbose", action="store_true", help="Подробный вывод команд FFmpeg")

    args = parser.parse_args()

    input_path = Path(args.input_video)
    if not input_path.exists():
        print(f"[ERROR] Файл {args.input_video} не найден.", file=sys.stderr)
        sys.exit(1)

    if not args.output:
        args.output = str(input_path.with_name(f"{input_path.stem}_adaptive_vfr.mp4"))

    # Извлечение разметки сходов
    bouts = []
    if args.video_id:
        bouts = get_bouts_from_db(args.video_id, args.db_path)
    elif args.bouts_json:
        if os.path.exists(args.bouts_json):
            with open(args.bouts_json, "r", encoding="utf-8") as f:
                bouts = json.load(f)
        else:
            bouts = json.loads(args.bouts_json)
    elif args.intervals:
        pairs = args.intervals.split(",")
        for p in pairs:
            if "-" in p:
                s, e = map(float, p.strip().split("-"))
                bouts.append({"start": s, "end": e})
    else:
        # Попытаться найти видео по имени в базе
        conn = sqlite3.connect(args.db_path)
        cursor = conn.cursor()
        cursor.execute("SELECT id FROM videos WHERE seafile_path LIKE ? OR seafile_path LIKE ?", 
                       (f"%{input_path.name}", f"%{input_path.stem}%"))
        row = cursor.fetchone()
        conn.close()
        if row:
            print(f"[INFO] Автоматически найден video_id в базе: {row[0]}")
            bouts = get_bouts_from_db(row[0], args.db_path)
        else:
            print("[ERROR] Не указаны сходы! Передайте --video-id, --bouts-json или --intervals.", file=sys.stderr)
            sys.exit(1)

    if not bouts:
        print("[WARNING] Не найдено сходов в разметке! Все видео будет закодировано как пауза.")

    compress_video(
        input_video=args.input_video,
        bouts=bouts,
        output_video=args.output,
        encoder=args.encoder,
        action_crf=args.action_crf,
        pause_crf=args.pause_crf,
        pause_fps=args.pause_fps,
        keep_temp=args.keep_temp,
        verbose=args.verbose
    )


if __name__ == "__main__":
    main()
