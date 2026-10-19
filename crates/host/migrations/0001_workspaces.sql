CREATE TABLE workspaces (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE members (
    id INTEGER PRIMARY KEY,
    terminal TEXT NOT NULL UNIQUE,
    workspace INTEGER NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE
);
