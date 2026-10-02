CREATE TABLE notifications (
    id          INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    actor_id    TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    event_type  TEXT NOT NULL,
    video_id    TEXT NOT NULL REFERENCES videos(id) ON DELETE CASCADE,
    bout_id     INTEGER REFERENCES bouts(id) ON DELETE CASCADE,
    comment_id  INTEGER REFERENCES comments(id) ON DELETE CASCADE,
    payload     TEXT,
    is_read     BOOLEAN NOT NULL DEFAULT 0,
    created_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_notifications_user_unread ON notifications(user_id, is_read);
CREATE INDEX idx_notifications_user_created ON notifications(user_id, created_at);
CREATE INDEX idx_notifications_video ON notifications(video_id);
