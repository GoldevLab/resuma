CREATE TABLE IF NOT EXISTS todos (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    owner_id TEXT NOT NULL,
    title    TEXT NOT NULL,
    done     INTEGER NOT NULL DEFAULT 0 CHECK (done IN (0, 1))
);

CREATE INDEX IF NOT EXISTS idx_todos_owner ON todos(owner_id);
