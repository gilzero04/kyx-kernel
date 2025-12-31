-- ================================================
-- Migration: 00000000000000_init_functions.sql
-- Purpose: Create shared database functions used by all tables
-- ================================================

-- Timestamp auto-update trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';
