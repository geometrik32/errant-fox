#!/usr/bin/env python3
"""
Migrate all GoPro videos in Seafile to have moov atom at the start (faststart).

Run inside the backend Docker container where ffmpeg is available:
    docker exec -it infra-backend-1 python3 /app/migrate_faststart.py

Or from host with access to SQLite and Seafile API:
    python3 infra/migrate_faststart.py

Environment variables (read from .env or environment):
    SEAFILE_URL   — e.g. https://seafile.aat-terra.ru
    SEAFILE_TOKEN — repo API token
    DATABASE_URL  — path to SQLite DB, e.g. /data/db/errant_fox.db
"""

import os
import sys
import sqlite3
import subprocess
import tempfile
import urllib.request
import urllib.error
import json
import argparse
import time

# ── Config ─────────────────────────────────────────────────────────────────────

def load_env():
    """Load .env file from common locations if env vars not set."""
    for path in [".env", "/app/.env", "../.env"]:
        if os.path.exists(path):
            with open(path) as f:
                for line in f:
                    line = line.strip()
                    if line and not line.startswith("#") and "=" in line:
                        k, _, v = line.partition("=")
                        if k.strip() not in os.environ:
                            os.environ[k.strip()] = v.strip()
            break

load_env()

SEAFILE_URL   = os.environ.get("SEAFILE_URL", "").rstrip("/")
SEAFILE_TOKEN = os.environ.get("SEAFILE_TOKEN", "")
DATABASE_URL  = os.environ.get("DATABASE_URL", "/data/db/errant_fox.db")

if not SEAFILE_URL or not SEAFILE_TOKEN:
    sys.exit("ERROR: SEAFILE_URL and SEAFILE_TOKEN must be set")

AUTH_HEADER = f"Bearer {SEAFILE_TOKEN}"

# ── Seafile API ─────────────────────────────────────────────────────────────────

def seafile_get(path):
    url = f"{SEAFILE_URL}{path}"
    req = urllib.request.Request(url, headers={"Authorization": AUTH_HEADER})
    with urllib.request.urlopen(req) as resp:
        return json.loads(resp.read())

def get_download_url(seafile_path):
    api_path = seafile_path if seafile_path.startswith("/") else f"/{seafile_path}"
    quoted = urllib.parse.quote(api_path)
    url = f"{SEAFILE_URL}/api/v2.1/via-repo-token/download-link/?path={quoted}"
    req = urllib.request.Request(url, headers={"Authorization": AUTH_HEADER})
    with urllib.request.urlopen(req) as resp:
        return json.loads(resp.read())  # returns a bare string

def get_upload_url(parent_dir="/"):
    quoted = urllib.parse.quote(parent_dir)
    url = f"{SEAFILE_URL}/api/v2.1/via-repo-token/upload-link/?path={quoted}"
    req = urllib.request.Request(url, headers={"Authorization": AUTH_HEADER})
    with urllib.request.urlopen(req) as resp:
        return json.loads(resp.read())  # returns a bare string

def download_file(download_url, dest_path):
    req = urllib.request.Request(download_url)
    with urllib.request.urlopen(req) as resp, open(dest_path, "wb") as f:
        while True:
            chunk = resp.read(1024 * 1024)
            if not chunk:
                break
            f.write(chunk)

def upload_file(upload_url, seafile_path, local_path):
    """Upload local_path to Seafile, replacing the file at seafile_path."""
    import http.client

    # Build multipart form manually
    boundary = "----MigrationBoundary7f3a9b"
    filename = os.path.basename(seafile_path)
    # parent_dir is the folder part, e.g. /2026.02.01
    parent_dir = "/" + seafile_path.rsplit("/", 1)[0].lstrip("/")

    with open(local_path, "rb") as f:
        file_data = f.read()

    body_parts = []
    # field: parent_dir
    body_parts.append(
        f"--{boundary}\r\n"
        f'Content-Disposition: form-data; name="parent_dir"\r\n\r\n'
        f"{parent_dir}\r\n"
    )
    # field: replace
    body_parts.append(
        f"--{boundary}\r\n"
        f'Content-Disposition: form-data; name="replace"\r\n\r\n'
        f"1\r\n"
    )
    # field: file
    body_parts.append(
        f"--{boundary}\r\n"
        f'Content-Disposition: form-data; name="file"; filename="{filename}"\r\n'
        f"Content-Type: video/mp4\r\n\r\n"
    )

    prefix = "".join(body_parts).encode()
    suffix = f"\r\n--{boundary}--\r\n".encode()
    body = prefix + file_data + suffix

    content_type = f"multipart/form-data; boundary={boundary}"

    from urllib.parse import urlparse
    parsed = urlparse(upload_url)
    is_https = parsed.scheme == "https"
    host = parsed.netloc
    path = parsed.path + (f"?{parsed.query}" if parsed.query else "")

    conn_cls = http.client.HTTPSConnection if is_https else http.client.HTTPConnection
    conn = conn_cls(host, timeout=120)
    conn.request("POST", path, body=body, headers={
        "Content-Type": content_type,
        "Content-Length": str(len(body)),
        "Authorization": AUTH_HEADER,
    })
    resp = conn.getresponse()
    resp_body = resp.read()
    conn.close()

    if resp.status not in (200, 201):
        raise RuntimeError(f"Upload failed: HTTP {resp.status}: {resp_body[:200]}")

# ── Core logic ──────────────────────────────────────────────────────────────────

import urllib.parse

def remux_faststart(input_path, output_path, video_id):
    """Run ffmpeg to copy stream and move moov to start."""
    result = subprocess.run(
        ["ffmpeg", "-y", "-i", input_path,
         "-c", "copy", "-movflags", "+faststart",
         output_path],
        capture_output=True,
    )
    if result.returncode != 0:
        raise RuntimeError(f"ffmpeg failed:\n{result.stderr.decode()[-400:]}")

def process_video(video_id, seafile_path, db_path, dry_run=False):
    print(f"  [{video_id[:8]}] {seafile_path}")

    with tempfile.TemporaryDirectory() as tmp:
        orig = os.path.join(tmp, "orig.mp4")
        fixed = os.path.join(tmp, "fixed.mp4")

        # 1. Download
        t0 = time.time()
        dl_url = get_download_url(seafile_path)
        download_file(dl_url, orig)
        size_mb = os.path.getsize(orig) / 1_048_576
        print(f"    downloaded {size_mb:.0f}MB in {time.time()-t0:.1f}s")

        if dry_run:
            print("    [dry-run] skipping remux and upload")
            return True

        # 2. Remux
        t1 = time.time()
        remux_faststart(orig, fixed, video_id)
        print(f"    remuxed in {time.time()-t1:.1f}s  ({os.path.getsize(fixed)/1_048_576:.0f}MB)")

        # 3. Upload (pass parent dir so Seafile creates the upload link in right folder)
        t2 = time.time()
        parent_dir = "/" + seafile_path.rsplit("/", 1)[0].lstrip("/")
        up_url = get_upload_url(parent_dir)
        upload_file(up_url, seafile_path, fixed)
        print(f"    uploaded in {time.time()-t2:.1f}s")

    # 4. Reset preview_count so it gets regenerated
    conn = sqlite3.connect(db_path)
    conn.execute("UPDATE videos SET preview_count=0 WHERE id=?", (video_id,))
    conn.commit()
    conn.close()

    print(f"    OK  total={time.time()-t0:.1f}s")
    return True

# ── Main ────────────────────────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(description="Migrate videos to faststart MP4")
    parser.add_argument("--dry-run", action="store_true", help="Download + remux only, no upload")
    parser.add_argument("--limit", type=int, default=0, help="Process at most N videos (0=all)")
    parser.add_argument("--id", default="", help="Process only this video ID")
    args = parser.parse_args()

    conn = sqlite3.connect(DATABASE_URL)
    if args.id:
        rows = conn.execute(
            "SELECT id, seafile_path FROM videos WHERE id=?", (args.id,)
        ).fetchall()
    else:
        rows = conn.execute(
            "SELECT id, seafile_path FROM videos ORDER BY date"
        ).fetchall()
    conn.close()

    if args.limit:
        rows = rows[:args.limit]

    total = len(rows)
    print(f"Videos to process: {total}" + (" [DRY RUN]" if args.dry_run else ""))
    print(f"Seafile: {SEAFILE_URL}")
    print(f"DB:      {DATABASE_URL}")
    print()

    failed = []
    for i, (vid_id, seafile_path) in enumerate(rows, 1):
        print(f"[{i}/{total}]")
        try:
            process_video(vid_id, seafile_path, DATABASE_URL, dry_run=args.dry_run)
        except Exception as e:
            print(f"    ERROR: {e}")
            failed.append((vid_id, seafile_path, str(e)))

    print()
    print(f"Done: {total - len(failed)} OK, {len(failed)} failed")
    if failed:
        print("Failed:")
        for vid_id, path, err in failed:
            print(f"  {vid_id[:8]} {path}: {err}")

if __name__ == "__main__":
    main()
