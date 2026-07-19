from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy.orm import Session
from typing import List

from app.database import get_db
from app import models, schemas
from app.routers.auth import get_current_user

router = APIRouter(prefix="/playtime", tags=["playtime"])

@router.post("/log", response_model=schemas.PlaytimeLogResponse, status_code=status.HTTP_201_CREATED)
def log_playtime(
    log_in: schemas.PlaytimeLogCreate,
    db: Session = Depends(get_db),
    current_user: models.User = Depends(get_current_user)
):
    # Verify game is registered
    game = db.query(models.Game).filter(
        models.Game.playnite_id == log_in.playnite_id,
        models.Game.user_id == current_user.id
    ).first()
    
    if not game:
        raise HTTPException(status_code=404, detail="Game not registered for this user")
        
    new_log = models.PlaytimeLog(
        playnite_id=log_in.playnite_id,
        duration_seconds=log_in.duration_seconds,
        user_id=current_user.id
    )
    db.add(new_log)
    db.commit()
    db.refresh(new_log)
    return new_log

@router.get("/{playnite_id}", response_model=List[schemas.PlaytimeLogResponse])
def get_playtime_history(
    playnite_id: str,
    db: Session = Depends(get_db),
    current_user: models.User = Depends(get_current_user)
):
    return db.query(models.PlaytimeLog).filter(
        models.PlaytimeLog.playnite_id == playnite_id,
        models.PlaytimeLog.user_id == current_user.id
    ).order_by(models.PlaytimeLog.logged_at.desc()).all()
