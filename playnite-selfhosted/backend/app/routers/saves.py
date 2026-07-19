from fastapi import APIRouter, Depends, HTTPException, status
from sqlalchemy.orm import Session
from sqlalchemy.sql import func
from app.database import get_db
from app import models, schemas
from app.routers.auth import get_current_user
from app.s3 import get_presigned_download_url, get_presigned_upload_url
from app.config import settings

router = APIRouter(prefix="/saves", tags=["saves"])

@router.post("/{playnite_id}/upload-url")
def get_save_upload_url(
    playnite_id: str,
    db: Session = Depends(get_db),
    current_user: models.User = Depends(get_current_user)
):
    s3_key = f"saves/{current_user.id}/{playnite_id}.zip"
    
    save = db.query(models.Save).filter(
        models.Save.playnite_id == playnite_id,
        models.Save.user_id == current_user.id
    ).first()
    
    if not save:
        save = models.Save(
            playnite_id=playnite_id,
            s3_save_key=s3_key,
            user_id=current_user.id
        )
        db.add(save)
    else:
        save.s3_save_key = s3_key
        save.updated_at = func.now()
        
    db.commit()
    
    upload_url = get_presigned_upload_url(settings.bucket_saves, s3_key)
    return {"upload_url": upload_url, "s3_key": s3_key}

@router.get("/{playnite_id}/download-url")
def get_save_download_url(
    playnite_id: str,
    db: Session = Depends(get_db),
    current_user: models.User = Depends(get_current_user)
):
    save = db.query(models.Save).filter(
        models.Save.playnite_id == playnite_id,
        models.Save.user_id == current_user.id
    ).first()
    
    if not save:
        raise HTTPException(status_code=404, detail="No save backup found for this game")
        
    download_url = get_presigned_download_url(settings.bucket_saves, save.s3_save_key)
    return {"download_url": download_url, "updated_at": save.updated_at}
