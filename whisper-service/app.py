import json
import multiprocessing
import os
import queue
import re
import subprocess
import sys
import tempfile
import threading
import time
import traceback

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


import asyncio


def _detect_exchanges_with_model(model, wav_path: str):
    print("  Transcribing audio via Faster-Whisper model...", flush=True)
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


def _worker_main(model_name: str, req_q: multiprocessing.Queue, res_q: multiprocessing.Queue):
    """Isolated child process hosting WhisperModel so OS reclaims 100% RAM on exit."""
    try:
        from faster_whisper import WhisperModel
        print(f"[WhisperWorker] Loading Faster-Whisper model '{model_name}' (from disk cache into RAM)...", flush=True)
        t0 = time.time()
        model = WhisperModel(model_name, device="cpu", compute_type="int8")
        print(f"[WhisperWorker] Model loaded successfully in {time.time() - t0:.2f}s.", flush=True)
        res_q.put(("ready", None))
    except Exception as exc:
        res_q.put(("err", f"Failed to load WhisperModel: {exc}\n{traceback.format_exc()}"))
        return

    while True:
        wav_path = req_q.get()
        if wav_path is None:
            print("[WhisperWorker] Shutdown signal received. Exiting.", flush=True)
            break
        try:
            result = _detect_exchanges_with_model(model, wav_path)
            res_q.put(("ok", result))
        except Exception as exc:
            res_q.put(("err", f"{exc}\n{traceback.format_exc()}"))


class DynamicWhisperModel:
    def __init__(self, model_name: str, idle_timeout_sec: float = 300.0):
        self.model_name = model_name
        self.idle_timeout_sec = idle_timeout_sec
        self._ctx = multiprocessing.get_context("spawn")
        self._process = None
        self._req_queue = None
        self._res_queue = None
        self._busy = False
        self.last_used = 0.0
        # Protects only fast in-memory state transitions; never held across blocking I/O or join().
        self._state_lock = threading.Lock()
        # Serializes transcribe calls against the worker queues.
        self._op_lock = threading.Lock()

    @property
    def is_loaded(self) -> bool:
        with self._state_lock:
            proc = self._process
        return proc is not None and proc.is_alive()

    def _detach_worker_locked(self):
        detached = (self._process, self._req_queue, self._res_queue)
        self._process = None
        self._req_queue = None
        self._res_queue = None
        return detached

    def _detach_if_same(self, proc):
        with self._state_lock:
            if self._process is proc:
                return self._detach_worker_locked()
        return (None, None, None)

    @staticmethod
    def _dispose_worker(proc, req_q, res_q):
        if proc is not None:
            if proc.is_alive() and req_q is not None:
                try:
                    req_q.put_nowait(None)
                    proc.join(timeout=5)
                except Exception:
                    pass
            if proc.is_alive():
                proc.terminate()
                proc.join(timeout=3)
            if proc.is_alive():
                proc.kill()
                proc.join(timeout=2)
        for q in (req_q, res_q):
            if q is not None:
                try:
                    q.close()
                    q.join_thread()
                except Exception:
                    pass

    def _ensure_worker(self):
        dead = None
        with self._state_lock:
            if self._process is not None and self._process.is_alive():
                return self._process, self._req_queue, self._res_queue
            if self._process is not None:
                dead = self._detach_worker_locked()

        if dead is not None and dead[0] is not None:
            self._dispose_worker(*dead)

        req_q = self._ctx.Queue()
        res_q = self._ctx.Queue()
        proc = self._ctx.Process(
            target=_worker_main,
            args=(self.model_name, req_q, res_q),
            daemon=True,
        )
        proc.start()

        # Wait until model finishes loading in child process without holding _state_lock
        while True:
            try:
                status, payload = res_q.get(timeout=0.5)
                if status == "ready":
                    with self._state_lock:
                        self._process = proc
                        self._req_queue = req_q
                        self._res_queue = res_q
                    return proc, req_q, res_q
                self._dispose_worker(proc, req_q, res_q)
                raise RuntimeError(payload or "Whisper worker failed during initialization")
            except queue.Empty:
                if not proc.is_alive():
                    exitcode = proc.exitcode
                    self._dispose_worker(proc, req_q, res_q)
                    raise RuntimeError(f"Whisper worker exited during startup (exitcode={exitcode})")

    def transcribe(self, wav_path: str):
        with self._op_lock:
            with self._state_lock:
                self._busy = True
                self.last_used = time.time()
            try:
                proc, req_q, res_q = self._ensure_worker()
                req_q.put(wav_path)
                while True:
                    try:
                        status, payload = res_q.get(timeout=0.5)
                        if status == "ok":
                            return payload
                        raise RuntimeError(payload or "Whisper worker transcription failed")
                    except queue.Empty:
                        if not proc.is_alive():
                            exitcode = proc.exitcode
                            detached = self._detach_if_same(proc)
                            if detached[0] is not None:
                                self._dispose_worker(*detached)
                            raise RuntimeError(f"Whisper worker crashed during transcription (exitcode={exitcode})")
            finally:
                with self._state_lock:
                    self._busy = False
                    self.last_used = time.time()

    def check_unload(self):
        detached = None
        reason = None
        with self._state_lock:
            if self._busy or self._process is None:
                return
            if not self._process.is_alive():
                detached = self._detach_worker_locked()
                reason = "dead"
            elif (time.time() - self.last_used) > self.idle_timeout_sec:
                print(
                    f"[WhisperModel] Inactivity timeout ({self.idle_timeout_sec}s) reached. "
                    "Terminating worker process to release RAM...",
                    flush=True,
                )
                detached = self._detach_worker_locked()
                reason = "timeout"

        if detached is not None and detached[0] is not None:
            self._dispose_worker(*detached)
            if reason == "timeout":
                print("[WhisperModel] Worker process terminated. OS RAM freed.", flush=True)

    def shutdown(self):
        with self._state_lock:
            detached = self._detach_worker_locked()
        if detached[0] is not None:
            self._dispose_worker(*detached)


model_manager = DynamicWhisperModel(WHISPER_MODEL_NAME, idle_timeout_sec=300.0)


async def _auto_unload_loop():
    while True:
        await asyncio.sleep(30)
        try:
            await asyncio.to_thread(model_manager.check_unload)
        except Exception as e:
            print(f"[WhisperModel] Error in auto-unload loop: {e}", flush=True)


@app.on_event("startup")
async def startup_event():
    asyncio.create_task(_auto_unload_loop())
    print("[WhisperModel] Background auto-unload task started.", flush=True)


@app.on_event("shutdown")
async def shutdown_event():
    model_manager.shutdown()


def _detect_exchanges(wav_path: str):
    return model_manager.transcribe(wav_path)


# ── Request / Response schemas ────────────────────────────────────────────────

class AnalyzeRequest(BaseModel):
    audio_url: str
    video_id: str


# ── Endpoints ─────────────────────────────────────────────────────────────────

@app.get("/health")
def health():
    return {"status": "ok", "model_loaded": model_manager.is_loaded}


analyze_lock = asyncio.Lock()
cancelled_video_ids: set[str] = set()


@app.post("/cancel/{video_id}")
async def cancel_video(video_id: str):
    """Mark a video_id as cancelled so queued /analyze requests skip it immediately."""
    cancelled_video_ids.add(video_id)
    print(f"[cancel] video_id={video_id} marked as cancelled. Queue will skip it.", flush=True)
    return {"status": "cancelled", "video_id": video_id}


from urllib.parse import urlparse

def _validate_url(url: str):
    parsed = urlparse(url)
    if parsed.scheme not in ("http", "https"):
        raise ValueError(f"Invalid URL scheme '{parsed.scheme}'. Only http and https are allowed.")
    allowed_hosts = os.environ.get("ALLOWED_AUDIO_HOSTS", "").strip()
    if allowed_hosts:
        allowed_list = [h.strip() for h in allowed_hosts.split(",") if h.strip()]
        if parsed.hostname not in allowed_list and parsed.netloc not in allowed_list:
            raise ValueError(f"Host '{parsed.hostname}' is not in ALLOWED_AUDIO_HOSTS list.")

def _download_and_convert_audio(audio_url: str, tmpdir: str) -> str:
    _validate_url(audio_url)
    wav_path = os.path.join(tmpdir, "audio.wav")

    print(f"  Streaming audio directly via ffmpeg from URL...", flush=True)
    try:
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
            timeout=180,
        )
        if result.returncode != 0:
            err_msg = result.stderr[-1000:] if result.stderr else "ffmpeg exited with non-zero code"
            print(f"  ffmpeg error:\n{err_msg}", flush=True)
            raise RuntimeError(f"ffmpeg failed to stream audio from URL: {err_msg}")
        return wav_path
    except subprocess.TimeoutExpired:
        raise RuntimeError("ffmpeg conversion timed out after 180 seconds")


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
            from fastapi import HTTPException
            raise HTTPException(status_code=500, detail=str(exc))
        finally:
            try:
                import shutil
                shutil.rmtree(tmpdir, ignore_errors=True)
            except Exception:
                pass
