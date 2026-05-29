import os
import time
import urllib.request
import numpy as np
import cv2
import openvino as ov
from ultralytics import YOLO

def benchmark_native_openvino(xml_path, device_name, num_runs=100):
    print(f"\n--- Native OpenVINO Benchmark on: {device_name.upper()} ---")
    try:
        core = ov.Core()
        
        # Замеряем время компиляции модели
        start_compile = time.time()
        model = core.read_model(xml_path)
        compiled_model = core.compile_model(model, device_name)
        compile_time = time.time() - start_compile
        print(f"Model compiled for {device_name.upper()} in {compile_time:.3f} seconds.")
        
        # Подготавливаем фейковый входной тензор (BCHW: 1x3x640x640)
        input_data = np.random.rand(1, 3, 640, 640).astype(np.float32)
        
        # Разогрев (Warmup)
        for _ in range(10):
            _ = compiled_model([input_data])
            
        # Замеряем чистую скорость инференса
        start_infer = time.time()
        for _ in range(num_runs):
            _ = compiled_model([input_data])
        infer_time = time.time() - start_infer
        
        avg_time_ms = (infer_time / num_runs) * 1000
        pure_fps = num_runs / infer_time
        print(f"Pure Inference Speed: {pure_fps:.2f} FPS (avg {avg_time_ms:.2f} ms per frame)")
        return True, pure_fps
    except Exception as e:
        print(f"Failed native benchmark on {device_name.upper()}: {e}")
        return False, 0.0

def main():
    print("=== Errant Fox 2.0: Pose Estimation PoC ===")
    
    # 1. Проверяем доступные устройства в OpenVINO
    devices = []
    try:
        core = ov.Core()
        print("OpenVINO version:", core.get_versions("CPU")["CPU"].build_number)
        devices = core.available_devices
        print("Available OpenVINO devices:", devices)
    except Exception as e:
        print("Failed to load OpenVINO core/devices:", e)
        return

    # 2. Создаем директорию для временных файлов
    scratch_dir = os.path.dirname(os.path.abspath(__file__))
    os.makedirs(scratch_dir, exist_ok=True)
    
    video_path = os.path.join(scratch_dir, "people-detection.mp4")
    
    # 3. Скачиваем тестовое видео, если его нет
    if not os.path.exists(video_path):
        url = "https://github.com/intel-iot-devkit/sample-videos/raw/master/people-detection.mp4"
        print(f"Downloading sample video from: {url}")
        try:
            urllib.request.urlretrieve(url, video_path)
            print(f"Downloaded test video to: {video_path}")
        except Exception as e:
            print("Failed to download sample video:", e)
            return

    # 4. Загружаем стандартную PyTorch модель YOLOv8-Pose (сохраняя ее в scratch_dir)
    pt_model_path = os.path.join(scratch_dir, "yolov8n-pose.pt")
    print(f"Loading YOLOv8-pose model (PyTorch) from {pt_model_path}...")
    model = YOLO(pt_model_path)
    
    # 5. Экспортируем модель в формат OpenVINO
    ov_model_path = os.path.join(scratch_dir, "yolov8n-pose_openvino_model")
    xml_path = os.path.join(ov_model_path, "yolov8n-pose.xml")
    if not os.path.exists(ov_model_path) or not os.path.exists(xml_path):
        print("Exporting YOLOv8-pose to OpenVINO IR format (FP16)...")
        try:
            model.export(format="openvino", half=True)
            print("Export completed successfully.")
        except Exception as e:
            print("Failed to export model to OpenVINO:", e)
            return

    # 6. Загружаем оптимизированную модель OpenVINO в Ultralytics (для CPU)
    print("\nLoading OpenVINO-optimized YOLO model in Ultralytics...")
    try:
        model_to_use = YOLO(ov_model_path, task="pose")
        print("Successfully loaded OpenVINO model in Ultralytics!")
    except Exception as e:
        print("Failed to load OpenVINO model in Ultralytics:", e)
        return

    # 7. Замеряем нативную производительность OpenVINO на всех устройствах (CPU/GPU)
    for dev in devices:
        benchmark_native_openvino(xml_path, dev)

    # 8. Запускаем сквозной трекинг на CPU через Ultralytics (с декодированием видео и трекером)
    print("\n>>> Running End-to-End Tracking (Ultralytics + OpenVINO CPU) <<<")
    cap = cv2.VideoCapture(video_path)
    if not cap.isOpened():
        print(f"Error: Could not open video file: {video_path}")
        return

    frame_count = int(cap.get(cv2.CAP_PROP_FRAME_COUNT))
    width = int(cap.get(cv2.CAP_PROP_FRAME_WIDTH))
    height = int(cap.get(cv2.CAP_PROP_FRAME_HEIGHT))
    fps = cap.get(cv2.CAP_PROP_FPS)
    print(f"Video Info: {width}x{height} @ {fps:.2f} FPS. Total frames: {frame_count}")

    processed_count = 0
    start_time = time.time()
    
    # Прогоняем первые 50 кадров
    while processed_count < 50:
        ret, frame = cap.read()
        if not ret:
            break
            
        # Запускаем инференс на CPU через Ultralytics
        results = model_to_use.track(frame, persist=True, verbose=False, device='cpu')
        processed_count += 1
        
        # Отладка
        if processed_count == 10:
            result = results[0]
            boxes = result.boxes
            keypoints = result.keypoints
            if keypoints is not None and len(keypoints) > 0:
                print(f"  [Frame 10 Debug] Found {len(keypoints)} persons:")
                for i, person_kps in enumerate(keypoints):
                    track_id = int(boxes[i].id[0].item()) if boxes[i].id is not None else -1
                    kps_xy = person_kps.xy[0]
                    kps_conf = person_kps.conf[0]
                    head_conf = kps_conf[:5].mean().item() if len(kps_conf) >= 5 else 0.0
                    print(f"    - ID: {track_id}, Head Conf: {head_conf:.3f}, Keypoints: {len(kps_xy)}")
            else:
                print("  [Frame 10 Debug] No keypoints detected.")

    end_time = time.time()
    elapsed = end_time - start_time
    actual_fps = processed_count / elapsed
    print(f"End-to-End Speed: {actual_fps:.2f} FPS (Processed {processed_count} frames in {elapsed:.2f}s)")
    cap.release()

    print("\n=== PoC Run Finished ===")

if __name__ == "__main__":
    main()
