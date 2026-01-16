-- ════════════════════════════════════════════════════════════════════════════
-- Migration: 0005_cms_media.sql
-- Purpose: CMS, Media Library, Social Infrastructure
-- Consolidated from: 0005, 0007
-- ════════════════════════════════════════════════════════════════════════════

-- ════════════════════════════════════════════════════════════════════════════
-- 1. CMS PAGES
-- ════════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS sys_pages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES auth_tenants(id) ON DELETE CASCADE,
    slug VARCHAR(255) NOT NULL,
    title VARCHAR(255) NOT NULL,
    content JSONB DEFAULT '[]'::jsonb,
    is_published BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    UNIQUE(tenant_id, slug)
);

CREATE INDEX idx_sys_pages_tenant_slug ON sys_pages(tenant_id, slug);
CREATE TRIGGER update_sys_pages_updated_at BEFORE UPDATE ON sys_pages FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ════════════════════════════════════════════════════════════════════════════
-- 2. MEDIA LIBRARY
-- ════════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS media_folders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES auth_tenants(id) ON DELETE CASCADE,
    parent_id UUID REFERENCES media_folders(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS media_assets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES auth_tenants(id) ON DELETE CASCADE,
    folder_id UUID REFERENCES media_folders(id) ON DELETE SET NULL,
    filename VARCHAR(255) NOT NULL,
    original_name VARCHAR(255) NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    file_size BIGINT NOT NULL,
    url TEXT NOT NULL,
    visibility VARCHAR(20) NOT NULL DEFAULT 'private',
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_media_folders_tenant ON media_folders(tenant_id);
CREATE INDEX idx_media_assets_tenant ON media_assets(tenant_id);
CREATE TRIGGER update_media_folders_updated_at BEFORE UPDATE ON media_folders FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_media_assets_updated_at BEFORE UPDATE ON media_assets FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ════════════════════════════════════════════════════════════════════════════
-- 3. LANDING PAGE LOGIC
-- ════════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS sys_tenant_type_role_defaults (
    tenant_type_id UUID NOT NULL REFERENCES sys_tenant_types(id) ON DELETE CASCADE,
    role_slug VARCHAR(50) NOT NULL,
    default_landing_path VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (tenant_type_id, role_slug)
);

-- ════════════════════════════════════════════════════════════════════════════
-- 4. SOCIAL POSTS
-- ════════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS social_posts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES auth_tenants(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
    parent_id UUID REFERENCES social_posts(id) ON DELETE CASCADE,
    
    post_type VARCHAR(20) NOT NULL DEFAULT 'post',
    original_post_id UUID REFERENCES social_posts(id) ON DELETE SET NULL,
    
    caption TEXT,
    is_published BOOLEAN DEFAULT TRUE,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_social_posts_user ON social_posts(user_id);
CREATE INDEX idx_social_posts_tenant ON social_posts(tenant_id);
CREATE INDEX idx_social_posts_parent ON social_posts(parent_id);
CREATE TRIGGER update_social_posts_updated_at BEFORE UPDATE ON social_posts FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ════════════════════════════════════════════════════════════════════════════
-- 5. SOCIAL MEDIA (Carousel)
-- ════════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS social_media (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    post_id UUID NOT NULL REFERENCES social_posts(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    media_type VARCHAR(20) NOT NULL DEFAULT 'image',
    sort_order INT DEFAULT 0,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_social_media_post ON social_media(post_id);
CREATE INDEX idx_social_media_user ON social_media(user_id);

-- ════════════════════════════════════════════════════════════════════════════
-- 6. SOCIAL INTERACTIONS
-- ════════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS social_interactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    post_id UUID NOT NULL REFERENCES social_posts(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
    interaction_type VARCHAR(20) NOT NULL,
    interaction_value VARCHAR(50),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(post_id, user_id, interaction_type)
);

CREATE INDEX idx_social_interactions_post ON social_interactions(post_id);
CREATE INDEX idx_social_interactions_user ON social_interactions(user_id);

-- ════════════════════════════════════════════════════════════════════════════
-- 7. SOCIAL FOLLOWS
-- ════════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS social_follows (
    follower_id UUID NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
    following_id UUID NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (follower_id, following_id)
);

CREATE INDEX idx_social_follows_following ON social_follows(following_id);
