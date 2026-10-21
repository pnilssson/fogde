CREATE TABLE machines (
    id INTEGER PRIMARY KEY,
    host TEXT UNIQUE,
    target TEXT UNIQUE,
    triple TEXT,
    connected INTEGER NOT NULL DEFAULT 1,
    CHECK ((host IS NULL) = (target IS NULL) AND (host IS NULL) = (triple IS NULL))
);

CREATE UNIQUE INDEX one_computer ON machines ((host IS NULL)) WHERE host IS NULL;
