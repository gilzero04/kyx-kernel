-- ================================================
-- Migration: 0007_social_infrastructure.sql
-- Purpose: Support for Social Plugins (Instagram/Twitter-like).
-- Includes: Posts, Media Carousels, Interactions, and Follows.
-- ================================================

-- 1. Social Posts (Threads & Reposts support)
CREATE TABLE IF NOT EXISTS social_posts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES auth_tenants(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
    parent_id UUID REFERENCES social_posts(id) ON DELETE CASCADE, -- For Threads/Replies
    
    post_type VARCHAR(20) NOT NULL DEFAULT 'post', -- 'post', 'repost', 'quote'
    original_post_id UUID REFERENCES social_posts(id) ON DELETE SET NULL, -- For Reposts/Quotes
    
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

-- 2. Social Media (Carousel support)
CREATE TABLE IF NOT EXISTS social_media (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    post_id UUID NOT NULL REFERENCES social_posts(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
    
    url TEXT NOT NULL,
    media_type VARCHAR(20) NOT NULL DEFAULT 'image', -- 'image', 'video'
    sort_order INT DEFAULT 0,
    
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_social_media_post ON social_media(post_id);
CREATE INDEX idx_social_media_user ON social_media(user_id);

-- 3. Social Interactions (Likes, Bookmarks, Reactions)
CREATE TABLE IF NOT EXISTS social_interactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    post_id UUID NOT NULL REFERENCES social_posts(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
    
    interaction_type VARCHAR(20) NOT NULL, -- 'like', 'bookmark', 'emoji'
    interaction_value VARCHAR(50), -- e.g., '🔥', '❤️'
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(post_id, user_id, interaction_type)
);

CREATE INDEX idx_social_interactions_post ON social_interactions(post_id);
CREATE INDEX idx_social_interactions_user ON social_interactions(user_id);

-- 4. Social Follows (Following System)
CREATE TABLE IF NOT EXISTS social_follows (
    follower_id UUID NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
    following_id UUID NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (follower_id, following_id)
);

CREATE INDEX idx_social_follows_following ON social_follows(following_id);
