-- Add Sidebar CORS i18n Key
INSERT INTO sys_i18n_translations (locale, key, message, is_auto_generated) VALUES
('en', 'core.sidebar.cors', 'CORS Origins', false),
('th', 'core.sidebar.cors', 'กฎ CORS', false)
ON CONFLICT (locale, key) DO NOTHING;
