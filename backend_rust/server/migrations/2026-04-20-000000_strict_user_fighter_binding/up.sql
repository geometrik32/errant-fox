-- 0. Add missing color column if they don't exist
ALTER TABLE users ADD COLUMN color TEXT;
ALTER TABLE hema_participants ADD COLUMN color TEXT;

-- 1. Create missing users for participants

INSERT OR IGNORE INTO users (id, name, created, is_admin, color)
SELECT 
    lower(replace(name, ' ', '-')), 
    name, 
    datetime('now'), 
    0, 
    color
FROM hema_participants 
WHERE account_id IS NULL OR account_id = '';

-- 2. Link participants to their new/existing accounts
UPDATE hema_participants 
SET account_id = lower(replace(name, ' ', '-'))
WHERE account_id IS NULL OR account_id = '';

-- 3. Cleanup ALL related triggers before table recreation to avoid SQLite errors
DROP TRIGGER IF EXISTS tr_hema_delete_participant_on_user_delete;
DROP TRIGGER IF EXISTS tr_hema_delete_user_on_participant_delete;
DROP TRIGGER IF EXISTS tr_hema_create_participant_on_user_insert;
DROP TRIGGER IF EXISTS tr_hema_update_participant_on_user_update;

-- 4. Recreate hema_participants table with NOT NULL account_id
PRAGMA foreign_keys=OFF;

CREATE TABLE hema_participants_new (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name VARCHAR NOT NULL UNIQUE,
    club VARCHAR,
    account_id TEXT NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE ON UPDATE CASCADE,
    color TEXT
);

INSERT INTO hema_participants_new (id, name, club, account_id, color)
SELECT id, name, club, account_id, color FROM hema_participants;

DROP TABLE hema_participants;
ALTER TABLE hema_participants_new RENAME TO hema_participants;

PRAGMA foreign_keys=ON;

-- 5. New trigger: Sync User -> Participant (INSERT)
CREATE TRIGGER tr_hema_create_participant_on_user_insert
AFTER INSERT ON users
FOR EACH ROW
BEGIN
    INSERT OR IGNORE INTO hema_participants (name, account_id, color)
    VALUES (NEW.name, NEW.id, NEW.color);
END;

-- 6. New trigger: Sync User -> Participant (UPDATE)
CREATE TRIGGER tr_hema_update_participant_on_user_update
AFTER UPDATE ON users
FOR EACH ROW
BEGIN
    UPDATE hema_participants 
    SET name = NEW.name, color = NEW.color 
    WHERE account_id = NEW.id;
END;

-- 7. New trigger: Sync User -> Participant (DELETE)
CREATE TRIGGER tr_hema_delete_participant_on_user_delete
AFTER DELETE ON users
FOR EACH ROW
BEGIN
    DELETE FROM hema_participants WHERE account_id = OLD.id;
END;

-- 8. Ensure ALL current users have participants
INSERT OR IGNORE INTO hema_participants (name, account_id, color)
SELECT name, id, color FROM users;
