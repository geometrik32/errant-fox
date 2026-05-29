import os
import json
import urllib.request
import urllib.parse
import cv2

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
    """Рекурсивно ищет все файлы .mp4 во всех папках библиотеки."""
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
                    if file_item.get("type") == "file" and file_item.get("name").lower().endswith(".mp4"):
                        # Относительный путь: "2026.02.01/video1.mp4"
                        video_paths.append(f"{dir_name}/{file_item.get('name')}")
            except Exception as e:
                print(f"  Failed to scan folder '{dir_name}': {e}")
                
    return video_paths

def get_download_url(file_path):
    """Получает временную прямую ссылку на скачивание файла из Seafile."""
    # Путь в API должен начинаться со слэша
    api_path = f"/{file_path}"
    data = api_request("/api/v2.1/via-repo-token/download-link/", {"path": api_path})
    return data # API возвращает строку с URL

def extract_balanced_frames(target_total_frames=500):
    print("=== Errant Fox 2.0: Multi-Video Seafile Frame Extractor ===")
    
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
    
    scratch_dir = os.path.dirname(os.path.abspath(__file__))
    output_dir = os.path.join(scratch_dir, "dataset_to_label")
    os.makedirs(output_dir, exist_ok=True)
    
    success_count = 0
    
    # 2. Обрабатываем каждое видео
    for idx, video_path in enumerate(videos):
        print(f"\n[{idx+1}/{num_videos}] Processing: {video_path}")
        
        try:
            # Получаем ссылку на поток
            download_url = get_download_url(video_path)
            
            # Открываем поток через OpenCV
            cap = cv2.VideoCapture(download_url)
            if not cap.isOpened():
                print(f"  Error: OpenCV could not open stream for {video_path}")
                continue
                
            total_frames = int(cap.get(cv2.CAP_PROP_FRAME_COUNT))
            if total_frames <= 0:
                print(f"  Warning: Invalid frame count ({total_frames}) for {video_path}. Skipping.")
                cap.release()
                continue
                
            # Пропускаем первые 10% и последние 10% видео
            start_frame = int(total_frames * 0.10)
            end_frame = int(total_frames * 0.90)
            
            # Вычисляем равномерные шаги для извлечения нужного количества кадров
            if end_frame - start_frame <= frames_per_video:
                step = 1
                frames_per_video_actual = max(1, end_frame - start_frame)
            else:
                step = (end_frame - start_frame) // frames_per_video
                frames_per_video_actual = frames_per_video
                
            print(f"  Extracting {frames_per_video_actual} frames (total frames: {total_frames}, step: {step})...")
            
            for i in range(frames_per_video_actual):
                target_frame = start_frame + (i * step)
                cap.set(cv2.CAP_PROP_POS_FRAMES, target_frame)
                ret, frame = cap.read()
                if not ret:
                    break
                    
                # Имя файла включает дату и название видео, чтобы не было конфликтов
                # Пример: "2026.02.01_fight1.mp4_frame_00520.jpg"
                safe_video_name = video_path.replace("/", "_")
                frame_name = f"{safe_video_name}_frame_{target_frame:05d}.jpg"
                output_path = os.path.join(output_dir, frame_name)
                
                cv2.imwrite(output_path, frame)
                success_count += 1
                
            cap.release()
            print(f"  Successfully extracted frames from {video_path}")
            
        except Exception as e:
            print(f"  Error during processing {video_path}: {e}")
            
    print(f"\n=== Process Completed ===")
    print(f"Successfully saved {success_count} frames to: {output_dir}")
    print("These frames are ready for upload and annotation in CVAT/Roboflow.")

if __name__ == "__main__":
    extract_balanced_frames(target_total_frames=500)
