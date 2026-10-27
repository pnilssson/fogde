CREATE TABLE tiles (
    id INTEGER PRIMARY KEY,
    tile TEXT NOT NULL UNIQUE,
    kind TEXT NOT NULL CHECK (kind IN ('terminal', 'note')),
    note TEXT CHECK (kind = 'note' OR note IS NULL),
    workspace INTEGER NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    checkout TEXT,
    position INTEGER NOT NULL DEFAULT 0
);

INSERT INTO tiles (id, tile, kind, workspace, checkout, position)
SELECT id, terminal, 'terminal', workspace, checkout, position FROM members;

DROP TABLE members;
