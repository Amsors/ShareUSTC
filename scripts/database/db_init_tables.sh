#!/bin/bash
# ============================================
# ShareUSTC 数据库表结构初始化脚本
# 不需要 sudo，普通用户执行
# 功能: 创建所有表、索引、触发器
# ============================================

set -e

# 配置变量
DB_NAME="shareustc"
DB_USER="shareustc_app"
DB_PASSWORD="ShareUSTC_default_pwd"  # 应与 db_create_system.sh 中一致
DB_HOST="localhost"
DB_PORT="5432"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}=== ShareUSTC 数据库表结构初始化 ===${NC}"
echo ""

# 检查 psql 是否可用
if ! command -v psql &> /dev/null; then
    echo -e "${RED}错误: 未找到 psql 命令，请安装 PostgreSQL 客户端${NC}"
    exit 1
fi

# 测试数据库连接
echo -e "${YELLOW}测试数据库连接...${NC}"
if ! PGPASSWORD="${DB_PASSWORD}" psql -h ${DB_HOST} -p ${DB_PORT} -U ${DB_USER} -d ${DB_NAME} -c "SELECT 1;" > /dev/null 2>&1; then
    echo -e "${RED}错误: 无法连接到数据库，请检查:${NC}"
    echo "  1. 数据库是否已创建 (运行 db_create_system.sh)"
    echo "  2. 用户名、密码是否正确"
    echo "  3. PostgreSQL 服务是否运行"
    exit 1
fi
echo -e "${GREEN}  数据库连接成功${NC}"
echo ""

# 创建表的 SQL
echo -e "${YELLOW}开始创建表结构...${NC}"

PGPASSWORD="${DB_PASSWORD}" psql -h ${DB_HOST} -p ${DB_PORT} -U ${DB_USER} -d ${DB_NAME} << 'EOF'
-- ============================================
-- ShareUSTC 数据库表结构初始化
-- ============================================

-- 启用扩展
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ============================================
-- 1. 用户表
-- ============================================
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

-- ============================================
-- 2. 资源表
-- ============================================
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

-- ============================================
-- 3. 资源统计表
-- ============================================
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

-- ============================================
-- 4. 评分表
-- ============================================
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

-- ============================================
-- 5. 点赞表
-- ============================================
CREATE TABLE IF NOT EXISTS likes (
    resource_id UUID REFERENCES resources(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (resource_id, user_id)
);

-- ============================================
-- 6. 评论表
-- ============================================
CREATE TABLE IF NOT EXISTS comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resource_id UUID NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    audit_status VARCHAR(20) DEFAULT 'approved',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ============================================
-- 7. 收藏夹表
-- ============================================
CREATE TABLE IF NOT EXISTS favorites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ============================================
-- 8. 收藏夹资源关联表
-- ============================================
CREATE TABLE IF NOT EXISTS favorite_resources (
    favorite_id UUID REFERENCES favorites(id) ON DELETE CASCADE,
    resource_id UUID REFERENCES resources(id) ON DELETE CASCADE,
    added_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (favorite_id, resource_id)
);

-- ============================================
-- 9. 申领表
-- ============================================
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

-- ============================================
-- 10. 通知表
-- ============================================
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

-- ============================================
-- 10b. 通知已读记录表（用于群发通知的独立已读状态）
-- ============================================
CREATE TABLE IF NOT EXISTS notification_reads (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    notification_id UUID NOT NULL REFERENCES notifications(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    read_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(notification_id, user_id)
);

-- ============================================
-- 11. 审计日志表
-- ============================================
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

-- ============================================
-- 12. 下载记录表
-- ============================================
CREATE TABLE IF NOT EXISTS download_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    resource_id UUID NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    ip_address INET NOT NULL,
    downloaded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ============================================
-- 13. 图片表
-- ============================================
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
-- 创建更新触发器 (自动更新 updated_at)
-- ============================================

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- 为用户表创建触发器
DROP TRIGGER IF EXISTS update_users_updated_at ON users;
CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- 为资源表创建触发器
DROP TRIGGER IF EXISTS update_resources_updated_at ON resources;
CREATE TRIGGER update_resources_updated_at
    BEFORE UPDATE ON resources
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- 验证创建结果
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
EOF

echo ""
echo -e "${GREEN}=== 表结构初始化完成 ===${NC}"
echo ""
echo "创建的表:"
echo "  - users (用户表)"
echo "  - resources (资源表)"
echo "  - resource_stats (资源统计表)"
echo "  - ratings (评分表)"
echo "  - likes (点赞表)"
echo "  - comments (评论表)"
echo "  - favorites (收藏夹表)"
echo "  - favorite_resources (收藏夹资源关联表)"
echo "  - claims (申领表)"
echo "  - notifications (通知表)"
echo "  - audit_logs (审计日志表)"
echo "  - download_logs (下载记录表)"
echo "  - images (图片表)"
echo ""
echo "创建的索引: 30+ 个"
echo "创建的触发器: 2 个 (自动更新 updated_at)"
