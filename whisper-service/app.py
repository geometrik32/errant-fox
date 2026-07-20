import json
import os
import re
import subprocess
import sys
import tempfile
import traceback
import gc

import requests
from fastapi import FastAPI
from pydantic import BaseModel

app = FastAPI(title="Whisper Exchange Detection Service (Faster-Whisper CTranslate2)")

WHISPER_MODEL_NAME = os.environ.get("WHISPER_MODEL", "medium")

# ── Pure Stop-word Detection Patterns ─────────────────────────────────────────

_stop_pattern = re.compile(
    r'\b(стоп|стопп|стопе|стопи|топ|топп|терпом|время|тайм|хоп|оп|ап|хальт|альт|halt|stop)\b',
    re.IGNORECASE
)
_start_pattern = re.compile(
    r'\b(бой|бои|бойте|боите|бойтэ|бойтес|бойцы|вход|входи|входь|начали|начал|начало|начнем|fight|begin|ready|go)\b',
    re.IGNORECASE
)


import wave
import numpy as np

def _read_wav_mono(wav_path: str):
    with wave.open(wav_path, 'rb') as w:
        params = w.getparams()
        nchannels, sampwidth, framerate, nframes = params[:4]
        frames = w.readframes(nframes)
        if sampwidth == 2:
            data = np.frombuffer(frames, dtype=np.int16).astype(np.float32) / 32768.0
        elif sampwidth == 4:
            data = np.frombuffer(frames, dtype=np.int32).astype(np.float32) / 2147483648.0
        else:
            raise ValueError(f"Unsupported sample width: {sampwidth}")
        if nchannels > 1:
            data = data.reshape(-1, nchannels).mean(axis=1)
        return framerate, data

def _find_shout_acoustic_peak(audio_data, t_whisper: float, search_window_before=1.5, search_window_after=0.5):
    if audio_data is None:
        return t_whisper

    rate, data = audio_data

    win_start_sec = max(0.0, t_whisper - search_window_before)
    win_end_sec = min(len(data) / rate, t_whisper + search_window_after)

    idx_start = int(win_start_sec * rate)
    idx_end = int(win_end_sec * rate)

    if idx_end <= idx_start:
        return t_whisper

    window_data = data[idx_start:idx_end]
    frame_len = int(rate * 0.02) # 20ms
    if len(window_data) < frame_len:
        return t_whisper

    num_frames = len(window_data) // frame_len
    rms = np.array([
        np.sqrt(np.mean(window_data[i * frame_len : (i + 1) * frame_len] ** 2))
        for i in range(num_frames)
    ])

    if len(rms) == 0:
        return t_whisper

    max_frame = np.argmax(rms)
    max_rms = rms[max_frame]
    background_noise = np.percentile(rms, 20)

    if max_rms < 0.04 or max_rms < background_noise * 1.8:
        return t_whisper

    onset_frame = max_frame
    thresh = background_noise + 0.35 * (max_rms - background_noise)
    for f in range(max_frame, -1, -1):
        if rms[f] < thresh:
            onset_frame = f
            break

    refined_time = win_start_sec + (onset_frame * 0.02)
    return float(refined_time)


def _extract_simple_exchanges_from_faster(segments_list, wav_path: str):
    """
    Pure & Simple Detection for faster-whisper output with Acoustic Peak Refinement:
    Every 'Стоп' word detected at T_stop is refined to the exact onset of the acoustic shout.
    Bout window: [max(0, T_exact - 2.0s), T_exact + 1.0s].
    Close duplicate stop words (within 3.0s) are grouped to keep only the first trigger.
    """
    all_words = []
    stops = []

    for seg in segments_list:
        if hasattr(seg, "words") and seg.words:
            for w in seg.words:
                word_raw = w.word.strip()
                word_clean = re.sub(r'[^\w\s]', '', word_raw.lower())
                start_t = w.start
                end_t = w.end
                prob = getattr(w, "probability", 1.0)

                disp_word = word_raw
                if re.search(r'\b(боите|бои|бойте|бойтэ|бойтес)\b', word_clean, re.IGNORECASE):
                    disp_word = "Бой"
                elif re.search(r'\b(стопп|стопе|стопи|топ|топп|терпом)\b', word_clean, re.IGNORECASE):
                    disp_word = "Стоп"

                item = {
                    "word": disp_word,
                    "clean": word_clean,
                    "start": start_t,
                    "end": end_t,
                    "prob": prob
                }
                all_words.append(item)

                if _stop_pattern.search(word_clean) and prob >= 0.15:
                    stops.append(item)

    # Filter close duplicate stop words (within 3.0s)
    grouped_stops = []
    if stops:
        curr = stops[0]
        for nxt in stops[1:]:
            if nxt["start"] - curr["start"] > 3.0:
                grouped_stops.append(curr)
                curr = nxt
        grouped_stops.append(curr)

    # Pre-load audio data ONCE for acoustic refinement of all stop words
    audio_data = None
    if grouped_stops:
        try:
            audio_data = _read_wav_mono(wav_path)
        except Exception as e:
            print(f"  Warning: Audio read error for acoustic refinement ({e}). Using Whisper timestamp.", flush=True)

    exchanges = []
    for stop in grouped_stops:
        stop_time = stop["start"]
        refined_stop_time = _find_shout_acoustic_peak(audio_data, stop_time)
        shift = refined_stop_time - stop_time
        if abs(shift) > 0.05:
            print(f"  [Acoustic Refinement] Stop word at {stop_time:.2f}s -> refined to shout onset at {refined_stop_time:.2f}s (shift {shift:+.2f}s)", flush=True)

        bout_start = max(0.0, refined_stop_time - 2.0)
        bout_end = refined_stop_time + 1.0

        exchanges.append({
            "start_ms": int(bout_start * 1000),
            "end_ms": int(bout_end * 1000),
            "stop_word": stop["word"]
        })

    # Clean up audio memory buffer
    if audio_data is not None:
        del audio_data

    return exchanges, all_words


def _detect_exchanges(wav_path: str):
    from faster_whisper import WhisperModel

    print(f"  Loading Faster-Whisper (CTranslate2) model '{WHISPER_MODEL_NAME}' on CPU...", flush=True)
    # compute_type="int8" optimized specifically for Intel CPU AVX2/AVX512 (3-4x speedup, 4x less RAM)
    model = WhisperModel(WHISPER_MODEL_NAME, device="cpu", compute_type="int8")

    try:
        segments, info = model.transcribe(
            wav_path,
            language="ru",
            word_timestamps=True,
            initial_prompt="Бой! Стоп! Удар! Разметка сходов фехтовального поединка."
        )
        segments_list = list(segments)
        exchanges, all_words = _extract_simple_exchanges_from_faster(segments_list, wav_path)

        for idx, ex in enumerate(exchanges):
            print(f"  Exchange {idx+1}: {ex['start_ms']}ms – {ex['end_ms']}ms ('{ex.get('stop_word', '')}')", flush=True)

        return exchanges, all_words
    finally:
        print("  Unloading Faster-Whisper model from RAM...", flush=True)
        del model
        gc.collect()


# ── Request / Response schemas ────────────────────────────────────────────────

class AnalyzeRequest(BaseModel):
    audio_url: str
    video_id: str


# ── Endpoints ─────────────────────────────────────────────────────────────────

@app.get("/health")
def health():
    return {"status": "ok"}


import asyncio

analyze_lock = asyncio.Lock()
cancelled_video_ids: set[str] = set()


@app.post("/cancel/{video_id}")
async def cancel_video(video_id: str):
    """Mark a video_id as cancelled so queued /analyze requests skip it immediately."""
    cancelled_video_ids.add(video_id)
    print(f"[cancel] video_id={video_id} marked as cancelled. Queue will skip it.", flush=True)
    return {"status": "cancelled", "video_id": video_id}


def _download_and_convert_audio(audio_url: str, tmpdir: str) -> str:
    wav_path = os.path.join(tmpdir, "audio.wav")

    print(f"  Streaming audio directly via ffmpeg from URL...", flush=True)
    result = subprocess.run(
        [
            "ffmpeg", "-y",
            "-vn",
            "-i", audio_url,
            "-ar", "16000",
            "-ac", "1",
            "-f", "wav",
            wav_path,
        ],
        capture_output=True,
        text=True,
    )
    if result.returncode == 0:
        return wav_path

    # Fallback: if direct HTTP ffmpeg input fails, download raw file & convert
    print("  Direct ffmpeg stream failed, falling back to stream download...", flush=True)
    raw_path = os.path.join(tmpdir, "raw_audio")
    try:
        with requests.get(audio_url, stream=True, timeout=120) as r:
            r.raise_for_status()
            with open(raw_path, "wb") as f:
                for chunk in r.iter_content(chunk_size=65536):
                    f.write(chunk)

        result_fb = subprocess.run(
            [
                "ffmpeg", "-y",
                "-vn",
                "-i", raw_path,
                "-ar", "16000",
                "-ac", "1",
                "-f", "wav",
                wav_path,
            ],
            capture_output=True,
            text=True,
        )
        if result_fb.returncode != 0:
            msg = result_fb.stderr[-2000:] if result_fb.stderr else "ffmpeg failed"
            raise RuntimeError(f"ffmpeg error: {msg}")
    finally:
        if os.path.exists(raw_path):
            try:
                os.remove(raw_path)
            except Exception:
                pass

    return wav_path


@app.post("/analyze")
async def analyze(body: AnalyzeRequest):
    vid = body.video_id
    print(f"[analyze] video_id={vid} request queued.", flush=True)

    # ── Pre-lock check: skip immediately if already cancelled while waiting in queue ──
    if vid in cancelled_video_ids:
        cancelled_video_ids.discard(vid)
        print(f"[analyze] video_id={vid} was cancelled before lock acquisition. Skipping.", flush=True)
        return {"video_id": vid, "skipped": True, "reason": "cancelled"}

    loop = asyncio.get_running_loop()

    async with analyze_lock:
        # ── Post-lock check: could have been cancelled while we waited for the lock ──
        if vid in cancelled_video_ids:
            cancelled_video_ids.discard(vid)
            print(f"[analyze] video_id={vid} was cancelled while waiting in queue. Skipping.", flush=True)
            return {"video_id": vid, "skipped": True, "reason": "cancelled"}

        print(f"[analyze] video_id={vid} processing started (download + transcribe).", flush=True)
        tmpdir = tempfile.mkdtemp()
        try:
            wav_path = await loop.run_in_executor(None, _download_and_convert_audio, body.audio_url, tmpdir)

            # ── Mid-processing check: cancelled during download? ──
            if vid in cancelled_video_ids:
                cancelled_video_ids.discard(vid)
                print(f"[analyze] video_id={vid} was cancelled during download. Skipping transcription.", flush=True)
                return {"video_id": vid, "skipped": True, "reason": "cancelled"}

            exchanges, all_words = await loop.run_in_executor(None, _detect_exchanges, wav_path)
            print(f"[analyze] video_id={vid} processing finished.", flush=True)

            # Clean up cancellation marker if still present (e.g. re-queued)
            cancelled_video_ids.discard(vid)

            return {"video_id": vid, "exchanges": exchanges, "words": all_words}
        except Exception as exc:
            traceback.print_exc(file=sys.stderr)
            return {"error": str(exc)}
        finally:
            try:
                import shutil
                shutil.rmtree(tmpdir, ignore_errors=True)
            except Exception:
                pass
