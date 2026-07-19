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


def _extract_simple_exchanges_from_faster(segments_list):
    """
    Pure & Simple Detection for faster-whisper output:
    Every 'Стоп' word detected at T_stop creates a bout: [max(0, T_stop - 2.0s), T_stop + 1.0s].
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

    exchanges = []
    for stop in grouped_stops:
        stop_time = stop["start"]
        bout_start = max(0.0, stop_time - 2.0)
        bout_end = stop_time + 1.0

        exchanges.append({
            "start_ms": int(bout_start * 1000),
            "end_ms": int(bout_end * 1000),
            "stop_word": stop["word"]
        })

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
        exchanges, all_words = _extract_simple_exchanges_from_faster(segments_list)

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


def _download_and_convert_audio(audio_url: str, tmpdir: str) -> str:
    raw_path = os.path.join(tmpdir, "raw_audio")
    wav_path = os.path.join(tmpdir, "audio.wav")

    print(f"  Downloading audio immediately from URL...", flush=True)
    with requests.get(audio_url, stream=True, timeout=120) as r:
        r.raise_for_status()
        with open(raw_path, "wb") as f:
            for chunk in r.iter_content(chunk_size=65536):
                f.write(chunk)

    print("  Converting to 16kHz mono WAV...", flush=True)
    result = subprocess.run(
        [
            "ffmpeg", "-y",
            "-i", raw_path,
            "-ar", "16000",
            "-ac", "1",
            "-f", "wav",
            wav_path,
        ],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        msg = result.stderr[-2000:] if result.stderr else "ffmpeg failed"
        raise RuntimeError(f"ffmpeg error: {msg}")

    return wav_path


@app.post("/analyze")
async def analyze(body: AnalyzeRequest):
    print(f"[analyze] video_id={body.video_id} request received.", flush=True)
    loop = asyncio.get_running_loop()

    # 1. Download and convert audio IMMEDIATELY so Seafile temporary URL never expires
    tmpdir = tempfile.mkdtemp()
    try:
        wav_path = await loop.run_in_executor(None, _download_and_convert_audio, body.audio_url, tmpdir)
    except Exception as exc:
        print(f"[analyze] video_id={body.video_id} download failed: {exc}", flush=True)
        try:
            import shutil
            shutil.rmtree(tmpdir, ignore_errors=True)
        except Exception:
            pass
        return {"error": f"Audio download failed: {exc}"}

    # 2. Acquire lock and run Faster-Whisper detection sequentially
    try:
        print(f"[analyze] video_id={body.video_id} waiting for CPU lock...", flush=True)
        async with analyze_lock:
            print(f"[analyze] video_id={body.video_id} Faster-Whisper processing started.", flush=True)
            exchanges, all_words = await loop.run_in_executor(None, _detect_exchanges, wav_path)
            print(f"[analyze] video_id={body.video_id} Faster-Whisper processing finished.", flush=True)
            return {"video_id": body.video_id, "exchanges": exchanges, "words": all_words}
    except Exception as exc:
        traceback.print_exc(file=sys.stderr)
        return {"error": str(exc)}
    finally:
        try:
            import shutil
            shutil.rmtree(tmpdir, ignore_errors=True)
        except Exception:
            pass
