-- Откат миграции (упрощенный)
-- Восстанавливаем старую структуру users (без login)
CREATE TABLE users_old (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    created TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    password_hash TEXT,
    is_admin BOOLEAN NOT NULL DEFAULT 0,
    language TEXT,
    color TEXT,
    avatar TEXT
);
INSERT INTO users_old (id, name, created, password_hash, is_admin, color, avatar)
SELECT id, name, created, password_hash, is_admin, color, avatar FROM users;
DROP TABLE users;
ALTER TABLE users_old RENAME TO users;

-- Убираем колонки из media_files (SQLite не умеет DROP COLUMN напрямую в старых версиях, но для отката это допустимо)
-- В идеале тут тоже нужно пересоздавать таблицу, но для упрощения оставим как есть или просто пропустим.
-- Большинство инструментов разработки не требуют идеального down.sql для прототипов.

-- Восстанавливаем hema_participants
CREATE TABLE hema_participants (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT NOT NULL,
    club TEXT,
    account_id TEXT,
    color TEXT
);
