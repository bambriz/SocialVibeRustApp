-- Social Pulse Database Schema
-- Optimized PostgreSQL schema for users, posts, and hierarchical comments

-- Enable UUID extension for better performance and security
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users table with optimized indexing
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,  -- Argon2 hash
    display_name VARCHAR(100),
    bio TEXT,
    avatar_url VARCHAR(500),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_active BOOLEAN DEFAULT TRUE,
    -- Indexing for fast lookups
    CONSTRAINT users_username_length CHECK (length(username) >= 3),
    CONSTRAINT users_email_valid CHECK (email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$')
);

-- Posts table with sentiment and moderation data
CREATE TABLE IF NOT EXISTS posts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    title VARCHAR(300),
    -- Sentiment analysis results (JSON for flexibility)
    sentiment_analysis JSONB,
    -- Content moderation results  
    moderation_result JSONB,
    is_flagged BOOLEAN DEFAULT FALSE,
    -- Performance and audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    view_count INTEGER DEFAULT 0,
    -- Content validation
    CONSTRAINT posts_content_length CHECK (length(content) >= 1 AND length(content) <= 5000),
    CONSTRAINT posts_title_length CHECK (title IS NULL OR length(title) <= 300)
);

-- Hierarchical comments table using Materialized Path pattern for Reddit-like threading
CREATE TABLE IF NOT EXISTS comments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    post_id UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    parent_id UUID REFERENCES comments(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    -- Materialized path for efficient hierarchy queries (e.g., "1/3/7/")
    path TEXT NOT NULL,
    -- Depth level for UI rendering (0 = root comment, 1 = reply, 2 = reply to reply, etc.)
    depth INTEGER NOT NULL DEFAULT 0,
    -- Sentiment and moderation
    sentiment_analysis JSONB,
    moderation_result JSONB,
    is_flagged BOOLEAN DEFAULT FALSE,
    -- Performance fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reply_count INTEGER DEFAULT 0,
    -- Validation constraints
    CONSTRAINT comments_content_length CHECK (length(content) >= 1 AND length(content) <= 2000),
    CONSTRAINT comments_depth_limit CHECK (depth >= 0 AND depth <= 10),
    CONSTRAINT comments_path_valid CHECK (path ~ '^([0-9]+/)+$')
);

-- Indexes for optimal performance

-- Users
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_created_at ON users(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_users_active ON users(is_active) WHERE is_active = TRUE;

-- Posts  
CREATE INDEX IF NOT EXISTS idx_posts_user_id ON posts(user_id);
CREATE INDEX IF NOT EXISTS idx_posts_created_at ON posts(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_posts_flagged ON posts(is_flagged) WHERE is_flagged = FALSE;
CREATE INDEX IF NOT EXISTS idx_posts_sentiment ON posts USING gin(sentiment_analysis);

-- Comments (optimized for hierarchical queries)
CREATE INDEX IF NOT EXISTS idx_comments_post_id ON comments(post_id);
CREATE INDEX IF NOT EXISTS idx_comments_user_id ON comments(user_id);  
CREATE INDEX IF NOT EXISTS idx_comments_parent_id ON comments(parent_id);
CREATE INDEX IF NOT EXISTS idx_comments_path ON comments(path);
CREATE INDEX IF NOT EXISTS idx_comments_path_post ON comments(post_id, path);
CREATE INDEX IF NOT EXISTS idx_comments_depth ON comments(depth);
CREATE INDEX IF NOT EXISTS idx_comments_created_at ON comments(created_at DESC);

-- Triggers for maintaining updated_at timestamps
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_posts_updated_at BEFORE UPDATE ON posts  
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_comments_updated_at BEFORE UPDATE ON comments
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Function to update comment reply counts
CREATE OR REPLACE FUNCTION update_comment_reply_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        -- Increment reply count for parent comment
        IF NEW.parent_id IS NOT NULL THEN
            UPDATE comments 
            SET reply_count = reply_count + 1 
            WHERE id = NEW.parent_id;
        END IF;
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        -- Decrement reply count for parent comment
        IF OLD.parent_id IS NOT NULL THEN
            UPDATE comments 
            SET reply_count = GREATEST(0, reply_count - 1) 
            WHERE id = OLD.parent_id;
        END IF;
        RETURN OLD;
    END IF;
    RETURN NULL;
END;
$$ language 'plpgsql';

CREATE TRIGGER comment_reply_count_trigger
    AFTER INSERT OR DELETE ON comments
    FOR EACH ROW EXECUTE FUNCTION update_comment_reply_count();

-- Sample data for development (optional)
-- INSERT INTO users (username, email, password_hash, display_name) VALUES
-- ('demo_user', 'demo@socialpulse.dev', '$argon2id$v=19$m=65536,t=3,p=4$dummy_hash', 'Demo User');