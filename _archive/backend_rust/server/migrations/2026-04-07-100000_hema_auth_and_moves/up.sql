-- Up migration (HEMA Auth & Moves)

-- 1. Extend Users table
ALTER TABLE users ADD COLUMN password_hash VARCHAR(255);
ALTER TABLE users ADD COLUMN is_admin BOOLEAN NOT NULL DEFAULT 0;
ALTER TABLE users ADD COLUMN avatar_url VARCHAR;
ALTER TABLE users ADD COLUMN language VARCHAR(10) DEFAULT 'ru';

-- 2. Create Technique Reference
CREATE TABLE hema_moves (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name VARCHAR NOT NULL UNIQUE
);

-- Seed some basic techniques
INSERT INTO hema_moves (name) VALUES 
('Oberhau'), ('Unterhau'), ('Zornhau'), ('Krumphau'), ('Zwerchhau'), 
('Schielhau'), ('Scheitelhau'), ('Ukol (Thrust)'), ('Slice');

-- 3. Extend Bouts with Moves
ALTER TABLE hema_bouts ADD COLUMN move_a_id INTEGER REFERENCES hema_moves (id);
ALTER TABLE hema_bouts ADD COLUMN move_b_id INTEGER REFERENCES hema_moves (id);

-- Update existing bouts if any to default (optional)
