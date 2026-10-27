CREATE TABLE tiles_next (
    id INTEGER PRIMARY KEY,
    tile TEXT NOT NULL UNIQUE,
    workspace INTEGER NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    checkout TEXT,
    position INTEGER NOT NULL DEFAULT 0
);

INSERT INTO tiles_next (id, tile, workspace, checkout, position)
SELECT id, tile, workspace, checkout, position FROM tiles WHERE kind = 'terminal';

DROP TABLE tiles;

ALTER TABLE tiles_next RENAME TO tiles;
