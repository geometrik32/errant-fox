import os
import sys
from ultralytics import YOLO

def train_model(dataset_dir, epochs=50, model_name="yolov8m-pose.pt"):
    print("=== Errant Fox 2.0: Model Fine-Tuning Script ===")
    
    # Нормализуем путь к датасету
    dataset_dir = os.path.abspath(dataset_dir)
    if not os.path.exists(dataset_dir):
        print(f"Error: Dataset directory '{dataset_dir}' not found.")
        print("Please extract your annotated zip file there.")
        return

    scratch_dir = os.path.dirname(os.path.abspath(__file__))
    yaml_path = os.path.join(scratch_dir, "hema_data.yaml")
    
    # 1. Автоматически генерируем hema_data.yaml для Ultralytics
    # Пути внутри yaml должны указывать на папки внутри распакованного датасета
    yaml_content = f"""
path: {dataset_dir} # путь к корню датасета
train: images/train  # путь к обучающим картинкам относительно path
val: images/val      # путь к валидационным картинкам относительно path

# Настройка ключевых точек (17 точек COCO, каждая содержит x, y, visibility)
kpt_shape: [17, 3]

names:
  0: person
"""
    with open(yaml_path, "w", encoding="utf-8") as f:
        f.write(yaml_content.strip())
    print(f"Generated datasets configuration at: {yaml_path}")

    # 2. Загружаем предобученную базовую модель
    # Мы дообучаем ту же Medium модель, которую использовали для теста
    pt_model_path = os.path.join(scratch_dir, model_name)
    print(f"Loading base model: {model_name}...")
    
    try:
        model = YOLO(pt_model_path)
    except Exception:
        model = YOLO(model_name)

    # 3. Запускаем дообучение
    # Так как PyTorch CUDA недоступен на Python 3.14 под Windows, обучение будет идти на CPU.
    # Для 50 картинок и 50 эпох на CPU это займет порядка 15-30 минут на Ryzen 3700.
    # В продакшене или при больших объемах рекомендуется залить это в Google Colab с бесплатной GPU.
    print(f"\nStarting training for {epochs} epochs on CPU...")
    print("This will take some time. Sit back and relax.")
    
    try:
        model.train(
            data=yaml_path,
            epochs=epochs,
            imgsz=640,
            batch=4,          # Маленький batch size для экономии оперативной памяти на CPU
            device="cpu",     # Принудительно CPU
            workers=2,        # Ограничиваем количество потоков
            project=os.path.join(scratch_dir, "training_runs"),
            name="hema_yolov8m_pose"
        )
        print("\nTraining completed successfully!")
        
        # Модель сохранится в папку: scratch/training_runs/hema_yolov8m_pose/weights/best.pt
        best_model_path = os.path.join(scratch_dir, "training_runs", "hema_yolov8m_pose", "weights", "best.pt")
        print(f"Your custom fine-tuned model weights are saved at: {best_model_path}")
        print("To test it, run visualize_pose_v2.py passing this path as the third argument!")
        
    except Exception as e:
        print("\nError occurred during training:", e)

if __name__ == "__main__":
    scratch_dir = os.path.dirname(os.path.abspath(__file__))
    dataset_folder = os.path.join(scratch_dir, "hema_dataset")
    
    epochs = 50
    if len(sys.argv) > 1:
        epochs = int(sys.argv[1])
        
    train_model(dataset_folder, epochs=epochs)
