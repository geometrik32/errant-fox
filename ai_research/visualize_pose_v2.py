import os
import sys
import time
import cv2
import torch
from ultralytics import YOLO

class KeypointSmoother:
    def __init__(self, alpha=0.35):
        self.alpha = alpha
        self.history = {}

    def smooth(self, track_id, keypoints_xy, keypoints_conf):
        if track_id not in self.history:
            self.history[track_id] = (keypoints_xy.copy(), keypoints_conf.copy())
            return keypoints_xy, keypoints_conf

        prev_xy, prev_conf = self.history[track_id]
        smoothed_xy = self.alpha * keypoints_xy + (1 - self.alpha) * prev_xy
        smoothed_conf = self.alpha * keypoints_conf + (1 - self.alpha) * prev_conf
        self.history[track_id] = (smoothed_xy, smoothed_conf)
        return smoothed_xy, smoothed_conf

def draw_skeleton(frame, track_id, box_xyxy, conf, smoothed_xy, smoothed_conf, colors, skeleton_connections):
    color = colors[track_id % len(colors)]
    cv2.rectangle(frame, (box_xyxy[0], box_xyxy[1]), (box_xyxy[2], box_xyxy[3]), color, 2)
    cv2.putText(frame, f"ID:{track_id} (Conf: {conf:.2f})", 
                (box_xyxy[0], box_xyxy[1] - 10), cv2.FONT_HERSHEY_SIMPLEX, 0.6, color, 2)
    
    for connection in skeleton_connections:
        pt1_idx, pt2_idx = connection
        if pt1_idx < len(smoothed_xy) and pt2_idx < len(smoothed_xy):
            if smoothed_conf[pt1_idx] > 0.4 and smoothed_conf[pt2_idx] > 0.4:
                pt1 = tuple(smoothed_xy[pt1_idx].astype(int))
                pt2 = tuple(smoothed_xy[pt2_idx].astype(int))
                cv2.line(frame, pt1, pt2, color, 2)
                
    for kp_idx, (pt_x, pt_y) in enumerate(smoothed_xy):
        if smoothed_conf[kp_idx] > 0.4:
            cv2.circle(frame, (int(pt_x), int(pt_y)), 5, (0, 255, 255), -1)

def visualize_video_v2(input_video_path, output_video_path, model_name="yolov8m-pose.pt", conf_threshold=0.75):
    print("=== Errant Fox 2.0: Advanced Pose Visualization (v2.0 with OpenVINO GPU & Frame Skip) ===")
    
    if not os.path.exists(input_video_path):
        print(f"Error: Input video not found at '{input_video_path}'")
        return

    device = "cuda" if torch.cuda.is_available() else "cpu"
    print(f"Detection PyTorch Backend Device: {device.upper()}")

    ai_dir = os.path.dirname(os.path.abspath(__file__))
    pt_model_path = os.path.join(ai_dir, model_name)
    
    base_name = os.path.splitext(model_name)[0]
    ov_model_dir = os.path.join(ai_dir, f"{base_name}_openvino_model")
    
    if not os.path.exists(pt_model_path):
        print(f"Downloading {model_name}...")
        model = YOLO(model_name)
        # Переносим скачанный файл в папку ai_research
        if os.path.exists(model_name):
            os.rename(model_name, pt_model_path)
    
    model = YOLO(pt_model_path)

    if device == "cpu" and not os.path.exists(ov_model_dir):
        print(f"Exporting {model_name} to OpenVINO for acceleration...")
        try:
            model.export(format="openvino", half=True)
            print("Export to OpenVINO completed successfully.")
            # Переносим экспортированную папку, если она создалась в корне
            default_ov_dir = os.path.join(os.getcwd(), f"{base_name}_openvino_model")
            if os.path.exists(default_ov_dir) and default_ov_dir != ov_model_dir:
                os.rename(default_ov_dir, ov_model_dir)
        except Exception as e:
            print("OpenVINO export failed, using PyTorch CPU fallback:", e)

    if device == "cpu" and os.path.exists(ov_model_dir):
        import openvino as ov
        core = ov.Core()
        available_devs = core.available_devices
        print("Available OpenVINO Devices:", available_devs)
        
        if "GPU" in available_devs:
            print(">>> Force-enabling OpenVINO GPU ('intel:gpu') <<<")
            device = "intel:gpu"
        else:
            print("Using OpenVINO CPU mode...")
            device = "cpu"
            
        print(f"Loading OpenVINO model from {ov_model_dir}...")
        model = YOLO(ov_model_dir, task="pose")
    else:
        print(f"Loading standard PyTorch model on device: {device.upper()}...")
        model = YOLO(pt_model_path)

    cap = cv2.VideoCapture(input_video_path)
    if not cap.isOpened():
        print(f"Error: Could not open video file: {input_video_path}")
        return

    width = int(cap.get(cv2.CAP_PROP_FRAME_WIDTH))
    height = int(cap.get(cv2.CAP_PROP_FRAME_HEIGHT))
    fps = cap.get(cv2.CAP_PROP_FPS)
    frame_count = int(cap.get(cv2.CAP_PROP_FRAME_COUNT))
    
    if fps <= 0 or fps > 200:
        fps = 25.0

    print(f"Video resolution: {width}x{height} @ {fps:.2f} FPS. Total frames: {frame_count}")

    skip_interval = 1
    if fps >= 90:
        skip_interval = 4
        print("Detected High-FPS video (>= 90 FPS). Frame skipping enabled (process every 4th frame).")
    elif fps >= 50:
        skip_interval = 2
        print("Detected Mid-FPS video (>= 50 FPS). Frame skipping enabled (process every 2nd frame).")

    fourcc = cv2.VideoWriter_fourcc(*'mp4v')
    out = cv2.VideoWriter(output_video_path, fourcc, fps, (width, height))
    
    smoother = KeypointSmoother(alpha=0.35)
    
    colors = [
        (0, 255, 0), (255, 0, 0), (0, 0, 255),
        (255, 255, 0), (255, 0, 255), (0, 255, 255)
    ]

    skeleton_connections = [
        (5, 6), (5, 7), (7, 9), (6, 8), (8, 10),
        (5, 11), (6, 12), (11, 12), (11, 13),
        (13, 15), (12, 14), (14, 16)
    ]

    roi_max_x = int(width * 0.85)
    print(f"ROI Boundary: Detections with center X > {roi_max_x} will be ignored.")

    start_time = time.time()
    processed = 0
    
    last_skeletons = []

    try:
        while True:
            ret, frame = cap.read()
            if not ret:
                break

            if processed % skip_interval == 0:
                results = model.track(
                    frame, 
                    persist=True, 
                    conf=conf_threshold, 
                    iou=0.5, 
                    verbose=False, 
                    device=device,
                    tracker="bytetrack.yaml"
                )
                
                result = results[0]
                boxes = result.boxes
                keypoints = result.keypoints
                
                last_skeletons = []
                if keypoints is not None and len(keypoints) > 0 and boxes.id is not None:
                    for i in range(len(keypoints)):
                        track_id = int(boxes.id[i].item())
                        box_xyxy = boxes.xyxy[i].cpu().numpy().astype(int)
                        conf = boxes.conf[i].item()
                        
                        box_center_x = (box_xyxy[0] + box_xyxy[2]) // 2
                        if box_center_x > roi_max_x:
                            continue
                            
                        kps_xy = keypoints.xy[i].cpu().numpy()
                        kps_conf = keypoints.conf[i].cpu().numpy()
                        
                        smoothed_xy, smoothed_conf = smoother.smooth(track_id, kps_xy, kps_conf)
                        
                        last_skeletons.append({
                            "track_id": track_id,
                            "box": box_xyxy,
                            "conf": conf,
                            "xy": smoothed_xy,
                            "kps_conf": smoothed_conf
                        })

            annotated_frame = frame.copy()
            
            cv2.line(annotated_frame, (roi_max_x, 0), (roi_max_x, height), (0, 0, 255), 2)
            cv2.putText(annotated_frame, "RING BOUNDARY (ROI)", (roi_max_x - 220, 25), 
                        cv2.FONT_HERSHEY_SIMPLEX, 0.6, (0, 0, 255), 2)

            for skel in last_skeletons:
                draw_skeleton(
                    annotated_frame, 
                    skel["track_id"], 
                    skel["box"], 
                    skel["conf"], 
                    skel["xy"], 
                    skel["kps_conf"], 
                    colors, 
                    skeleton_connections
                )

            out.write(annotated_frame)
            processed += 1
            
            if processed % 100 == 0 or processed == frame_count:
                progress = (processed / frame_count) * 100 if frame_count > 0 else 0
                elapsed_run = time.time() - start_time
                fps_run = processed / elapsed_run
                print(f"Progress: {processed}/{frame_count} frames ({progress:.1f}%) | Speed: {fps_run:.1f} FPS")

    except KeyboardInterrupt:
        print("\nProcessing interrupted.")
    finally:
        cap.release()
        out.release()

    elapsed = time.time() - start_time
    print(f"\nFinished! Processed {processed} frames in {elapsed:.1f} seconds ({processed / elapsed:.2f} FPS).")
    print(f"Output saved to: {output_video_path}")

if __name__ == "__main__":
    ai_dir = os.path.dirname(os.path.abspath(__file__))
    input_path = os.path.join(ai_dir, "input.mp4")
    output_path = os.path.join(ai_dir, "output.mp4")
    model_name = "yolov8m-pose.pt"
    
    if len(sys.argv) > 1:
        input_path = sys.argv[1]
    if len(sys.argv) > 2:
        output_path = sys.argv[2]
    if len(sys.argv) > 3:
        model_name = sys.argv[3]
        
    visualize_video_v2(input_path, output_path, model_name=model_name)
