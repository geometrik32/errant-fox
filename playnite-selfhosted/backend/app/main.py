from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from app.database import engine, Base
from app.routers import auth, games, saves, achievements, playtime
from app.s3 import ensure_buckets_exist

# Create database tables
Base.metadata.create_all(bind=engine)

app = FastAPI(
    title="Playnite Self-Hosted Backend API",
    description="Backend API for managing game builds, cloud saves, session playtime logs, and achievements for Playnite launcher.",
    version="1.0.0"
)

# Enable CORS
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Startup event to ensure S3 buckets exist
@app.on_event("startup")
def on_startup():
    ensure_buckets_exist()

# Include routers
app.include_router(auth.router)
app.include_router(games.router)
app.include_router(saves.router)
app.include_router(achievements.router)
app.include_router(playtime.router)

@app.get("/")
def read_root():
    return {
        "status": "online",
        "service": "Playnite Self-Hosted Backend API",
        "version": "1.0.0"
    }
