CREATE TABLE users (
    id           TEXT PRIMARY KEY NOT NULL,
    username     TEXT UNIQUE NOT NULL,
    display_name TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    is_admin     BOOLEAN NOT NULL DEFAULT 0,
    avatar_path  TEXT,
    color        TEXT,
    created_at   TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE videos (
    id           TEXT PRIMARY KEY NOT NULL,
    seafile_path TEXT UNIQUE NOT NULL,
    fighter_a_id TEXT REFERENCES users(id),
    fighter_b_id TEXT REFERENCES users(id),
    date         DATE NOT NULL,
    duration_ms  INTEGER,
    preview_count INTEGER NOT NULL DEFAULT 0,
    created_at   TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE techniques (
    id   INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT UNIQUE NOT NULL
);

CREATE TABLE bouts (
    id             INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    video_id       TEXT NOT NULL REFERENCES videos(id),
    order_index    INTEGER NOT NULL,
    time_start_ms  INTEGER NOT NULL,
    time_end_ms    INTEGER NOT NULL,
    score_a        INTEGER NOT NULL DEFAULT 0,
    score_b        INTEGER NOT NULL DEFAULT 0,
    technique_a_id INTEGER REFERENCES techniques(id),
    technique_b_id INTEGER REFERENCES techniques(id),
    hit_zone_a     TEXT,
    hit_zone_b     TEXT,
    result_a       TEXT,
    result_b       TEXT
);

CREATE TABLE comments (
    id           INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    video_id     TEXT NOT NULL REFERENCES videos(id),
    author_id    TEXT NOT NULL REFERENCES users(id),
    timestamp_ms INTEGER NOT NULL,
    text         TEXT NOT NULL,
    reply_to_id  INTEGER REFERENCES comments(id),
    created_at   TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    edited_at    TIMESTAMP
);

CREATE INDEX idx_videos_date      ON videos(date);
CREATE INDEX idx_videos_fighter_a ON videos(fighter_a_id);
CREATE INDEX idx_videos_fighter_b ON videos(fighter_b_id);
CREATE INDEX idx_bouts_video      ON bouts(video_id);
CREATE INDEX idx_comments_video   ON comments(video_id);
CREATE INDEX idx_bouts_techniques ON bouts(technique_a_id, technique_b_id);
