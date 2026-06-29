CREATE TABLE bout_history (
    id          INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    bout_id     INTEGER NOT NULL REFERENCES bouts(id) ON DELETE CASCADE,
    user_id     TEXT NOT NULL REFERENCES users(id),
    action      TEXT NOT NULL,
    details     TEXT,
    created_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_bout_history_bout_id ON bout_history(bout_id);
