-- Down migration (Sqlite doesn't support DROP COLUMN in older versions, but we add dummy columns often)
-- For Sqlite 3.35+ it is supported.
ALTER TABLE hema_bouts DROP COLUMN start_timecode;
ALTER TABLE hema_bouts DROP COLUMN end_timecode;
