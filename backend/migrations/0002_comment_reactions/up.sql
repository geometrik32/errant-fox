CREATE TABLE comment_reactions (
    comment_id INTEGER NOT NULL REFERENCES comments(id) ON DELETE CASCADE,
    user_id    TEXT    NOT NULL REFERENCES users(id)    ON DELETE CASCADE,
    kind       TEXT    NOT NULL CHECK(kind IN ('like', 'dislike')),
    PRIMARY KEY (comment_id, user_id)
);
