-- Add Default i18n Keys for Sidebar
-- This script adds all the translation keys currently used in the application

-- Sidebar Keys (core.sidebar.*)
INSERT INTO sys_i18n_translations (locale, key, message, is_auto_generated) VALUES
-- English
('en', 'core.sidebar.home', 'Home', false),
('en', 'core.sidebar.theme_tester', 'Theme Tester', false),
('en', 'core.sidebar.dashboard', 'Dashboard', false),
('en', 'core.sidebar.branding', 'Branding', false),
('en', 'core.sidebar.settings', 'System Settings', false),
('en', 'core.sidebar.translations', 'Translations', false),
('en', 'core.sidebar.logs', 'Audit Logs', false),
('en', 'core.sidebar.tenants', 'Tenants', false),
('en', 'core.sidebar.users', 'Users', false),
('en', 'core.sidebar.permissions', 'Permissions', false),
('en', 'core.sidebar.auth', 'Authentication', false),
('en', 'core.sidebar.signin', 'Sign In', false),
('en', 'core.sidebar.setup', 'Setup', false),

-- Thai
('th', 'core.sidebar.home', 'หน้าแรก', false),
('th', 'core.sidebar.theme_tester', 'ทดสอบธีม', false),
('th', 'core.sidebar.dashboard', 'แดชบอร์ด', false),
('th', 'core.sidebar.branding', 'ตั้งค่าแบรนด์', false),
('th', 'core.sidebar.settings', 'ตั้งค่าระบบ', false),
('th', 'core.sidebar.translations', 'การแปลภาษา', false),
('th', 'core.sidebar.logs', 'บันทึกการใช้งาน', false),
('th', 'core.sidebar.tenants', 'องค์กร', false),
('th', 'core.sidebar.users', 'ผู้ใช้งาน', false),
('th', 'core.sidebar.permissions', 'สิทธิ์การใช้งาน', false),
('th', 'core.sidebar.auth', 'การยืนยันตัวตน', false),
('th', 'core.sidebar.signin', 'เข้าสู่ระบบ', false),
('th', 'core.sidebar.setup', 'ตั้งค่าเริ่มต้น', false)

ON CONFLICT (locale, key) DO NOTHING;
