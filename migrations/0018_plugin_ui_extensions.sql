-- Add UI extensions column to sys_plugins
ALTER TABLE sys_plugins ADD COLUMN ui JSONB DEFAULT NULL;
