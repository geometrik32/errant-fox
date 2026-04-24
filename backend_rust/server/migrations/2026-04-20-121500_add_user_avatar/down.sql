-- SQLite doesn't support DROP COLUMN in older versions, but for newer ones it does.
-- However, since this is clapshot.sqlite, we usually recreate the table if we really need to downgrade.
-- For now, just a dummy down script or leave it empty as it's hard to downgrade SQLite columns.
-- Actually, newer SQLite (3.35.0+) supports DROP COLUMN.
ALTER TABLE users DROP COLUMN avatar;
