ALTER TABLE videos ADD COLUMN is_tournament INTEGER NOT NULL DEFAULT 0;
ALTER TABLE videos ADD COLUMN tournament_name TEXT;
UPDATE users SET display_name = 'Вне клуба', color = '#475569', avatar_path = 'guest.jpg' WHERE id = 'guest';
