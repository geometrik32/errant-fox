from pydantic import BaseModel, Field
from typing import Optional, List
from datetime import datetime

# --- User Schemas ---
class UserCreate(BaseModel):
    username: str
    password: str

class UserResponse(BaseModel):
    id: int
    username: str
    is_active: bool

    class Config:
        from_attributes = True

class Token(BaseModel):
    access_token: str
    token_type: str

class TokenData(BaseModel):
    username: Optional[str] = None

# --- Game Schemas ---
class GameCreate(BaseModel):
    playnite_id: str
    name: str
    version: Optional[str] = None

class GameUpdate(BaseModel):
    name: Optional[str] = None
    version: Optional[str] = None
    s3_build_key: Optional[str] = None
    is_installed_cloud: Optional[bool] = None

class GameResponse(BaseModel):
    id: int
    playnite_id: str
    name: str
    s3_build_key: Optional[str] = None
    version: Optional[str] = None
    is_installed_cloud: bool

    class Config:
        from_attributes = True

# --- Save Schemas ---
class SaveResponse(BaseModel):
    playnite_id: str
    s3_save_key: str
    updated_at: datetime

    class Config:
        from_attributes = True

# --- Playtime Schemas ---
class PlaytimeLogCreate(BaseModel):
    playnite_id: str
    duration_seconds: int

class PlaytimeLogResponse(BaseModel):
    id: int
    playnite_id: str
    duration_seconds: int
    logged_at: datetime

    class Config:
        from_attributes = True

# --- Achievement Schemas ---
class AchievementCreate(BaseModel):
    playnite_id: str
    api_name: str
    name: str
    description: Optional[str] = None
    unlocked: bool = False
    unlock_time: Optional[datetime] = None

class AchievementUpdate(BaseModel):
    unlocked: bool
    unlock_time: Optional[datetime] = None

class AchievementResponse(BaseModel):
    id: int
    playnite_id: str
    api_name: str
    name: str
    description: Optional[str] = None
    unlocked: bool
    unlock_time: Optional[datetime] = None

    class Config:
        from_attributes = True
