ALTER TABLE hema_participants ADD COLUMN account_id TEXT REFERENCES users(id) ON DELETE SET NULL;
CREATE UNIQUE INDEX idx_hema_participants_account_id ON hema_participants(account_id);

-- Sync existing users into participants table
INSERT INTO hema_participants (name, account_id)
SELECT name, id FROM users
WHERE id NOT IN (SELECT account_id FROM hema_participants WHERE account_id IS NOT NULL);
