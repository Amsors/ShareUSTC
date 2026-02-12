-- ============================================
-- ShareUSTC 数据库完整初始化 SQL
-- 手动执行时使用
-- ============================================

-- 连接到 PostgreSQL
-- psql -U postgres

-- 1. 创建用户 (修改密码!)
CREATE USER shareustc_app WITH PASSWORD 'shareustc_app_password';

-- 2. 创建数据库
CREATE DATABASE shareustc
    OWNER shareustc_app
    ENCODING 'UTF8'
    LC_COLLATE 'en_US.UTF-8'
    LC_CTYPE 'en_US.UTF-8';

-- 3. 连接到新数据库
\c shareustc

-- 4. 启用扩展
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ============================================
-- 创建表结构
-- ============================================

-- 用户表
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE,
    role VARCHAR(20) DEFAULT 'user',
    bio TEXT,
    social_links JSONB DEFAULT '{}',
    real_info JSONB DEFAULT '{}',
    is_verified BOOLEAN DEFAULT FALSE,
    is_active BOOLEAN DEFAULT TRUE,
    avatar_url VARCHAR(500),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 资源表
CREATE TABLE IF NOT EXISTS resources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    author_id UUID REFERENCES users(id),
    uploader_id UUID NOT NULL REFERENCES users(id),
    course_name VARCHAR(255),
    resource_type VARCHAR(50),
    category VARCHAR(50),
    tags JSONB DEFAULT '[]',
    file_path VARCHAR(500),
    source_file_path VARCHAR(500),
    file_hash VARCHAR(64),
    file_size BIGINT,
    content_accuracy FLOAT8,
    audit_status VARCHAR(20) DEFAULT 'pending',
    ai_reject_reason TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 资源统计表
CREATE TABLE IF NOT EXISTS resource_stats (
    resource_id UUID PRIMARY KEY REFERENCES resources(id) ON DELETE CASCADE,
    views INTEGER DEFAULT 0,
    downloads INTEGER DEFAULT 0,
    likes INTEGER DEFAULT 0,
    avg_difficulty FLOAT8,
    avg_quality FLOAT8,
    avg_detail FLOAT8,
    rating_count INTEGER DEFAULT 0
);

-- 评分表
CREATE TABLE IF NOT EXISTS ratings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resource_id UUID NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    difficulty INTEGER CHECK (difficulty BETWEEN 1 AND 10),
    quality INTEGER CHECK (quality BETWEEN 1 AND 10),
    detail INTEGER CHECK (detail BETWEEN 1 AND 10),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(resource_id, user_id)
);

-- 点赞表
CREATE TABLE IF NOT EXISTS likes (
    resource_id UUID REFERENCES resources(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (resource_id, user_id)
);

-- 评论表
CREATE TABLE IF NOT EXISTS comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resource_id UUID NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    audit_status VARCHAR(20) DEFAULT 'approved',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 收藏夹表
CREATE TABLE IF NOT EXISTS favorites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 收藏夹资源关联表
CREATE TABLE IF NOT EXISTS favorite_resources (
    favorite_id UUID REFERENCES favorites(id) ON DELETE CASCADE,
    resource_id UUID REFERENCES resources(id) ON DELETE CASCADE,
    added_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (favorite_id, resource_id)
);

-- 申领表
CREATE TABLE IF NOT EXISTS claims (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resource_id UUID NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    applicant_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    claim_type VARCHAR(20),
    reason TEXT NOT NULL,
    proof_files JSONB DEFAULT '[]',
    status VARCHAR(20) DEFAULT 'pending',
    reviewer_id UUID REFERENCES users(id),
    reviewed_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 通知表
CREATE TABLE IF NOT EXISTS notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recipient_id UUID REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    notification_type VARCHAR(50),
    priority VARCHAR(20) DEFAULT 'normal',
    is_read BOOLEAN DEFAULT FALSE,
    link_url VARCHAR(500),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 通知已读记录表（用于群发通知的独立已读状态）
CREATE TABLE IF NOT EXISTS notification_reads (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    notification_id UUID NOT NULL REFERENCES notifications(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    read_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(notification_id, user_id)
);

-- 审计日志表
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    action VARCHAR(100) NOT NULL,
    target_type VARCHAR(50),
    target_id UUID,
    details JSONB DEFAULT '{}',
    ip_address INET,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 下载记录表
CREATE TABLE IF NOT EXISTS download_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resource_id UUID NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    ip_address INET NOT NULL,
    downloaded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 图片表
CREATE TABLE IF NOT EXISTS images (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    uploader_id UUID NOT NULL REFERENCES users(id),
    file_path VARCHAR(500) NOT NULL,
    original_name VARCHAR(255),
    file_size INTEGER,
    mime_type VARCHAR(50),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ============================================
-- 创建索引
-- ============================================

-- 用户表索引
CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);
CREATE INDEX IF NOT EXISTS idx_users_is_verified ON users(is_verified);

-- 资源表索引
CREATE INDEX IF NOT EXISTS idx_resources_uploader ON resources(uploader_id);
CREATE INDEX IF NOT EXISTS idx_resources_author ON resources(author_id);
CREATE INDEX IF NOT EXISTS idx_resources_course ON resources(course_name);
CREATE INDEX IF NOT EXISTS idx_resources_type ON resources(resource_type);
CREATE INDEX IF NOT EXISTS idx_resources_category ON resources(category);
CREATE INDEX IF NOT EXISTS idx_resources_audit_status ON resources(audit_status);
CREATE INDEX IF NOT EXISTS idx_resources_tags ON resources USING GIN(tags);
CREATE INDEX IF NOT EXISTS idx_resources_created_at ON resources(created_at DESC);

-- 评分表索引
CREATE INDEX IF NOT EXISTS idx_ratings_resource ON ratings(resource_id);
CREATE INDEX IF NOT EXISTS idx_ratings_user ON ratings(user_id);

-- 点赞表索引
CREATE INDEX IF NOT EXISTS idx_likes_user ON likes(user_id);

-- 评论表索引
CREATE INDEX IF NOT EXISTS idx_comments_resource ON comments(resource_id);
CREATE INDEX IF NOT EXISTS idx_comments_user ON comments(user_id);
CREATE INDEX IF NOT EXISTS idx_comments_created_at ON comments(created_at DESC);

-- 收藏夹索引
CREATE INDEX IF NOT EXISTS idx_favorites_user ON favorites(user_id);
CREATE INDEX IF NOT EXISTS idx_fav_res_resource ON favorite_resources(resource_id);

-- 申领表索引
CREATE INDEX IF NOT EXISTS idx_claims_resource ON claims(resource_id);
CREATE INDEX IF NOT EXISTS idx_claims_applicant ON claims(applicant_id);
CREATE INDEX IF NOT EXISTS idx_claims_status ON claims(status);

-- 通知表索引
CREATE INDEX IF NOT EXISTS idx_notifications_recipient ON notifications(recipient_id);
CREATE INDEX IF NOT EXISTS idx_notifications_priority ON notifications(priority);
CREATE INDEX IF NOT EXISTS idx_notifications_is_read ON notifications(is_read);
CREATE INDEX IF NOT EXISTS idx_notifications_created_at ON notifications(created_at DESC);

-- 通知已读记录表索引
CREATE INDEX IF NOT EXISTS idx_notification_reads_notification ON notification_reads(notification_id);
CREATE INDEX IF NOT EXISTS idx_notification_reads_user ON notification_reads(user_id);
CREATE INDEX IF NOT EXISTS idx_notification_reads_unique ON notification_reads(notification_id, user_id);

-- 审计日志索引
CREATE INDEX IF NOT EXISTS idx_audit_logs_user ON audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_action ON audit_logs(action);
CREATE INDEX IF NOT EXISTS idx_audit_logs_created_at ON audit_logs(created_at DESC);

-- 下载记录索引
CREATE INDEX IF NOT EXISTS idx_download_logs_resource ON download_logs(resource_id);
CREATE INDEX IF NOT EXISTS idx_download_logs_user ON download_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_download_logs_time ON download_logs(downloaded_at DESC);

-- 图片表索引
CREATE INDEX IF NOT EXISTS idx_images_uploader ON images(uploader_id);

-- ============================================
-- 创建触发器
-- ============================================

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

DROP TRIGGER IF EXISTS update_users_updated_at ON users;
CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_resources_updated_at ON resources;
CREATE TRIGGER update_resources_updated_at
    BEFORE UPDATE ON resources
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================
-- 验证
-- ============================================

SELECT 'users' as table_name, COUNT(*) as column_count FROM information_schema.columns WHERE table_name = 'users'
UNION ALL
SELECT 'resources', COUNT(*) FROM information_schema.columns WHERE table_name = 'resources'
UNION ALL
SELECT 'resource_stats', COUNT(*) FROM information_schema.columns WHERE table_name = 'resource_stats'
UNION ALL
SELECT 'ratings', COUNT(*) FROM information_schema.columns WHERE table_name = 'ratings'
UNION ALL
SELECT 'likes', COUNT(*) FROM information_schema.columns WHERE table_name = 'likes'
UNION ALL
SELECT 'comments', COUNT(*) FROM information_schema.columns WHERE table_name = 'comments'
UNION ALL
SELECT 'favorites', COUNT(*) FROM information_schema.columns WHERE table_name = 'favorites'
UNION ALL
SELECT 'favorite_resources', COUNT(*) FROM information_schema.columns WHERE table_name = 'favorite_resources'
UNION ALL
SELECT 'claims', COUNT(*) FROM information_schema.columns WHERE table_name = 'claims'
UNION ALL
SELECT 'notifications', COUNT(*) FROM information_schema.columns WHERE table_name = 'notifications'
UNION ALL
SELECT 'audit_logs', COUNT(*) FROM information_schema.columns WHERE table_name = 'audit_logs'
UNION ALL
SELECT 'download_logs', COUNT(*) FROM information_schema.columns WHERE table_name = 'download_logs'
UNION ALL
SELECT 'images', COUNT(*) FROM information_schema.columns WHERE table_name = 'images';
