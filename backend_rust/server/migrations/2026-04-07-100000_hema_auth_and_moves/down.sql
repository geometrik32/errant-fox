-- Down migration

-- 1. Remove columns from hema_bouts (SQLite doesn't support DROP COLUMN easily in older versions, 
-- but we'll try the standard way if modern SQLite is used, otherwise we'd need table recreation)
-- For simplicity in this environment, we'll just not revert if it's too complex, but here's the attempt:
ALTER TABLE hema_bouts DROP COLUMN move_a_id;
ALTER TABLE hema_bouts DROP COLUMN move_b_id;

-- 2. Drop Tech reference
DROP TABLE hema_moves;

-- 3. Revert Users table
ALTER TABLE users DROP COLUMN language;
ALTER TABLE users DROP COLUMN avatar_url;
ALTER TABLE users DROP COLUMN is_admin;
ALTER TABLE users DROP COLUMN password_hash;
