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

_stop_pattern = re.compile(r'\b(стоп|оп|топ|гоп|доп|хоп|альт|stop)\b', re.IGNORECASE)
_start_pattern = re.compile(r'\b(бой|ой|fight|begin|начали)\b', re.IGNORECASE)
_filler_pattern = re.compile(r'^(ха|хаха|ха-ха|хх|э|ээ|хм|мм|гм|ух|ах|ох|фу|ы)$', re.IGNORECASE)


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
            _stop_pattern = re.compile(r'\b(стоп|оп|топ|гоп|доп|хоп|альт|stop)\b', re.IGNORECASE)
            curr_is_explicit = bool(_stop_pattern.search(curr_stop))
            prev_is_explicit = bool(_stop_pattern.search(prev_stop))

            if curr_is_explicit and not prev_is_explicit:
                filtered[-1] = curr
            # Else keep prev, discard duplicate curr
        else:
            filtered.append(curr)

    return filtered


def _extract_decision_tree_exchanges(segments):
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

                if _start_pattern.search(word_clean) and prob >= 0.01:
                    starts.append(item)
                elif _stop_pattern.search(word_clean) and prob >= 0.20:
                    explicit_stops.append(item)

    # Filter close duplicate 'Бой' markers (within 2.5s)
    unique_starts = []
    if starts:
        curr = starts[0]
        for nxt in starts[1:]:
            if nxt["start"] - curr["start"] >= 2.5:
                unique_starts.append(curr)
                curr = nxt
        unique_starts.append(curr)

    exchanges = []
    covered_stop_times = set()

    # Process Start-Anchored Scenarios (1, 3, 4, 5)
    for i, st in enumerate(unique_starts):
        start_time = st["start"]
        next_start_time = unique_starts[i+1]["start"] if i + 1 < len(unique_starts) else 99999.0

        # Scenario 4: Duplicate/Short 'Бой' noise
        if next_start_time - start_time < 2.5:
            continue

        candidates = [
            w for w in all_words
            if w["start"] >= start_time + 0.4 and w["start"] < next_start_time
        ]

        # Scenario 1 & 5: Search for explicit stop words in range
        candidate_stops = [w for w in candidates if _stop_pattern.search(w["clean"]) and w["prob"] >= 0.20]

        selected_stop = None

        if candidate_stops:
            # Scenario 5: Pick the stop with best dialogue proximity
            best_st = candidate_stops[0]
            best_score = -999.0
            for c_st in candidate_stops:
                st_t = c_st["start"]
                foll_word = next((w for w in all_words if w["start"] >= st_t + 0.1), None)
                foll_gap = (foll_word["start"] - st_t) if foll_word else 999.0
                score = c_st["prob"] + (3.0 if foll_gap <= 2.5 else 0.0)
                if score > best_score:
                    best_score = score
                    best_st = c_st
            selected_stop = best_st
        else:
            # Scenario 3: Missing 'Стоп' -> Fallback to first meaningful spoken word after 1.0s (skipping vocal fillers like 'ха-ха')
            first_spoken = next((w for w in candidates if w["start"] >= start_time + 1.0 and not _filler_pattern.search(w["clean"])), None)
            if first_spoken:
                selected_stop = first_spoken

        if selected_stop:
            stop_t = selected_stop["start"]
            bout_start = max(0.0, stop_t - 2.0)
            bout_end = stop_t + 1.0
            covered_stop_times.add(round(stop_t, 2))

            exchanges.append({
                "start_ms": int(bout_start * 1000),
                "end_ms": int(bout_end * 1000),
                "stop_word": selected_stop["word"]
            })

    # Scenario 2: Process Orphaned Stops (Missing 'Бой')
    for s_st in explicit_stops:
        s_time = s_st["start"]
        if round(s_time, 2) in covered_stop_times:
            continue

        prec_word = next((w for w in reversed(all_words) if w["end"] <= s_time - 0.1), None)
        prec_gap = s_time - (prec_word["end"] if prec_word else 0.0)

        # Scenario 6: Ignore isolated stops without preceding combat gap
        if prec_gap >= 1.5:
            bout_start = max(0.0, s_time - 2.0)
            bout_end = s_time + 1.0
            exchanges.append({
                "start_ms": int(bout_start * 1000),
                "end_ms": int(bout_end * 1000),
                "stop_word": s_st["word"]
            })

    exchanges.sort(key=lambda x: x["start_ms"])
    return _filter_overlapping_bouts(exchanges, min_gap_sec=3.0)


def _detect_exchanges(wav_path: str):
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
        exchanges = _extract_decision_tree_exchanges(result.get("segments", []))

        for idx, ex in enumerate(exchanges):
            print(f"  Exchange {idx+1}: {ex['start_ms']}ms – {ex['end_ms']}ms ('{ex.get('stop_word', '')}')", flush=True)

        return exchanges
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


@app.post("/analyze")
def analyze(body: AnalyzeRequest):
    print(f"[analyze] video_id={body.video_id}  url={body.audio_url[:80]}...", flush=True)

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
            exchanges = _detect_exchanges(wav_path)

        return {"video_id": body.video_id, "exchanges": exchanges}

    except Exception as exc:
        traceback.print_exc(file=sys.stderr)
        return {"error": str(exc)}
