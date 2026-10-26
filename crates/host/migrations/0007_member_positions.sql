ALTER TABLE members ADD COLUMN position INTEGER NOT NULL DEFAULT 0;
UPDATE members SET position = id;
