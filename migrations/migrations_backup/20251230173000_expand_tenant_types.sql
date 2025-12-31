-- ================================================
-- Migration: 20251230173000_expand_tenant_types.sql
-- Purpose: Add more descriptive tenant types for business scaling
-- ================================================

INSERT INTO sys_tenant_types (slug, name, description, icon, is_system) VALUES 
    ('partner', 'Partner / พันธมิตร', 'Strategic partner or platform associate', 'handshake', TRUE),
    ('franchise', 'Franchise / แฟรนไชส์', 'Franchise location or branch', 'award', TRUE),
    ('service_provider', 'Service Provider / ผู้ให้บริการ', 'Third-party service provider', 'briefcase', TRUE)
ON CONFLICT (slug) DO UPDATE SET 
    name = EXCLUDED.name,
    description = EXCLUDED.description,
    icon = EXCLUDED.icon;

-- Ensure translations/friendly names for existing types
UPDATE sys_tenant_types SET slug = 'owner', name = 'System Owner / เจ้าของระบบ' WHERE slug = 'standard';
UPDATE sys_tenant_types SET name = 'Branch / สาขา' WHERE slug = 'branch';
UPDATE sys_tenant_types SET name = 'Vendor / ผู้ขาย' WHERE slug = 'vendor';
UPDATE sys_tenant_types SET name = 'Supplier / ผู้จัดจำหน่าย' WHERE slug = 'supplier';
