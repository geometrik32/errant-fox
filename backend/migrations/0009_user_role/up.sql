ALTER TABLE users ADD COLUMN role TEXT NOT NULL DEFAULT 'fighter';
UPDATE users SET role = 'guest' WHERE id = 'guest' OR id LIKE 'guest_%';
