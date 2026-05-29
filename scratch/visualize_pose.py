import os
import sys
import time
import cv2
from ultralytics import YOLO

def visualize_video(input_video_path, output_video_path):
    print("=== Errant Fox 2.0: Pose Visualization Tool ===")
    
    # Проверяем наличие входного видео
    if not os.path.exists(input_video_path):
        print(f"Error: Input video not found at '{input_video_path}'")
        print("Please place your video file there or specify the path.")
        return

    scratch_dir = os.path.dirname(os.path.abspath(__file__))
    ov_model_path = os.path.join(scratch_dir, "yolov8n-pose_openvino_model")
    pt_model_path = os.path.join(scratch_dir, "yolov8n-pose.pt")

    # Выбираем модель (OpenVINO если есть, иначе PyTorch)
    if os.path.exists(ov_model_path):
        print("Loading OpenVINO-optimized YOLO model...")
        model = YOLO(ov_model_path, task="pose")
    else:
        print(f"OpenVINO model not found, loading PyTorch model from {pt_model_path}...")
        model = YOLO(pt_model_path)

    # Открываем входное видео
    cap = cv2.VideoCapture(input_video_path)
    if not cap.isOpened():
        print(f"Error: Could not open video file: {input_video_path}")
        return

    # Получаем параметры видео
    width = int(cap.get(cv2.CAP_PROP_FRAME_WIDTH))
    height = int(cap.get(cv2.CAP_PROP_FRAME_HEIGHT))
    fps = cap.get(cv2.CAP_PROP_FPS)
    frame_count = int(cap.get(cv2.CAP_PROP_FRAME_COUNT))
    
    if fps <= 0 or fps > 100:
        fps = 25.0
        
    print(f"Processing Video: {input_video_path}")
    print(f"Dimensions: {width}x{height} @ {fps:.2f} FPS")
    print(f"Total Frames: {frame_count}")

    # Создаем объект записи видео
    # На Windows 'mp4v' или 'xvid' работают лучше всего для MP4 контейнера
    fourcc = cv2.VideoWriter_fourcc(*'mp4v')
    out = cv2.VideoWriter(output_video_path, fourcc, fps, (width, height))
    
    print(f"Saving output to: {output_video_path}")
    
    start_time = time.time()
    processed = 0

    try:
        while True:
            ret, frame = cap.read()
            if not ret:
                break

            # Запускаем трекинг на CPU (для теста)
            # persist=True сохраняет ID трека между кадрами
            results = model.track(frame, persist=True, verbose=False, device='cpu')
            
            # Отрисовываем скелеты, маски детекции и боксы
            # result.plot() возвращает изображение с нанесенной графикой
            annotated_frame = results[0].plot(boxes=True, kpt_line=True)
            
            # Записываем кадр в новый видеофайл
            out.write(annotated_frame)
            
            processed += 1
            if processed % 50 == 0 or processed == frame_count:
                progress = (processed / frame_count) * 100 if frame_count > 0 else 0
                print(f"Progress: {processed}/{frame_count} frames processed ({progress:.1f}%)")

    except KeyboardInterrupt:
        print("\nProcessing interrupted by user.")
    finally:
        cap.release()
        out.release()
        print("Video release successful.")

    elapsed = time.time() - start_time
    print(f"\nFinished! Processed {processed} frames in {elapsed:.1f} seconds.")
    print(f"Output saved to: {output_video_path}")

if __name__ == "__main__":
    # Вы можете запустить скрипт как: python visualize_pose.py [путь_к_видео]
    scratch_dir = os.path.dirname(os.path.abspath(__file__))
    input_path = os.path.join(scratch_dir, "input.mp4")
    output_path = os.path.join(scratch_dir, "output.mp4")
    
    if len(sys.argv) > 1:
        input_path = sys.argv[1]
    if len(sys.argv) > 2:
        output_path = sys.argv[2]
        
    visualize_video(input_path, output_path)
