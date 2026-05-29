import os
import sys
from ultralytics import YOLO

def train_model(dataset_dir, epochs=50, model_name="yolov8m-pose.pt"):
    print("=== Errant Fox 2.0: Model Fine-Tuning Script ===")
    
    dataset_dir = os.path.abspath(dataset_dir)
    if not os.path.exists(dataset_dir):
        print(f"Error: Dataset directory '{dataset_dir}' not found.")
        return

    ai_dir = os.path.dirname(os.path.abspath(__file__))
    yaml_path = os.path.join(ai_dir, "hema_data.yaml")
    
    yaml_content = f"""
path: {dataset_dir} 
train: images/train  
val: images/val      

kpt_shape: [17, 3]

names:
  0: person
"""
    with open(yaml_path, "w", encoding="utf-8") as f:
        f.write(yaml_content.strip())
    print(f"Generated datasets configuration at: {yaml_path}")

    pt_model_path = os.path.join(ai_dir, model_name)
    print(f"Loading base model: {model_name}...")
    
    try:
        model = YOLO(pt_model_path)
    except Exception:
        model = YOLO(model_name)

    print(f"\nStarting training for {epochs} epochs on CPU...")
    
    try:
        model.train(
            data=yaml_path,
            epochs=epochs,
            imgsz=640,
            batch=4,          
            device="cpu",     
            workers=2,        
            project=os.path.join(ai_dir, "training_runs"),
            name="hema_yolov8m_pose"
        )
        print("\nTraining completed successfully!")
        
        best_model_path = os.path.join(ai_dir, "training_runs", "hema_yolov8m_pose", "weights", "best.pt")
        print(f"Your custom weights are saved at: {best_model_path}")
        
    except Exception as e:
        print("\nError occurred during training:", e)

if __name__ == "__main__":
    ai_dir = os.path.dirname(os.path.abspath(__file__))
    dataset_folder = os.path.join(ai_dir, "hema_dataset")
    
    epochs = 50
    if len(sys.argv) > 1:
        epochs = int(sys.argv[1])
        
    train_model(dataset_folder, epochs=epochs)
