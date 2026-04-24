-- Up migration
ALTER TABLE hema_bouts ADD COLUMN start_timecode VARCHAR;
ALTER TABLE hema_bouts ADD COLUMN end_timecode VARCHAR;
