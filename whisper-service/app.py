import json
import os
import subprocess
import sys
import tempfile
import traceback

import requests
import whisper
from fastapi import FastAPI
from pydantic import BaseModel

app = FastAPI(title="Whisper Exchange Detection Service")

import gc

WHISPER_MODEL_NAME = os.environ.get("WHISPER_MODEL", "small")

# ── Stop-word detection helpers ───────────────────────────────────────────────

import re
import wave
import numpy as np

_stop_pattern = re.compile(r'\b(стоп|хоп|оп|ап|хальт|альт|halt|stop)\b', re.IGNORECASE)
_start_pattern = re.compile(r'\b(бой|бои|бойцы|вход|входи|входь|начали|начал|начало|начнем|fight|begin|ready|go)\b', re.IGNORECASE)
_filler_pattern = re.compile(r'^(ха|хаха|ха-ха|хх|э|ээ|хм|мм|гм|ух|ах|ох|фу|ы)$', re.IGNORECASE)


def _read_wav_mono(wav_path: str):
    """
    Reads 16kHz mono PCM 16-bit WAV file using Python's built-in wave module and numpy.
    """
    with wave.open(wav_path, 'rb') as w:
        params = w.getparams()
        nchannels, sampwidth, framerate, nframes = params[:4]
        if nchannels != 1 or sampwidth != 2:
            raise ValueError("WAV file must be mono 16-bit PCM")
        frames = w.readframes(nframes)
        data = np.frombuffer(frames, dtype=np.int16).astype(np.float32) / 32768.0
        return framerate, data


def _filter_overlapping_bouts(bouts, min_gap_sec=3.0):
    if not bouts:
        return []

    sorted_bouts = sorted(bouts, key=lambda x: x["start_ms"])
    filtered = [sorted_bouts[0]]

    for curr in sorted_bouts[1:]:
        prev = filtered[-1]
        min_gap_ms = int(min_gap_sec * 1000)

        # Check overlap or gap < min_gap_sec
        if curr["start_ms"] < prev["end_ms"] + min_gap_ms:
            # Keep the bout with explicit stop or higher confidence
            curr_stop = curr.get("stop_word", "")
            prev_stop = prev.get("stop_word", "")
            
            # Prefer explicit stop word over fallback spoken word
            curr_is_explicit = bool(_stop_pattern.search(curr_stop))
            prev_is_explicit = bool(_stop_pattern.search(prev_stop))

            if curr_is_explicit and not prev_is_explicit:
                filtered[-1] = curr
            # Else keep prev, discard duplicate curr
        else:
            filtered.append(curr)

    return filtered


def _extract_decision_tree_exchanges(segments, peaks):
    all_words = []
    starts = []
    explicit_stops = []

    for seg in segments:
        if "words" in seg and seg["words"]:
            for w in seg["words"]:
                word_str = w.get("word", "").strip()
                word_clean = re.sub(r'[^\w\s]', '', word_str.lower())
                start_t = w.get("start")
                end_t = w.get("end")
                prob = w.get("probability", 1.0)

                item = {
                    "word": word_str,
                    "clean": word_clean,
                    "start": start_t,
                    "end": end_t,
                    "prob": prob
                }
                all_words.append(item)

                if _start_pattern.search(word_clean) and prob >= 0.10:
                    starts.append(item)
                elif _stop_pattern.search(word_clean) and prob >= 0.15:
                    explicit_stops.append(item)

    # Filter close duplicate 'Бой' markers (within 3.0s)
    unique_starts = []
    if starts:
        curr = starts[0]
        for nxt in starts[1:]:
            if nxt["start"] - curr["start"] >= 3.0:
                unique_starts.append(curr)
                curr = nxt
        unique_starts.append(curr)

    exchanges = []
    covered_stop_times = set()

    # Process Start-Anchored Scenarios (1, 3, 4, 5)
    for i, st in enumerate(unique_starts):
        start_time = st["start"]
        next_start_time = unique_starts[i+1]["start"] if i + 1 < len(unique_starts) else 99999.0

        if next_start_time - start_time < 3.0:
            continue

        candidates = [
            w for w in all_words
            if w["start"] >= start_time + 0.4 and w["start"] < next_start_time
        ]

        selected_stop = None
        final_stop_t = None

        for cand in candidates:
            is_explicit = bool(_stop_pattern.search(cand["clean"]) and cand["prob"] >= 0.15)
            is_fallback = False
            if not is_explicit and cand["start"] >= start_time + 1.5 and not _filler_pattern.search(cand["clean"]):
                is_fallback = True

            if is_explicit or is_fallback:
                cand_t = cand["start"]
                # Filter active future peaks (if combat clashing continues within 1.0s to 3.5s after stop word, it is not a stop)
                future_peaks = [p for p in peaks if p[0] > cand_t + 1.0 and p[0] <= cand_t + 3.5 and p[1] > 0.08]
                if (not is_explicit or cand["prob"] < 0.50) and future_peaks:
                    continue

                # Align fallback/low-confidence stops to preceding peaks in the last 15 seconds
                stop_t = cand_t
                preceding_peaks = [p for p in peaks if p[0] >= start_time and p[0] <= stop_t and p[0] >= stop_t - 15.0]
                if (cand["prob"] < 0.50 or not _stop_pattern.search(cand["clean"])) and preceding_peaks:
                    last_peak = max(preceding_peaks, key=lambda x: x[0])
                    stop_t = last_peak[0] + 1.0

                # Verify if this window contains clash peaks
                window_peaks = [p for p in peaks if p[0] >= start_time + 1.2 and p[0] <= stop_t]
                if not window_peaks:
                    continue

                selected_stop = cand
                final_stop_t = stop_t
                break

        if selected_stop and final_stop_t:
            bout_start = max(0.0, final_stop_t - 2.0)
            bout_end = final_stop_t + 1.0
            covered_stop_times.add(round(selected_stop["start"], 2))

            exchanges.append({
                "start_ms": int(bout_start * 1000),
                "end_ms": int(bout_end * 1000),
                "stop_word": selected_stop["word"]
            })

    # Process Orphaned Stops (Missing 'Бой')
    for s_st in explicit_stops:
        s_time = s_st["start"]
        if round(s_time, 2) in covered_stop_times:
            continue

        prec_word = next((w for w in reversed(all_words) if w["end"] <= s_time - 0.1), None)
        prec_gap = s_time - (prec_word["end"] if prec_word else 0.0)

        if prec_gap >= 1.5:
            bout_start = max(0.0, s_time - 2.0)
            bout_end = s_time + 1.0
            exchanges.append({
                "start_ms": int(bout_start * 1000),
                "end_ms": int(bout_end * 1000),
                "stop_word": s_st["word"]
            })

    exchanges.sort(key=lambda x: x["start_ms"])
    filtered_exchanges = _filter_overlapping_bouts(exchanges, min_gap_sec=3.0)
    return filtered_exchanges, all_words


def _detect_exchanges(wav_path: str):
    # 1. Read WAV and compute RMS peaks
    try:
        rate, data = _read_wav_mono(wav_path)
        frame_len = int(rate * 0.05) # 50ms frames
        num_frames = len(data) // frame_len
        rms = np.array([np.sqrt(np.mean(data[i*frame_len : (i+1)*frame_len]**2)) for i in range(num_frames)])
        times = np.arange(num_frames) * 0.05

        peaks = []
        threshold = 0.06
        for i in range(1, num_frames - 1):
            if rms[i] > threshold and rms[i] > rms[i-1] and rms[i] > rms[i+1]:
                peaks.append((times[i], rms[i]))
        print(f"  Loaded {len(peaks)} audio clash peaks (RMS > {threshold})", flush=True)
    except Exception as e:
        print(f"  Warning: Audio envelope analysis failed ({e}). Falling back to empty peaks.", flush=True)
        peaks = []

    print(f"  Loading Whisper model '{WHISPER_MODEL_NAME}' on-demand...", flush=True)
    _device = "cpu"
    try:
        import torch
        if torch.cuda.is_available():
            _device = "cuda"
    except ImportError:
        pass

    model = whisper.load_model(WHISPER_MODEL_NAME, device=_device)

    try:
        result = model.transcribe(
            wav_path,
            language="ru",
            word_timestamps=True,
            condition_on_previous_text=False,
            logprob_threshold=None,
            no_speech_threshold=None
        )
        exchanges, all_words = _extract_decision_tree_exchanges(result.get("segments", []), peaks)

        for idx, ex in enumerate(exchanges):
            print(f"  Exchange {idx+1}: {ex['start_ms']}ms – {ex['end_ms']}ms ('{ex.get('stop_word', '')}')", flush=True)

        return exchanges, all_words
    finally:
        print("  Unloading Whisper model to free server RAM...", flush=True)
        del model
        gc.collect()
        try:
            import torch
            if torch.cuda.is_available():
                torch.cuda.empty_cache()
        except ImportError:
            pass


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


def _process_analyze_sync(body: AnalyzeRequest):
    try:
        # 1. Download audio to a temp file
        with tempfile.TemporaryDirectory() as tmpdir:
            raw_path = os.path.join(tmpdir, "raw_audio")
            wav_path = os.path.join(tmpdir, "audio.wav")

            print("  Downloading audio...", flush=True)
            with requests.get(body.audio_url, stream=True, timeout=120) as r:
                r.raise_for_status()
                with open(raw_path, "wb") as f:
                    for chunk in r.iter_content(chunk_size=65536):
                        f.write(chunk)

            # 2. Convert to 16 kHz mono WAV using ffmpeg
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
                return {"error": f"ffmpeg error: {msg}"}

            # 3. Run Whisper detection
            print("  Running Whisper detection...", flush=True)
            exchanges, all_words = _detect_exchanges(wav_path)

        return {"video_id": body.video_id, "exchanges": exchanges, "words": all_words}

    except Exception as exc:
        traceback.print_exc(file=sys.stderr)
        return {"error": str(exc)}


@app.post("/analyze")
async def analyze(body: AnalyzeRequest):
    print(f"[analyze] video_id={body.video_id} queued in request lock.", flush=True)
    async with analyze_lock:
        print(f"[analyze] video_id={body.video_id} processing started.", flush=True)
        loop = asyncio.get_running_loop()
        result = await loop.run_in_executor(None, _process_analyze_sync, body)
        print(f"[analyze] video_id={body.video_id} processing finished.", flush=True)
        return result
