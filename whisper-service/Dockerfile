FROM python:3.10-slim

# Install ffmpeg and build deps
RUN apt-get update && apt-get install -y ffmpeg && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY requirements.txt .
# Install torch CPU version first to keep image smaller (GPU not available in container on this server)
RUN pip install --no-cache-dir torch==2.1.0 --index-url https://download.pytorch.org/whl/cpu
# Install openai-whisper without its deps first (to skip triton which is 197MB and GPU-only)
RUN pip install --no-cache-dir --no-deps openai-whisper
# Install remaining deps (triton excluded via requirements.txt)
RUN pip install --no-cache-dir -r requirements.txt

COPY app.py .

EXPOSE 8000

CMD ["uvicorn", "app:app", "--host", "0.0.0.0", "--port", "8000"]
