-- Add visibility column to sys_plugins
-- Values: 'private', 'shared', 'global'
ALTER TABLE sys_plugins ADD COLUMN visibility VARCHAR(20) NOT NULL DEFAULT 'private';

-- Index for performance on visibility queries (used in find_available_plugins)
CREATE INDEX idx_plugins_visibility ON sys_plugins(visibility);

COMMENT ON COLUMN sys_plugins.visibility IS 'Visibility scope: private, shared, global';
