-- 1. Удаление старых таблиц участников (теперь они в users)
DROP TABLE IF EXISTS hema_media_participants;
DROP TABLE IF EXISTS hema_participants;

-- 2. Пересоздание таблицы пользователей с полем login
CREATE TABLE users_new (
    id TEXT PRIMARY KEY NOT NULL,
    login TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    password_hash TEXT,
    is_admin BOOLEAN NOT NULL DEFAULT 0,
    color TEXT,
    avatar TEXT,
    created TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Копируем старых пользователей (id становится и login и id)
INSERT INTO users_new (id, login, name, password_hash, is_admin, color, avatar, created)
SELECT id, id, name, password_hash, is_admin, color, avatar, created FROM users;

DROP TABLE users;
ALTER TABLE users_new RENAME TO users;

-- 3. Обновление таблицы видео (media_files)
ALTER TABLE media_files ADD COLUMN fight_date TIMESTAMP;
ALTER TABLE media_files ADD COLUMN participant_a_id TEXT REFERENCES users(id);
ALTER TABLE media_files ADD COLUMN participant_b_id TEXT REFERENCES users(id);
ALTER TABLE media_files ADD COLUMN total_score_a INTEGER DEFAULT 0;
ALTER TABLE media_files ADD COLUMN total_score_b INTEGER DEFAULT 0;

-- 4. Обновление таблицы сходов (hema_bouts) - меняем типы ID участников на TEXT
CREATE TABLE hema_bouts_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    video_hash TEXT NOT NULL REFERENCES media_files(id),
    start_time REAL NOT NULL,
    end_time REAL NOT NULL,
    participant_a_id TEXT REFERENCES users(id),
    participant_b_id TEXT REFERENCES users(id),
    score_a INTEGER,
    score_b INTEGER,
    notes TEXT,
    start_timecode TEXT,
    end_timecode TEXT,
    move_a_id INTEGER REFERENCES hema_moves(id),
    move_b_id INTEGER REFERENCES hema_moves(id),
    hit_zone_a TEXT,
    hit_zone_b TEXT
);

-- Копируем данные с приведением типов
INSERT INTO hema_bouts_new (id, video_hash, start_time, end_time, participant_a_id, participant_b_id, score_a, score_b, notes, start_timecode, end_timecode, move_a_id, move_b_id, hit_zone_a, hit_zone_b)
SELECT id, video_hash, start_time, end_time, CAST(participant_a_id AS TEXT), CAST(participant_b_id AS TEXT), score_a, score_b, notes, start_timecode, end_timecode, move_a_id, move_b_id, hit_zone_a, hit_zone_b 
FROM hema_bouts;

DROP TABLE hema_bouts;
ALTER TABLE hema_bouts_new RENAME TO hema_bouts;

-- 5. Удаление функционала субтитров
DROP TABLE IF EXISTS subtitles;
