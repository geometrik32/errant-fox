import os
import cv2

def extract_frames(video_path, output_dir, num_frames=50):
    print("=== Errant Fox 2.0: Dataset Frame Extractor ===")
    
    if not os.path.exists(video_path):
        print(f"Error: Video file '{video_path}' not found.")
        return
        
    os.makedirs(output_dir, exist_ok=True)
    
    cap = cv2.VideoCapture(video_path)
    if not cap.isOpened():
        print("Error: Could not open video file.")
        return
        
    total_frames = int(cap.get(cv2.CAP_PROP_FRAME_COUNT))
    fps = cap.get(cv2.CAP_PROP_FPS)
    print(f"Video contains {total_frames} frames at {fps:.2f} FPS.")
    
    # Чтобы кадры были разнообразными, будем пропускать начало видео (где обычно разминка/пустой зал)
    # и извлекать кадры равномерно с шагом во времени.
    start_frame = int(total_frames * 0.05)  # Пропускаем первые 5%
    end_frame = int(total_frames * 0.95)    # Пропускаем последние 5%
    
    step = (end_frame - start_frame) // num_frames
    if step <= 0:
        step = 1

    print(f"Extracting {num_frames} frames from frame {start_frame} to {end_frame} (step: {step} frames)...")
    
    count = 0
    cap.set(cv2.CAP_PROP_POS_FRAMES, start_frame)
    
    for i in range(num_frames):
        target_frame = start_frame + (i * step)
        cap.set(cv2.CAP_PROP_POS_FRAMES, target_frame)
        ret, frame = cap.read()
        if not ret:
            print("Reached end of video early.")
            break
            
        frame_name = f"frame_{target_frame:05d}.jpg"
        output_path = os.path.join(output_dir, frame_name)
        cv2.imwrite(output_path, frame)
        count += 1
        
    cap.release()
    print(f"\nExtraction complete! Saved {count} images to: {output_dir}")
    print("You can now upload these images to Roboflow or CVAT for keypoint annotation.")

if __name__ == "__main__":
    scratch_dir = os.path.dirname(os.path.abspath(__file__))
    input_video = os.path.join(scratch_dir, "input.mp4")
    output_folder = os.path.join(scratch_dir, "dataset_to_label")
    
    extract_frames(input_video, output_folder)
