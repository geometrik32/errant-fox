from sqlalchemy import Column, Integer, String, Boolean, ForeignKey, DateTime, Float
from sqlalchemy.orm import relationship
from sqlalchemy.sql import func
from app.database import Base

class User(Base):
    __tablename__ = "users"

    id = Column(Integer, primary key=True, index=True)
    username = Column(String, unique=True, index=True, nullable=False)
    hashed_password = Column(String, nullable=False)
    is_active = Column(Boolean, default=True)

    games = relationship("Game", back_populates="owner", cascade="all, delete-orphan")
    saves = relationship("Save", back_populates="owner", cascade="all, delete-orphan")
    playtime_logs = relationship("PlaytimeLog", back_populates="owner", cascade="all, delete-orphan")
    achievements = relationship("Achievement", back_populates="owner", cascade="all, delete-orphan")


class Game(Base):
    __tablename__ = "games"

    id = Column(Integer, primary key=True, index=True)
    playnite_id = Column(String, index=True, nullable=False)  # Playnite's Game GUID
    name = Column(String, nullable=False)
    s3_build_key = Column(String, nullable=True)  # Path to zip/7z build in MinIO
    version = Column(String, nullable=True)
    is_installed_cloud = Column(Boolean, default=False)
    
    user_id = Column(Integer, ForeignKey("users.id", ondelete="CASCADE"), nullable=False)
    owner = relationship("User", back_populates="games")


class Save(Base):
    __tablename__ = "saves"

    id = Column(Integer, primary key=True, index=True)
    playnite_id = Column(String, nullable=False)
    s3_save_key = Column(String, nullable=False)  # Path to save zip in MinIO
    updated_at = Column(DateTime(timezone=True), onupdate=func.now(), default=func.now())

    user_id = Column(Integer, ForeignKey("users.id", ondelete="CASCADE"), nullable=False)
    owner = relationship("User", back_populates="saves")


class PlaytimeLog(Base):
    __tablename__ = "playtime_logs"

    id = Column(Integer, primary key=True, index=True)
    playnite_id = Column(String, nullable=False)
    duration_seconds = Column(Integer, nullable=False)
    logged_at = Column(DateTime(timezone=True), server_default=func.now())

    user_id = Column(Integer, ForeignKey("users.id", ondelete="CASCADE"), nullable=False)
    owner = relationship("User", back_populates="playtime_logs")


class Achievement(Base):
    __tablename__ = "achievements"

    id = Column(Integer, primary key=True, index=True)
    playnite_id = Column(String, nullable=False)
    api_name = Column(String, nullable=False)  # Internal name (e.g. ACH_LEVEL_10)
    name = Column(String, nullable=False)      # Display name
    description = Column(String, nullable=True)
    unlocked = Column(Boolean, default=False)
    unlock_time = Column(DateTime(timezone=True), nullable=True)

    user_id = Column(Integer, ForeignKey("users.id", ondelete="CASCADE"), nullable=False)
    owner = relationship("User", back_populates="achievements")
