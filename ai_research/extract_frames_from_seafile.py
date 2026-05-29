import os
import json
import urllib.request
import urllib.parse
import subprocess

# Конфигурация подключения к Seafile
SEAFILE_URL = "https://seafile.aat-terra.ru"
SEAFILE_TOKEN = "1d12d16fdcef05c2be5598a033f120c4aa04e54e"

def api_request(endpoint, params=None):
    """Выполняет авторизованный запрос к API Seafile."""
    url = f"{SEAFILE_URL}{endpoint}"
    if params:
        url += "?" + urllib.parse.urlencode(params)
        
    req = urllib.request.Request(url)
    req.add_header("Authorization", f"Bearer {SEAFILE_TOKEN}")
    
    with urllib.request.urlopen(req) as response:
        return json.loads(response.read().decode())

def get_all_videos():
    """Рекурсивно ищет все файлы .mp4/.MP4 во всех папках библиотеки."""
    print("Listing directories in Seafile root...")
    try:
        root_data = api_request("/api/v2.1/via-repo-token/dir/", {"path": "/"})
    except Exception as e:
        print("Failed to list root directory:", e)
        return []
        
    video_paths = []
    
    # Обходим все папки (например, "2026.02.01")
    for item in root_data.get("dirent_list", []):
        if item.get("type") == "dir":
            dir_name = item.get("name")
            print(f"Scanning folder: {dir_name}...")
            
            try:
                dir_data = api_request("/api/v2.1/via-repo-token/dir/", {"path": f"/{dir_name}"})
                for file_item in dir_data.get("dirent_list", []):
                    if file_item.get("type") == "file":
                        name = file_item.get("name")
                        if name.lower().endswith(".mp4"):
                            # Относительный путь: "2026.02.01/video1.mp4"
                            video_paths.append(f"{dir_name}/{name}")
            except Exception as e:
                print(f"  Failed to scan folder '{dir_name}': {e}")
                
    return video_paths

def get_download_url(file_path):
    """Получает временную прямую ссылку на скачивание файла из Seafile."""
    api_path = f"/{file_path}"
    data = api_request("/api/v2.1/via-repo-token/download-link/", {"path": api_path})
    return data

def get_video_duration_ffmpeg(download_url):
    """Получает длительность видео в секундах с помощью ffprobe с обходом 403."""
    cmd = [
        "ffprobe", "-v", "error",
        "-user_agent", "Mozilla/5.0",
        "-show_entries", "format=duration",
        "-of", "default=noprint_wrappers=1:nokey=1",
        download_url
    ]
    try:
        result = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, timeout=15)
        if result.returncode == 0:
            return float(result.stdout.strip())
        else:
            print(f"  ffprobe failed with exit code {result.returncode}: {result.stderr}")
    except Exception as e:
        print(f"  ffprobe run error: {e}")
    return 0.0

def extract_frame_ffmpeg(download_url, time_seconds, output_path):
    """Надежно извлекает один кадр с медленным поиском для обхода 403."""
    cmd = [
        "ffmpeg", "-y",
        "-user_agent", "Mozilla/5.0",
        "-i", download_url,
        "-ss", f"{time_seconds:.3f}",  # Медленный поиск (после -i) исключает ошибку 403 Forbidden
        "-vframes", "1",
        "-q:v", "2",
        output_path
    ]
    try:
        result = subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=20)
        return result.returncode == 0
    except Exception as e:
        print(f"  ffmpeg run error: {e}")
        return False

def extract_balanced_frames(target_total_frames=500):
    print("=== Errant Fox 2.0: Multi-Video Seafile Frame Extractor (FFmpeg mode v2) ===")
    
    # 1. Получаем список всех видео
    videos = get_all_videos()
    num_videos = len(videos)
    print(f"\nFound {num_videos} video files in Seafile.")
    
    if num_videos == 0:
        print("No videos found to process.")
        return
        
    # Вычисляем, сколько кадров вытащить из каждого видео
    frames_per_video = target_total_frames // num_videos
    if frames_per_video < 1:
        frames_per_video = 1
        
    actual_total_target = frames_per_video * num_videos
    print(f"We will extract {frames_per_video} frames from EACH video (Total target: {actual_total_target} frames).")
    
    ai_dir = os.path.dirname(os.path.abspath(__file__))
    output_dir = os.path.join(ai_dir, "dataset_to_label")
    os.makedirs(output_dir, exist_ok=True)
    
    success_count = 0
    
    # 2. Обрабатываем каждое видео
    for idx, video_path in enumerate(videos):
        print(f"\n[{idx+1}/{num_videos}] Processing: {video_path}")
        
        try:
            download_url = get_download_url(video_path)
            
            # Получаем длительность видео
            duration = get_video_duration_ffmpeg(download_url)
            if duration <= 0:
                print(f"  Warning: Could not read duration for {video_path}. Skipping.")
                continue
                
            print(f"  Duration: {duration:.2f} seconds.")
            
            # Пропускаем первые 10% и последние 10% видео
            start_time = duration * 0.10
            end_time = duration * 0.90
            
            # Рассчитываем временные интервалы
            if end_time - start_time <= frames_per_video:
                step = 1.0
                frames_actual = int(max(1, end_time - start_time))
            else:
                step = (end_time - start_time) / frames_per_video
                frames_actual = frames_per_video
                
            print(f"  Extracting {frames_actual} frames (step: {step:.2f}s)...")
            
            for i in range(frames_actual):
                target_time = start_time + (i * step)
                
                safe_video_name = video_path.replace("/", "_")
                frame_name = f"{safe_video_name}_time_{target_time:.2f}.jpg"
                output_path = os.path.join(output_dir, frame_name)
                
                if extract_frame_ffmpeg(download_url, target_time, output_path):
                    success_count += 1
                else:
                    print(f"    Failed to extract frame at {target_time:.2f}s")
            
            print(f"  Successfully processed {video_path}")
            
        except Exception as e:
            print(f"  Error during processing {video_path}: {e}")
            
    print(f"\n=== Process Completed ===")
    print(f"Successfully saved {success_count} frames to: {output_dir}")
    print("These frames are ready for upload and annotation in CVAT/Roboflow.")

if __name__ == "__main__":
    extract_balanced_frames(target_total_frames=500)
