CREATE TABLE tiles_next (
    id INTEGER PRIMARY KEY,
    tile TEXT NOT NULL UNIQUE,
    kind TEXT NOT NULL CHECK (kind IN ('terminal', 'note', 'monitor')),
    note TEXT CHECK (kind = 'note' OR note IS NULL),
    workspace INTEGER NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    checkout TEXT,
    position INTEGER NOT NULL DEFAULT 0
);

INSERT INTO tiles_next (id, tile, kind, note, workspace, checkout, position)
SELECT id, tile, kind, note, workspace, checkout, position FROM tiles;

DROP TABLE tiles;

ALTER TABLE tiles_next RENAME TO tiles;
