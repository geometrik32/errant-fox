use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use anyhow::{Context, Result};
use tokio::sync::{watch, Mutex, Semaphore};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranscodeState {
    InProgress,
    Ready,
    Failed(String),
}

pub struct TranscodeManager {
    pub temp_dir: PathBuf,
    pub server_port: u16,
    semaphore: Arc<Semaphore>,
    active_jobs: Arc<Mutex<HashMap<String, watch::Receiver<TranscodeState>>>>,
    pub ws_hub: crate::services::ws::WsHub,
}

/// Ensures temporary transcode `.tmp` files are removed on error, cancellation, or panic.
struct TmpFileGuard(Option<PathBuf>);

impl TmpFileGuard {
    fn disarm(&mut self) {
        self.0 = None;
    }
}

impl Drop for TmpFileGuard {
    fn drop(&mut self) {
        if let Some(path) = self.0.take() {
            let _ = std::fs::remove_file(path);
        }
    }
}

impl TranscodeManager {
    pub fn new(temp_dir: PathBuf, server_port: u16, ws_hub: crate::services::ws::WsHub) -> Arc<Self> {
        let mgr = Arc::new(Self {
            temp_dir,
            server_port,
            semaphore: Arc::new(Semaphore::new(1)),
            active_jobs: Arc::new(Mutex::new(HashMap::new())),
            ws_hub,
        });

        // Ensure temp directory exists and clean orphaned temporary files from previous runs
        let _ = std::fs::create_dir_all(&mgr.temp_dir);
        Self::clean_orphaned_temp_files_sync(&mgr.temp_dir);

        // Start background TTL cleaner
        let cleaner_mgr = mgr.clone();
        tokio::spawn(async move {
            cleaner_mgr.run_cleaner_loop().await;
        });

        mgr
    }

    pub async fn active_job_ids(&self) -> std::collections::HashSet<String> {
        let jobs = self.active_jobs.lock().await;
        jobs.keys().cloned().collect()
    }

    pub fn target_path(&self, video_id: &str) -> PathBuf {
        self.temp_dir.join(format!("h264_{}.mp4", video_id))
    }

    pub fn tmp_path(&self, video_id: &str) -> PathBuf {
        self.temp_dir.join(format!("h264_{}.mp4.tmp", video_id))
    }

    /// Check status: (is_ready, is_in_progress)
    pub async fn check_status(&self, video_id: &str) -> (bool, bool) {
        let final_path = self.target_path(video_id);
        if final_path.exists() {
            return (true, false);
        }
        let jobs = self.active_jobs.lock().await;
        let in_progress = jobs.contains_key(video_id);
        (false, in_progress)
    }

    /// Touch file mtime to extend TTL on playback
    pub async fn touch(&self, video_id: &str) {
        let path = self.target_path(video_id);
        if path.exists() {
            let _ = tokio::task::spawn_blocking(move || {
                if let Ok(file) = std::fs::File::open(&path) {
                    let _ = file.set_modified(SystemTime::now());
                }
            })
            .await;
        }
    }

    /// Start transcode task or subscribe to existing one.
    pub async fn start_or_subscribe(
        self: &Arc<Self>,
        video_id: &str,
    ) -> watch::Receiver<TranscodeState> {
        let final_path = self.target_path(video_id);
        if final_path.exists() {
            let (_, rx) = watch::channel(TranscodeState::Ready);
            return rx;
        }

        let mut jobs = self.active_jobs.lock().await;
        if let Some(rx) = jobs.get(video_id) {
            return rx.clone();
        }

        let (tx, rx) = watch::channel(TranscodeState::InProgress);
        jobs.insert(video_id.to_string(), rx.clone());
        drop(jobs);

        let _ = self.ws_hub.send(crate::services::ws::WsEvent::UpdateVideoTranscode {
            video_id: video_id.to_string(),
            has_h264: false,
            is_transcoding: true,
        });

        let mgr = self.clone();
        let vid = video_id.to_string();
        tokio::spawn(async move {
            let res = mgr.run_transcode(&vid).await;
            let (final_state, has_h264) = match res {
                Ok(_) => (TranscodeState::Ready, true),
                Err(e) => {
                    tracing::error!("Transcode failed for video {}: {:?}", vid, e);
                    (TranscodeState::Failed(e.to_string()), false)
                }
            };
            let _ = tx.send(final_state);
            let mut jobs = mgr.active_jobs.lock().await;
            jobs.remove(&vid);
            drop(jobs);

            let _ = mgr.ws_hub.send(crate::services::ws::WsEvent::UpdateVideoTranscode {
                video_id: vid,
                has_h264,
                is_transcoding: false,
            });
        });

        rx
    }

    async fn run_transcode(&self, video_id: &str) -> Result<()> {
        let _permit = self.semaphore.acquire().await.context("acquire transcode semaphore")?;

        tokio::time::timeout(Duration::from_secs(1800), self.run_transcode_inner(video_id))
            .await
            .context("transcode timed out after 30 minutes")?
    }

    async fn run_transcode_inner(&self, video_id: &str) -> Result<()> {

        let final_path = self.target_path(video_id);
        if final_path.exists() {
            return Ok(());
        }

        let tmp_path = self.tmp_path(video_id);
        let _ = tokio::fs::remove_file(&tmp_path).await;
        let mut tmp_guard = TmpFileGuard(Some(tmp_path.clone()));

        let local_stream_url = format!(
            "http://127.0.0.1:{}/api/videos/{}/stream",
            self.server_port, video_id
        );
        const FAKE_USER_AGENT: &str = "Mozilla/5.0";

        // Check for Linux VAAPI device (Intel QuickSync)
        let vaapi_available = Path::new("/dev/dri/renderD128").exists();
        let mut success = false;

        if vaapi_available {
            tracing::info!("Starting VAAPI hardware transcode for video {}", video_id);
            let status = tokio::process::Command::new("ffmpeg")
                .kill_on_drop(true)
                .arg("-y")
                .arg("-hwaccel")
                .arg("vaapi")
                .arg("-hwaccel_device")
                .arg("/dev/dri/renderD128")
                .arg("-hwaccel_output_format")
                .arg("vaapi")
                .arg("-user_agent")
                .arg(FAKE_USER_AGENT)
                .arg("-i")
                .arg(&local_stream_url)
                .arg("-fps_mode")
                .arg("passthrough")
                .arg("-c:v")
                .arg("h264_vaapi")
                .arg("-b:v")
                .arg("8M")
                .arg("-maxrate")
                .arg("12M")
                .arg("-bufsize")
                .arg("16M")
                .arg("-c:a")
                .arg("aac")
                .arg("-b:a")
                .arg("128k")
                .arg("-ac")
                .arg("2")
                .arg("-movflags")
                .arg("+faststart")
                .arg("-f")
                .arg("mp4")
                .arg(&tmp_path)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::piped())
                .status()
                .await;

            match status {
                Ok(s) if s.success() => {
                    tracing::info!("VAAPI transcode completed successfully for video {}", video_id);
                    success = true;
                }
                Ok(s) => {
                    tracing::warn!(
                        "VAAPI transcode failed with exit code {:?}, falling back to CPU",
                        s.code()
                    );
                    let _ = tokio::fs::remove_file(&tmp_path).await;
                }
                Err(e) => {
                    tracing::warn!("Failed to spawn VAAPI ffmpeg: {:?}, falling back to CPU", e);
                    let _ = tokio::fs::remove_file(&tmp_path).await;
                }
            }
        }

        if !success {
            // Attempt NVIDIA NVENC hardware encoder
            tracing::info!("Attempting NVIDIA NVENC hardware transcode for video {}", video_id);
            let nvenc_output = tokio::process::Command::new("ffmpeg")
                .kill_on_drop(true)
                .arg("-y")
                .arg("-user_agent")
                .arg(FAKE_USER_AGENT)
                .arg("-i")
                .arg(&local_stream_url)
                .arg("-fps_mode")
                .arg("passthrough")
                .arg("-c:v")
                .arg("h264_nvenc")
                .arg("-pix_fmt")
                .arg("yuv420p")
                .arg("-preset")
                .arg("p1")
                .arg("-b:v")
                .arg("8M")
                .arg("-maxrate")
                .arg("12M")
                .arg("-bufsize")
                .arg("16M")
                .arg("-c:a")
                .arg("aac")
                .arg("-b:a")
                .arg("128k")
                .arg("-ac")
                .arg("2")
                .arg("-movflags")
                .arg("+faststart")
                .arg("-f")
                .arg("mp4")
                .arg(&tmp_path)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::piped())
                .output()
                .await;

            match nvenc_output {
                Ok(out) if out.status.success() => {
                    tracing::info!("NVENC transcode completed successfully for video {}", video_id);
                    success = true;
                }
                Ok(out) => {
                    tracing::warn!(
                        "NVENC transcode failed (exit code {:?}), falling back to CPU. stderr:\n{}",
                        out.status.code(),
                        String::from_utf8_lossy(&out.stderr)
                    );
                    let _ = tokio::fs::remove_file(&tmp_path).await;
                }
                Err(e) => {
                    tracing::warn!("Failed to spawn NVENC ffmpeg: {:?}, falling back to Intel/CPU", e);
                    let _ = tokio::fs::remove_file(&tmp_path).await;
                }
            }
        }

        if !success {
            // Attempt Intel VAAPI hardware encoder (Linux / Docker with /dev/dri)
            if std::path::Path::new("/dev/dri/renderD128").exists() || cfg!(target_os = "linux") {
                tracing::info!("Attempting Intel VAAPI hardware transcode for video {}", video_id);
                let vaapi_output = tokio::process::Command::new("ffmpeg")
                    .kill_on_drop(true)
                    .arg("-y")
                    .arg("-vaapi_device")
                    .arg("/dev/dri/renderD128")
                    .arg("-user_agent")
                    .arg(FAKE_USER_AGENT)
                    .arg("-i")
                    .arg(&local_stream_url)
                    .arg("-vf")
                    .arg("format=nv12,hwupload")
                    .arg("-c:v")
                    .arg("h264_vaapi")
                    .arg("-b:v")
                    .arg("8M")
                    .arg("-maxrate")
                    .arg("12M")
                    .arg("-bufsize")
                    .arg("16M")
                    .arg("-c:a")
                    .arg("aac")
                    .arg("-b:a")
                    .arg("128k")
                    .arg("-ac")
                    .arg("2")
                    .arg("-movflags")
                    .arg("+faststart")
                    .arg("-f")
                    .arg("mp4")
                    .arg(&tmp_path)
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::piped())
                    .output()
                    .await;

                match vaapi_output {
                    Ok(out) if out.status.success() => {
                        tracing::info!("Intel VAAPI transcode completed successfully for video {}", video_id);
                        success = true;
                    }
                    Ok(out) => {
                        tracing::warn!(
                            "Intel VAAPI transcode failed (exit code {:?}), falling back: {}",
                            out.status.code(),
                            String::from_utf8_lossy(&out.stderr)
                        );
                        let _ = tokio::fs::remove_file(&tmp_path).await;
                    }
                    Err(e) => {
                        tracing::warn!("Failed to spawn VAAPI ffmpeg: {:?}, falling back", e);
                        let _ = tokio::fs::remove_file(&tmp_path).await;
                    }
                }
            }
        }

        if !success {
            // Attempt Intel QSV hardware encoder (Windows or Linux with QSV runtime)
            tracing::info!("Attempting Intel QSV hardware transcode for video {}", video_id);
            let qsv_output = tokio::process::Command::new("ffmpeg")
                .kill_on_drop(true)
                .arg("-y")
                .arg("-user_agent")
                .arg(FAKE_USER_AGENT)
                .arg("-i")
                .arg(&local_stream_url)
                .arg("-c:v")
                .arg("h264_qsv")
                .arg("-preset")
                .arg("veryfast")
                .arg("-b:v")
                .arg("8M")
                .arg("-maxrate")
                .arg("12M")
                .arg("-bufsize")
                .arg("16M")
                .arg("-c:a")
                .arg("aac")
                .arg("-b:a")
                .arg("128k")
                .arg("-ac")
                .arg("2")
                .arg("-movflags")
                .arg("+faststart")
                .arg("-f")
                .arg("mp4")
                .arg(&tmp_path)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::piped())
                .output()
                .await;

            match qsv_output {
                Ok(out) if out.status.success() => {
                    tracing::info!("Intel QSV transcode completed successfully for video {}", video_id);
                    success = true;
                }
                Ok(out) => {
                    tracing::warn!(
                        "Intel QSV transcode failed (exit code {:?}), falling back: {}",
                        out.status.code(),
                        String::from_utf8_lossy(&out.stderr)
                    );
                    let _ = tokio::fs::remove_file(&tmp_path).await;
                }
                Err(e) => {
                    tracing::warn!("Failed to spawn QSV ffmpeg: {:?}, falling back", e);
                    let _ = tokio::fs::remove_file(&tmp_path).await;
                }
            }
        }

        if !success {
            tracing::info!("Starting CPU libx264 ultrafast transcode for video {}", video_id);
            let output = tokio::process::Command::new("ffmpeg")
                .kill_on_drop(true)
                .arg("-y")
                .arg("-user_agent")
                .arg(FAKE_USER_AGENT)
                .arg("-i")
                .arg(&local_stream_url)
                .arg("-fps_mode")
                .arg("passthrough")
                .arg("-c:v")
                .arg("libx264")
                .arg("-preset")
                .arg("ultrafast")
                .arg("-crf")
                .arg("22")
                .arg("-c:a")
                .arg("aac")
                .arg("-b:a")
                .arg("128k")
                .arg("-ac")
                .arg("2")
                .arg("-movflags")
                .arg("+faststart")
                .arg("-f")
                .arg("mp4")
                .arg(&tmp_path)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::piped())
                .output()
                .await
                .context("spawn ffmpeg libx264")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let _ = tokio::fs::remove_file(&tmp_path).await;
                anyhow::bail!(
                    "ffmpeg libx264 failed (exit={:?}):\n{}",
                    output.status.code(),
                    stderr
                );
            }
        }

        tokio::fs::rename(&tmp_path, &final_path)
            .await
            .context("rename temp transcode file to final")?;
        tmp_guard.disarm();

        tracing::info!("Transcode finished: saved to {:?}", final_path);
        Ok(())
    }

    fn clean_orphaned_temp_files_sync(temp_dir: &Path) {
        let Ok(entries) = std::fs::read_dir(temp_dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let is_orphan = (name.starts_with("h264_") && name.ends_with(".tmp"))
                || name.starts_with("input_")
                || name.starts_with("trimmed_")
                || name.starts_with("opt_in_")
                || name.starts_with("opt_out_");
            if is_orphan {
                tracing::info!("Removing orphaned temp file on startup: {:?}", path);
                let _ = std::fs::remove_file(&path);
            }
        }
    }

    async fn run_cleaner_loop(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(24 * 3600));
        loop {
            interval.tick().await;
            self.clean_expired_files().await;
        }
    }

    async fn clean_expired_files(&self) {
        let ttl = Duration::from_secs(7 * 24 * 3600);
        let tmp_ttl = Duration::from_secs(3600);
        let task_tmp_ttl = Duration::from_secs(6 * 3600);
        let active_ids = self.active_job_ids().await;
        let mut read_dir = match tokio::fs::read_dir(&self.temp_dir).await {
            Ok(rd) => rd,
            Err(e) => {
                tracing::warn!("Failed to read temp directory for transcode cleanup: {:?}", e);
                return;
            }
        };

        while let Ok(Some(entry)) = read_dir.next_entry().await {
            let path = entry.path();
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if file_name.starts_with("h264_")
                && (file_name.ends_with(".mp4") || file_name.ends_with(".tmp"))
            {
                let is_tmp = file_name.ends_with(".tmp");
                let vid_opt = file_name
                    .strip_prefix("h264_")
                    .and_then(|s| s.strip_suffix(".mp4.tmp").or_else(|| s.strip_suffix(".mp4")));

                if is_tmp {
                    if let Some(vid) = vid_opt {
                        if active_ids.contains(vid) {
                            continue;
                        }
                    }
                }

                let max_age = if is_tmp { tmp_ttl } else { ttl };
                if let Ok(metadata) = entry.metadata().await {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(elapsed) = modified.elapsed() {
                            if elapsed > max_age {
                                tracing::info!("Deleting expired transcode file: {:?}", path);
                                let _ = tokio::fs::remove_file(&path).await;
                                if !is_tmp {
                                    if let Some(vid) = vid_opt {
                                        let _ = self.ws_hub.send(crate::services::ws::WsEvent::UpdateVideoTranscode {
                                            video_id: vid.to_string(),
                                            has_h264: false,
                                            is_transcoding: false,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            } else if file_name.starts_with("keyframes_") && file_name.ends_with(".json") {
                if let Ok(metadata) = entry.metadata().await {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(elapsed) = modified.elapsed() {
                            if elapsed > ttl {
                                tracing::info!("Deleting expired keyframes cache: {:?}", path);
                                let _ = tokio::fs::remove_file(&path).await;
                            }
                        }
                    }
                }
            } else if file_name.starts_with("input_")
                || file_name.starts_with("trimmed_")
                || file_name.starts_with("opt_in_")
                || file_name.starts_with("opt_out_")
            {
                if let Ok(metadata) = entry.metadata().await {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(elapsed) = modified.elapsed() {
                            if elapsed > task_tmp_ttl {
                                tracing::info!("Deleting stale temp file: {:?}", path);
                                let _ = tokio::fs::remove_file(&path).await;
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("errant_fox_test_{}_{}", name, uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_tmp_file_guard_drop_and_disarm() {
        let dir = make_temp_dir("guard");
        let p_drop = dir.join("h264_drop.mp4.tmp");
        let p_keep = dir.join("h264_keep.mp4.tmp");
        std::fs::write(&p_drop, b"partial").unwrap();
        std::fs::write(&p_keep, b"complete").unwrap();

        {
            let _guard_drop = TmpFileGuard(Some(p_drop.clone()));
            let mut guard_keep = TmpFileGuard(Some(p_keep.clone()));
            guard_keep.disarm();
        }

        assert!(!p_drop.exists(), "dropped guard should remove temp file");
        assert!(p_keep.exists(), "disarmed guard should keep file");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_clean_orphaned_temp_files_sync() {
        let dir = make_temp_dir("orphan");
        let orphan_tmp = dir.join("h264_v1.mp4.tmp");
        let orphan_in = dir.join("input_v1.mp4");
        let orphan_trim = dir.join("trimmed_v1.mp4");
        let orphan_opt_in = dir.join("opt_in_v1.mp4");
        let orphan_opt_out = dir.join("opt_out_v1.mp4");
        let valid_cache = dir.join("h264_v1.mp4");
        let valid_kf = dir.join("keyframes_v1.json");

        for p in [&orphan_tmp, &orphan_in, &orphan_trim, &orphan_opt_in, &orphan_opt_out, &valid_cache, &valid_kf] {
            std::fs::write(p, b"data").unwrap();
        }

        TranscodeManager::clean_orphaned_temp_files_sync(&dir);

        assert!(!orphan_tmp.exists());
        assert!(!orphan_in.exists());
        assert!(!orphan_trim.exists());
        assert!(!orphan_opt_in.exists());
        assert!(!orphan_opt_out.exists());
        assert!(valid_cache.exists());
        assert!(valid_kf.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn test_clean_expired_files_respects_ttl_and_active_jobs() {
        let dir = make_temp_dir("ttl");
        let (ws_tx, _ws_rx) = tokio::sync::broadcast::channel(16);
        let mgr = Arc::new(TranscodeManager {
            temp_dir: dir.clone(),
            server_port: 3000,
            semaphore: Arc::new(Semaphore::new(1)),
            active_jobs: Arc::new(Mutex::new(HashMap::new())),
            ws_hub: ws_tx,
        });

        let old_h264 = dir.join("h264_old.mp4");
        let fresh_h264 = dir.join("h264_fresh.mp4");
        let old_tmp_inactive = dir.join("h264_dead.mp4.tmp");
        let old_tmp_active = dir.join("h264_busy.mp4.tmp");
        let old_kf = dir.join("keyframes_old.json");
        let old_input = dir.join("input_stale.mp4");

        for p in [&old_h264, &fresh_h264, &old_tmp_inactive, &old_tmp_active, &old_kf, &old_input] {
            std::fs::write(p, b"data").unwrap();
        }

        let eight_days_ago = SystemTime::now() - Duration::from_secs(8 * 24 * 3600);
        let two_hours_ago = SystemTime::now() - Duration::from_secs(2 * 3600);
        let seven_hours_ago = SystemTime::now() - Duration::from_secs(7 * 3600);

        std::fs::OpenOptions::new().write(true).open(&old_h264).unwrap().set_modified(eight_days_ago).unwrap();
        std::fs::OpenOptions::new().write(true).open(&old_kf).unwrap().set_modified(eight_days_ago).unwrap();
        std::fs::OpenOptions::new().write(true).open(&old_tmp_inactive).unwrap().set_modified(two_hours_ago).unwrap();
        std::fs::OpenOptions::new().write(true).open(&old_tmp_active).unwrap().set_modified(two_hours_ago).unwrap();
        std::fs::OpenOptions::new().write(true).open(&old_input).unwrap().set_modified(seven_hours_ago).unwrap();

        let (_tx, rx) = tokio::sync::watch::channel(TranscodeState::InProgress);
        mgr.active_jobs.lock().await.insert("busy".to_string(), rx);

        mgr.clean_expired_files().await;

        assert!(!old_h264.exists(), "expired h264 cache should be deleted");
        assert!(fresh_h264.exists(), "fresh h264 cache should stay");
        assert!(!old_tmp_inactive.exists(), "inactive stale .tmp should be deleted");
        assert!(old_tmp_active.exists(), "active .tmp must not be deleted");
        assert!(!old_kf.exists(), "expired keyframes cache should be deleted");
        assert!(!old_input.exists(), "stale input file should be deleted");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
