DROP TRIGGER IF EXISTS tr_hema_create_participant_on_user_insert;
DROP TRIGGER IF EXISTS tr_hema_update_participant_on_user_update;
DROP TRIGGER IF EXISTS tr_hema_delete_participant_on_user_delete;

PRAGMA foreign_keys=OFF;

CREATE TABLE hema_participants_old (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name VARCHAR NOT NULL UNIQUE,
    club VARCHAR,
    account_id TEXT REFERENCES users(id) ON DELETE SET NULL,
    color TEXT
);

INSERT INTO hema_participants_old (id, name, club, account_id, color)
SELECT id, name, club, account_id, color FROM hema_participants;

DROP TABLE hema_participants;
ALTER TABLE hema_participants_old RENAME TO hema_participants;

PRAGMA foreign_keys=ON;
