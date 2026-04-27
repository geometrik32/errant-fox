-- SQLite does not support DROP COLUMN in older versions; recreate table without bout_id
CREATE TABLE comments_backup AS SELECT id, video_id, author_id, timestamp_ms, text, reply_to_id, created_at, edited_at FROM comments;
DROP TABLE comments;
ALTER TABLE comments_backup RENAME TO comments;
