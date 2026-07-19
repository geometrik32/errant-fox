from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy.orm import Session
from typing import List

from app.database import get_db
from app import models, schemas
from app.routers.auth import get_current_user
from app.s3 import get_presigned_download_url, get_presigned_upload_url
from app.config import settings

router = APIRouter(prefix="/games", tags=["games"])

@router.get("/", response_model=List[schemas.GameResponse])
def get_user_games(
    db: Session = Depends(get_db),
    current_user: models.User = Depends(get_current_user)
):
    return db.query(models.Game).filter(models.Game.user_id == current_user.id).all()

@router.post("/", response_model=schemas.GameResponse, status_code=status.HTTP_201_CREATED)
def register_game(
    game_in: schemas.GameCreate,
    db: Session = Depends(get_db),
    current_user: models.User = Depends(get_current_user)
):
    db_game = db.query(models.Game).filter(
        models.Game.playnite_id == game_in.playnite_id,
        models.Game.user_id == current_user.id
    ).first()
    
    if db_game:
        # Update name/version if game already registered
        db_game.name = game_in.name
        if game_in.version:
            db_game.version = game_in.version
        db.commit()
        db.refresh(db_game)
        return db_game

    new_game = models.Game(
        playnite_id=game_in.playnite_id,
        name=game_in.name,
        version=game_in.version,
        user_id=current_user.id
    )
    db.add(new_game)
    db.commit()
    db.refresh(new_game)
    return new_game

@router.get("/{playnite_id}", response_model=schemas.GameResponse)
def get_game(
    playnite_id: str,
    db: Session = Depends(get_db),
    current_user: models.User = Depends(get_current_user)
):
    game = db.query(models.Game).filter(
        models.Game.playnite_id == playnite_id,
        models.Game.user_id == current_user.id
    ).first()
    
    if not game:
        raise HTTPException(status_code=404, detail="Game not found")
    return game

@router.post("/{playnite_id}/upload-url")
def get_game_upload_url(
    playnite_id: str,
    extension: str = "7z",  # e.g., zip or 7z
    db: Session = Depends(get_db),
    current_user: models.User = Depends(get_current_user)
):
    game = db.query(models.Game).filter(
        models.Game.playnite_id == playnite_id,
        models.Game.user_id == current_user.id
    ).first()
    
    if not game:
        raise HTTPException(status_code=404, detail="Game not registered")
        
    s3_key = f"builds/{current_user.id}/{playnite_id}.{extension}"
    game.s3_build_key = s3_key
    db.commit()
    
    upload_url = get_presigned_upload_url(settings.bucket_games, s3_key)
    return {"upload_url": upload_url, "s3_key": s3_key}

@router.post("/{playnite_id}/confirm-upload")
def confirm_game_upload(
    playnite_id: str,
    db: Session = Depends(get_db),
    current_user: models.User = Depends(get_current_user)
):
    game = db.query(models.Game).filter(
        models.Game.playnite_id == playnite_id,
        models.Game.user_id == current_user.id
    ).first()
    
    if not game:
        raise HTTPException(status_code=404, detail="Game not found")
        
    game.is_installed_cloud = True
    db.commit()
    return {"status": "success", "message": "Game build marked as available in cloud"}

@router.get("/{playnite_id}/download-url")
def get_game_download_url(
    playnite_id: str,
    db: Session = Depends(get_db),
    current_user: models.User = Depends(get_current_user)
):
    game = db.query(models.Game).filter(
        models.Game.playnite_id == playnite_id,
        models.Game.user_id == current_user.id
    ).first()
    
    if not game or not game.is_installed_cloud or not game.s3_build_key:
        raise HTTPException(status_code=404, detail="Game build not available in cloud")
        
    download_url = get_presigned_download_url(settings.bucket_games, game.s3_build_key)
    return {"download_url": download_url}
