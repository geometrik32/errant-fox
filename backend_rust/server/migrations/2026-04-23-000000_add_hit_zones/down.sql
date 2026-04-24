-- SQLite doesn't support dropping columns easily, usually done by recreating the table.
-- For this simple migration, we can just leave it as is or use the standard pattern if needed.
-- However, for a dev environment, we usually don't rollback much.
-- To keep it simple, I'll just put a comment here or use the ALTER TABLE DROP COLUMN if the sqlite version supports it (3.35.0+).
ALTER TABLE hema_bouts DROP COLUMN hit_zone_a;
ALTER TABLE hema_bouts DROP COLUMN hit_zone_b;
