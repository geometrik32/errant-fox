CREATE TABLE techniques_new (
    id   INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT UNIQUE NOT NULL
);
INSERT INTO techniques_new SELECT id, name FROM techniques;
DROP TABLE techniques;
ALTER TABLE techniques_new RENAME TO techniques;
