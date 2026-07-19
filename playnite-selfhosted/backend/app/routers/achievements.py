from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy.orm import Session
from typing import List
from datetime import datetime

from app.database import get_db
from app import models, schemas
from app.routers.auth import get_current_user

router = APIRouter(prefix="/achievements", tags=["achievements"])

@router.get("/{playnite_id}", response_model=List[schemas.AchievementResponse])
def get_game_achievements(
    playnite_id: str,
    db: Session = Depends(get_db),
    current_user: models.User = Depends(get_current_user)
):
    return db.query(models.Achievement).filter(
        models.Achievement.playnite_id == playnite_id,
        models.Achievement.user_id == current_user.id
    ).all()

@router.post("/{playnite_id}/sync", response_model=List[schemas.AchievementResponse])
def sync_achievements(
    playnite_id: str,
    achievements_in: List[schemas.AchievementCreate],
    db: Session = Depends(get_db),
    current_user: models.User = Depends(get_current_user)
):
    synced_achievements = []
    
    for ach_in in achievements_in:
        # Check if achievement already exists for this user and game
        db_ach = db.query(models.Achievement).filter(
            models.Achievement.playnite_id == playnite_id,
            models.Achievement.api_name == ach_in.api_name,
            models.Achievement.user_id == current_user.id
        ).first()
        
        if db_ach:
            # If it's already unlocked in DB, don't change unlock time. 
            # If client reports it unlocked but DB has it locked, update it.
            if ach_in.unlocked and not db_ach.unlocked:
                db_ach.unlocked = True
                db_ach.unlock_time = ach_in.unlock_time or datetime.utcnow()
                db_ach.name = ach_in.name
                if ach_in.description:
                    db_ach.description = ach_in.description
        else:
            # Create new achievement record
            db_ach = models.Achievement(
                playnite_id=playnite_id,
                api_name=ach_in.api_name,
                name=ach_in.name,
                description=ach_in.description,
                unlocked=ach_in.unlocked,
                unlock_time=ach_in.unlock_time if ach_in.unlocked else None,
                user_id=current_user.id
            )
            db.add(db_ach)
            
        synced_achievements.append(db_ach)
        
    db.commit()
    
    # Refresh all synced items
    for ach in synced_achievements:
        db.refresh(ach)
        
    return synced_achievements
