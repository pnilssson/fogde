ALTER TABLE workspaces DROP COLUMN name;
ALTER TABLE workspaces ADD COLUMN identity TEXT;
ALTER TABLE members ADD COLUMN checkout TEXT;
