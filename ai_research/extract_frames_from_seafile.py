import os
import json
import urllib.request
import urllib.parse
import cv2
import time

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
                            video_paths.append(f"{dir_name}/{name}")
            except Exception as e:
                print(f"  Failed to scan folder '{dir_name}': {e}")
                
    return video_paths

def get_download_url(file_path):
    """Получает временную прямую ссылку на скачивание файла из Seafile."""
    api_path = f"/{file_path}"
    data = api_request("/api/v2.1/via-repo-token/download-link/", {"path": api_path})
    return data

def extract_balanced_frames():
    print("=== Errant Fox 2.0: Multi-Video Seafile Frame Extractor (Progressive OpenCV Mode) ===")
    
    # 1. Получаем список всех видео
    videos = get_all_videos()
    num_videos = len(videos)
    print(f"\nFound {num_videos} video files in Seafile.")
    
    if num_videos == 0:
        print("No videos found to process.")
        return
        
    ai_dir = os.path.dirname(os.path.abspath(__file__))
    output_dir = os.path.join(ai_dir, "dataset_to_label")
    os.makedirs(output_dir, exist_ok=True)
    
    # Мы будем извлекать по 3 кадра из каждого видео: на 5-й, 10-й и 15-й секунде.
    # Это даст сбалансированный и разнообразный датасет: 144 видео * 3 кадра = 432 кадра.
    # Последовательное чтение без seek-запросов работает очень быстро и обходит защиту 403.
    target_timestamps = [5.0, 10.0, 15.0]
    success_count = 0
    start_time = time.time()
    
    # 2. Обрабатываем каждое видео
    for idx, video_path in enumerate(videos):
        print(f"\n[{idx+1}/{num_videos}] Processing: {video_path}")
        
        try:
            download_url = get_download_url(video_path)
            
            # Открываем поток через OpenCV
            cap = cv2.VideoCapture(download_url)
            if not cap.isOpened():
                print(f"  Error: OpenCV could not open stream for {video_path}")
                continue
                
            fps = cap.get(cv2.CAP_PROP_FPS)
            total_frames = int(cap.get(cv2.CAP_PROP_FRAME_COUNT))
            
            if fps <= 0 or fps > 200:
                fps = 100.0  # Дефолт для GoPro-видео
                
            # Если видео слишком короткое, извлекаем только один кадр посередине
            if total_frames > 0 and total_frames < (15.0 * fps):
                print(f"  Video is too short ({total_frames} frames). Extracting 1 frame in the middle.")
                target_frames = [total_frames // 2]
            else:
                target_frames = [int(t * fps) for t in target_timestamps]
            
            current_frame = 0
            
            for t_idx, target_f in enumerate(target_frames):
                # Читаем кадры последовательно до цели
                while current_frame < target_f:
                    ret = cap.grab()
                    if not ret:
                        break
                    current_frame += 1
                
                # Декодируем и сохраняем кадр
                ret, frame = cap.retrieve()
                if ret and frame is not None:
                    safe_video_name = video_path.replace("/", "_")
                    frame_name = f"{safe_video_name}_frame_{current_frame:05d}.jpg"
                    output_path = os.path.join(output_dir, frame_name)
                    
                    cv2.imwrite(output_path, frame)
                    success_count += 1
                else:
                    print(f"    Failed to retrieve frame at index {target_f}")
            
            cap.release()
            print(f"  Successfully extracted frames from {video_path}")
            
        except Exception as e:
            print(f"  Error during processing {video_path}: {e}")
            
    elapsed = time.time() - start_time
    print(f"\n=== Process Completed ===")
    print(f"Successfully saved {success_count} frames to: {output_dir}")
    print(f"Total time elapsed: {elapsed:.2f} seconds.")
    print("These frames are ready for upload and annotation in CVAT/Roboflow.")

if __name__ == "__main__":
    extract_balanced_frames()
