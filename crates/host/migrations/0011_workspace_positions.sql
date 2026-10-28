ALTER TABLE workspaces ADD COLUMN position INTEGER NOT NULL DEFAULT 0;

UPDATE workspaces SET position = id;
