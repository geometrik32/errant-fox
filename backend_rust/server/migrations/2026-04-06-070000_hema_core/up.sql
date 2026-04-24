-- Up migration
CREATE TABLE hema_participants (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name VARCHAR NOT NULL UNIQUE,
    club VARCHAR
);

CREATE TABLE hema_media_participants (
    video_hash VARCHAR NOT NULL,
    participant_id INTEGER NOT NULL,
    PRIMARY KEY (video_hash, participant_id),
    FOREIGN KEY(video_hash) REFERENCES media_files (id) ON DELETE CASCADE,
    FOREIGN KEY(participant_id) REFERENCES hema_participants (id) ON DELETE CASCADE
);

CREATE TABLE hema_bouts (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    video_hash VARCHAR NOT NULL,
    start_time FLOAT NOT NULL,
    end_time FLOAT NOT NULL,
    participant_a_id INTEGER,
    participant_b_id INTEGER,
    score_a INTEGER DEFAULT 0,
    score_b INTEGER DEFAULT 0,
    notes TEXT,
    FOREIGN KEY(video_hash) REFERENCES media_files (id) ON DELETE CASCADE,
    FOREIGN KEY(participant_a_id) REFERENCES hema_participants (id) ON DELETE SET NULL,
    FOREIGN KEY(participant_b_id) REFERENCES hema_participants (id) ON DELETE SET NULL
);

CREATE INDEX ix_hema_bouts_video_hash ON hema_bouts (video_hash);
