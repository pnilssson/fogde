CREATE TABLE folders (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    folder TEXT NOT NULL
);

INSERT INTO folders (id, name, folder)
SELECT id, name, folder FROM workspaces WHERE folder IS NOT NULL;

CREATE TABLE placed (
    id INTEGER PRIMARY KEY,
    terminal TEXT NOT NULL UNIQUE,
    workspace INTEGER NOT NULL REFERENCES folders (id) ON DELETE CASCADE
);

INSERT INTO placed (id, terminal, workspace)
SELECT members.id, members.terminal, members.workspace
FROM members JOIN folders ON folders.id = members.workspace;

DROP TABLE members;
DROP TABLE workspaces;

ALTER TABLE folders RENAME TO workspaces;
ALTER TABLE placed RENAME TO members;
