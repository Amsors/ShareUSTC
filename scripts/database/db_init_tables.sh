#!/bin/bash
# ============================================
# ShareUSTC 数据库表结构初始化脚本
# 不需要 sudo，普通用户执行
# 功能: 创建/更新所有表、索引、触发器（支持增量更新）
# ============================================

set -e

# 配置变量
DB_NAME="shareustc"
DB_USER="shareustc_app"
DB_PASSWORD="114514"  # 应与 db_create_system.sh 中一致
DB_HOST="localhost"
DB_PORT="5432"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}=== ShareUSTC 数据库表结构初始化（支持增量更新） ===${NC}"
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

# 创建/更新表的 SQL
echo -e "${YELLOW}开始执行增量更新...${NC}"

PGPASSWORD="${DB_PASSWORD}" psql -h ${DB_HOST} -p ${DB_PORT} -U ${DB_USER} -d ${DB_NAME} << 'EOF'
-- ============================================
-- ShareUSTC 数据库增量更新脚本
-- 支持: 1) 首次创建表  2) 添加新列  3) 创建索引和触发器
-- 特点: 可重复执行，不会丢失已有数据
-- ============================================

-- 启用扩展
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ============================================
-- 创建 sn 序列（从1开始自增）
-- ============================================
CREATE SEQUENCE IF NOT EXISTS user_sn_seq START 1;

-- ============================================
-- 增强：确保序列起始值正确（考虑已有数据）
-- 版本迁移注意：如果数据库已有用户数据，此逻辑会自动调整序列
-- ============================================
DO $$
BEGIN
    PERFORM setval('user_sn_seq',
        (SELECT COALESCE(MAX(sn), 0) + 1 FROM users),
        false);
EXCEPTION
    WHEN undefined_table THEN NULL;
    WHEN undefined_column THEN NULL;
END $$;

-- ============================================
-- 创建表结构（支持增量更新）
-- 策略：先创建表（IF NOT EXISTS），再添加列（IF NOT EXISTS）
-- ============================================

-- ============================================
-- 1. 用户表
-- ============================================
-- 第一步：创建基础表（仅包含必要的约束和默认值）
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 第二步：添加各列（如果不存在）
DO $$
BEGIN
    -- sn: 用户编号，从1开始自增
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'users' AND column_name = 'sn') THEN
        ALTER TABLE users ADD COLUMN sn BIGINT UNIQUE;
    END IF;

    -- username: 用户名
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'users' AND column_name = 'username') THEN
        ALTER TABLE users ADD COLUMN username VARCHAR(50) UNIQUE NOT NULL DEFAULT 'temp_' || gen_random_uuid();
    END IF;

    -- password_hash: 密码哈希
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'users' AND column_name = 'password_hash') THEN
        ALTER TABLE users ADD COLUMN password_hash VARCHAR(255) NOT NULL DEFAULT '';
    END IF;

    -- email: 邮箱
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'users' AND column_name = 'email') THEN
        ALTER TABLE users ADD COLUMN email VARCHAR(255) UNIQUE;
    END IF;

    -- role: 角色 (guest, user, verified, admin)
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'users' AND column_name = 'role') THEN
        ALTER TABLE users ADD COLUMN role VARCHAR(20) DEFAULT 'user';
    END IF;

    -- bio: 个人简介
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'users' AND column_name = 'bio') THEN
        ALTER TABLE users ADD COLUMN bio TEXT;
    END IF;

    -- social_links: 社交链接 (JSONB)
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'users' AND column_name = 'social_links') THEN
        ALTER TABLE users ADD COLUMN social_links JSONB DEFAULT '{}';
    END IF;

    -- real_info: 实名信息 (JSONB)
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'users' AND column_name = 'real_info') THEN
        ALTER TABLE users ADD COLUMN real_info JSONB DEFAULT '{}';
    END IF;

    -- is_verified: 是否实名认证
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'users' AND column_name = 'is_verified') THEN
        ALTER TABLE users ADD COLUMN is_verified BOOLEAN DEFAULT FALSE;
    END IF;

    -- is_active: 是否启用
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'users' AND column_name = 'is_active') THEN
        ALTER TABLE users ADD COLUMN is_active BOOLEAN DEFAULT TRUE;
    END IF;

    -- avatar_url: 头像URL
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'users' AND column_name = 'avatar_url') THEN
        ALTER TABLE users ADD COLUMN avatar_url VARCHAR(500);
    END IF;

    -- updated_at: 更新时间
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'users' AND column_name = 'updated_at') THEN
        ALTER TABLE users ADD COLUMN updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP;
    END IF;
END $$;

-- ============================================
-- 2. 资源表
-- ============================================
CREATE TABLE IF NOT EXISTS resources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'resources' AND column_name = 'title') THEN
        ALTER TABLE resources ADD COLUMN title VARCHAR(255) NOT NULL DEFAULT '';
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'resources' AND column_name = 'author_id') THEN
        ALTER TABLE resources ADD COLUMN author_id UUID REFERENCES users(id);
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'resources' AND column_name = 'uploader_id') THEN
        -- 版本迁移注意：旧数据的外键需要手动处理
        IF EXISTS (SELECT 1 FROM resources LIMIT 1) THEN
            -- 表已有数据，先添加可为NULL的列（需要业务层处理NULL值）
            ALTER TABLE resources ADD COLUMN uploader_id UUID REFERENCES users(id);
        ELSE
            -- 新表：使用 NOT NULL + DEFAULT
            ALTER TABLE resources ADD COLUMN uploader_id UUID NOT NULL REFERENCES users(id) DEFAULT '00000000-0000-0000-0000-000000000000';
        END IF;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'resources' AND column_name = 'course_name') THEN
        ALTER TABLE resources ADD COLUMN course_name VARCHAR(255);
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'resources' AND column_name = 'resource_type') THEN
        ALTER TABLE resources ADD COLUMN resource_type VARCHAR(50);
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'resources' AND column_name = 'category') THEN
        ALTER TABLE resources ADD COLUMN category VARCHAR(50);
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'resources' AND column_name = 'tags') THEN
        ALTER TABLE resources ADD COLUMN tags JSONB DEFAULT '[]';
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'resources' AND column_name = 'file_path') THEN
        ALTER TABLE resources ADD COLUMN file_path VARCHAR(500);
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'resources' AND column_name = 'source_file_path') THEN
        ALTER TABLE resources ADD COLUMN source_file_path VARCHAR(500);
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'resources' AND column_name = 'file_hash') THEN
        ALTER TABLE resources ADD COLUMN file_hash VARCHAR(64);
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'resources' AND column_name = 'file_size') THEN
        ALTER TABLE resources ADD COLUMN file_size BIGINT;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'resources' AND column_name = 'content_accuracy') THEN
        ALTER TABLE resources ADD COLUMN content_accuracy FLOAT8;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'resources' AND column_name = 'audit_status') THEN
        ALTER TABLE resources ADD COLUMN audit_status VARCHAR(20) DEFAULT 'pending';
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'resources' AND column_name = 'ai_reject_reason') THEN
        ALTER TABLE resources ADD COLUMN ai_reject_reason TEXT;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'resources' AND column_name = 'updated_at') THEN
        ALTER TABLE resources ADD COLUMN updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP;
    END IF;
END $$;

-- ============================================
-- 3. 资源统计表
-- ============================================
CREATE TABLE IF NOT EXISTS resource_stats (
    resource_id UUID PRIMARY KEY REFERENCES resources(id) ON DELETE CASCADE
);

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'resource_stats' AND column_name = 'views') THEN
        ALTER TABLE resource_stats ADD COLUMN views INTEGER DEFAULT 0;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'resource_stats' AND column_name = 'downloads') THEN
        ALTER TABLE resource_stats ADD COLUMN downloads INTEGER DEFAULT 0;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'resource_stats' AND column_name = 'likes') THEN
        ALTER TABLE resource_stats ADD COLUMN likes INTEGER DEFAULT 0;
    END IF;

    -- 评分人数（冗余字段，用于快速查询）
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'resource_stats' AND column_name = 'rating_count') THEN
        ALTER TABLE resource_stats ADD COLUMN rating_count INTEGER DEFAULT 0;
    END IF;

    -- 评分统计：每个维度独立记录总分和评分次数
    -- 难度维度
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'resource_stats' AND column_name = 'difficulty_total') THEN
        ALTER TABLE resource_stats ADD COLUMN difficulty_total INTEGER DEFAULT 0;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'resource_stats' AND column_name = 'difficulty_count') THEN
        ALTER TABLE resource_stats ADD COLUMN difficulty_count INTEGER DEFAULT 0;
    END IF;

    -- 总体质量维度
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'resource_stats' AND column_name = 'overall_quality_total') THEN
        ALTER TABLE resource_stats ADD COLUMN overall_quality_total INTEGER DEFAULT 0;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'resource_stats' AND column_name = 'overall_quality_count') THEN
        ALTER TABLE resource_stats ADD COLUMN overall_quality_count INTEGER DEFAULT 0;
    END IF;

    -- 参考答案质量维度
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'resource_stats' AND column_name = 'answer_quality_total') THEN
        ALTER TABLE resource_stats ADD COLUMN answer_quality_total INTEGER DEFAULT 0;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'resource_stats' AND column_name = 'answer_quality_count') THEN
        ALTER TABLE resource_stats ADD COLUMN answer_quality_count INTEGER DEFAULT 0;
    END IF;

    -- 格式质量维度
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'resource_stats' AND column_name = 'format_quality_total') THEN
        ALTER TABLE resource_stats ADD COLUMN format_quality_total INTEGER DEFAULT 0;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'resource_stats' AND column_name = 'format_quality_count') THEN
        ALTER TABLE resource_stats ADD COLUMN format_quality_count INTEGER DEFAULT 0;
    END IF;

    -- 知识点详细程度维度
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'resource_stats' AND column_name = 'detail_level_total') THEN
        ALTER TABLE resource_stats ADD COLUMN detail_level_total INTEGER DEFAULT 0;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'resource_stats' AND column_name = 'detail_level_count') THEN
        ALTER TABLE resource_stats ADD COLUMN detail_level_count INTEGER DEFAULT 0;
    END IF;
END $$;

-- ============================================
-- 4. 评分表
-- ============================================
CREATE TABLE IF NOT EXISTS ratings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'ratings' AND column_name = 'resource_id') THEN
        -- 版本迁移注意：旧数据的外键需要手动处理
        IF EXISTS (SELECT 1 FROM ratings LIMIT 1) THEN
            ALTER TABLE ratings ADD COLUMN resource_id UUID REFERENCES resources(id) ON DELETE CASCADE;
        ELSE
            ALTER TABLE ratings ADD COLUMN resource_id UUID NOT NULL REFERENCES resources(id) ON DELETE CASCADE DEFAULT '00000000-0000-0000-0000-000000000000';
        END IF;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'ratings' AND column_name = 'user_id') THEN
        -- 版本迁移注意：旧数据的外键需要手动处理
        IF EXISTS (SELECT 1 FROM ratings LIMIT 1) THEN
            ALTER TABLE ratings ADD COLUMN user_id UUID REFERENCES users(id) ON DELETE CASCADE;
        ELSE
            ALTER TABLE ratings ADD COLUMN user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE DEFAULT '00000000-0000-0000-0000-000000000000';
        END IF;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'ratings' AND column_name = 'difficulty') THEN
        ALTER TABLE ratings ADD COLUMN difficulty INTEGER CHECK (difficulty BETWEEN 1 AND 10);
    END IF;

    -- 注意：quality 和 detail 字段已移除，使用 overall_quality 和 detail_level 替代

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'ratings' AND column_name = 'overall_quality') THEN
        ALTER TABLE ratings ADD COLUMN overall_quality INTEGER CHECK (overall_quality BETWEEN 1 AND 10);
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'ratings' AND column_name = 'answer_quality') THEN
        ALTER TABLE ratings ADD COLUMN answer_quality INTEGER CHECK (answer_quality BETWEEN 1 AND 10);
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'ratings' AND column_name = 'format_quality') THEN
        ALTER TABLE ratings ADD COLUMN format_quality INTEGER CHECK (format_quality BETWEEN 1 AND 10);
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'ratings' AND column_name = 'detail_level') THEN
        ALTER TABLE ratings ADD COLUMN detail_level INTEGER CHECK (detail_level BETWEEN 1 AND 10);
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'ratings' AND column_name = 'updated_at') THEN
        ALTER TABLE ratings ADD COLUMN updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP;
    END IF;
END $$;

-- 添加唯一约束（如果存在重复数据，需要先清理）
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'ratings_resource_id_user_id_key' AND conrelid = 'ratings'::regclass
    ) THEN
        ALTER TABLE ratings ADD CONSTRAINT ratings_resource_id_user_id_key UNIQUE (resource_id, user_id);
    END IF;
EXCEPTION
    WHEN unique_violation THEN
        RAISE NOTICE '无法添加唯一约束：存在重复数据 (resource_id, user_id)';
END $$;

-- ============================================
-- 5. 点赞表
-- ============================================
CREATE TABLE IF NOT EXISTS likes (
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'likes' AND column_name = 'resource_id') THEN
        ALTER TABLE likes ADD COLUMN resource_id UUID REFERENCES resources(id) ON DELETE CASCADE;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'likes' AND column_name = 'user_id') THEN
        ALTER TABLE likes ADD COLUMN user_id UUID REFERENCES users(id) ON DELETE CASCADE;
    END IF;
END $$;

-- 添加主键约束
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'likes_pkey' AND conrelid = 'likes'::regclass
    ) THEN
        ALTER TABLE likes ADD PRIMARY KEY (resource_id, user_id);
    END IF;
EXCEPTION
    WHEN unique_violation THEN
        RAISE NOTICE '无法添加主键约束：存在重复数据';
END $$;

-- ============================================
-- 6. 评论表
-- ============================================
CREATE TABLE IF NOT EXISTS comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'comments' AND column_name = 'resource_id') THEN
        -- 版本迁移注意：旧数据的外键需要手动处理
        IF EXISTS (SELECT 1 FROM comments LIMIT 1) THEN
            ALTER TABLE comments ADD COLUMN resource_id UUID REFERENCES resources(id) ON DELETE CASCADE;
        ELSE
            ALTER TABLE comments ADD COLUMN resource_id UUID NOT NULL REFERENCES resources(id) ON DELETE CASCADE DEFAULT '00000000-0000-0000-0000-000000000000';
        END IF;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'comments' AND column_name = 'user_id') THEN
        -- 版本迁移注意：旧数据的外键需要手动处理
        IF EXISTS (SELECT 1 FROM comments LIMIT 1) THEN
            ALTER TABLE comments ADD COLUMN user_id UUID REFERENCES users(id) ON DELETE CASCADE;
        ELSE
            ALTER TABLE comments ADD COLUMN user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE DEFAULT '00000000-0000-0000-0000-000000000000';
        END IF;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'comments' AND column_name = 'content') THEN
        ALTER TABLE comments ADD COLUMN content TEXT NOT NULL DEFAULT '';
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'comments' AND column_name = 'audit_status') THEN
        ALTER TABLE comments ADD COLUMN audit_status VARCHAR(20) DEFAULT 'approved';
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'comments' AND column_name = 'updated_at') THEN
        ALTER TABLE comments ADD COLUMN updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP;
    END IF;
END $$;

-- ============================================
-- 7. 收藏夹表
-- ============================================
CREATE TABLE IF NOT EXISTS favorites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'favorites' AND column_name = 'user_id') THEN
        -- 版本迁移注意：旧数据的外键需要手动处理
        IF EXISTS (SELECT 1 FROM favorites LIMIT 1) THEN
            ALTER TABLE favorites ADD COLUMN user_id UUID REFERENCES users(id) ON DELETE CASCADE;
        ELSE
            ALTER TABLE favorites ADD COLUMN user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE DEFAULT '00000000-0000-0000-0000-000000000000';
        END IF;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'favorites' AND column_name = 'name') THEN
        ALTER TABLE favorites ADD COLUMN name VARCHAR(255) NOT NULL DEFAULT '未命名收藏夹';
    END IF;
END $$;

-- ============================================
-- 8. 收藏夹资源关联表
-- ============================================
CREATE TABLE IF NOT EXISTS favorite_resources (
    added_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'favorite_resources' AND column_name = 'favorite_id') THEN
        ALTER TABLE favorite_resources ADD COLUMN favorite_id UUID REFERENCES favorites(id) ON DELETE CASCADE;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'favorite_resources' AND column_name = 'resource_id') THEN
        ALTER TABLE favorite_resources ADD COLUMN resource_id UUID REFERENCES resources(id) ON DELETE CASCADE;
    END IF;
END $$;

-- 添加主键约束
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'favorite_resources_pkey' AND conrelid = 'favorite_resources'::regclass
    ) THEN
        ALTER TABLE favorite_resources ADD PRIMARY KEY (favorite_id, resource_id);
    END IF;
EXCEPTION
    WHEN unique_violation THEN
        RAISE NOTICE '无法添加主键约束：存在重复数据';
END $$;

-- ============================================
-- 9. 申领表
-- ============================================
CREATE TABLE IF NOT EXISTS claims (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'claims' AND column_name = 'resource_id') THEN
        -- 版本迁移注意：旧数据的外键需要手动处理
        IF EXISTS (SELECT 1 FROM claims LIMIT 1) THEN
            ALTER TABLE claims ADD COLUMN resource_id UUID REFERENCES resources(id) ON DELETE CASCADE;
        ELSE
            ALTER TABLE claims ADD COLUMN resource_id UUID NOT NULL REFERENCES resources(id) ON DELETE CASCADE DEFAULT '00000000-0000-0000-0000-000000000000';
        END IF;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'claims' AND column_name = 'applicant_id') THEN
        -- 版本迁移注意：旧数据的外键需要手动处理
        IF EXISTS (SELECT 1 FROM claims LIMIT 1) THEN
            ALTER TABLE claims ADD COLUMN applicant_id UUID REFERENCES users(id) ON DELETE CASCADE;
        ELSE
            ALTER TABLE claims ADD COLUMN applicant_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE DEFAULT '00000000-0000-0000-0000-000000000000';
        END IF;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'claims' AND column_name = 'claim_type') THEN
        ALTER TABLE claims ADD COLUMN claim_type VARCHAR(20);
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'claims' AND column_name = 'reason') THEN
        ALTER TABLE claims ADD COLUMN reason TEXT NOT NULL DEFAULT '';
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'claims' AND column_name = 'proof_files') THEN
        ALTER TABLE claims ADD COLUMN proof_files JSONB DEFAULT '[]';
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'claims' AND column_name = 'status') THEN
        ALTER TABLE claims ADD COLUMN status VARCHAR(20) DEFAULT 'pending';
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'claims' AND column_name = 'reviewer_id') THEN
        ALTER TABLE claims ADD COLUMN reviewer_id UUID REFERENCES users(id);
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'claims' AND column_name = 'reviewed_at') THEN
        ALTER TABLE claims ADD COLUMN reviewed_at TIMESTAMP;
    END IF;
END $$;

-- ============================================
-- 10. 通知表
-- ============================================
CREATE TABLE IF NOT EXISTS notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'notifications' AND column_name = 'recipient_id') THEN
        ALTER TABLE notifications ADD COLUMN recipient_id UUID REFERENCES users(id) ON DELETE CASCADE;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'notifications' AND column_name = 'title') THEN
        ALTER TABLE notifications ADD COLUMN title VARCHAR(255) NOT NULL DEFAULT '';
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'notifications' AND column_name = 'content') THEN
        ALTER TABLE notifications ADD COLUMN content TEXT NOT NULL DEFAULT '';
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'notifications' AND column_name = 'notification_type') THEN
        ALTER TABLE notifications ADD COLUMN notification_type VARCHAR(50);
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'notifications' AND column_name = 'priority') THEN
        ALTER TABLE notifications ADD COLUMN priority VARCHAR(20) DEFAULT 'normal';
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'notifications' AND column_name = 'is_read') THEN
        ALTER TABLE notifications ADD COLUMN is_read BOOLEAN DEFAULT FALSE;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'notifications' AND column_name = 'link_url') THEN
        ALTER TABLE notifications ADD COLUMN link_url VARCHAR(500);
    END IF;
END $$;

-- ============================================
-- 10b. 通知已读记录表（用于群发通知的独立已读状态）
-- ============================================
CREATE TABLE IF NOT EXISTS notification_reads (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    read_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'notification_reads' AND column_name = 'notification_id') THEN
        -- 版本迁移注意：旧数据的外键需要手动处理
        IF EXISTS (SELECT 1 FROM notification_reads LIMIT 1) THEN
            ALTER TABLE notification_reads ADD COLUMN notification_id UUID REFERENCES notifications(id) ON DELETE CASCADE;
        ELSE
            ALTER TABLE notification_reads ADD COLUMN notification_id UUID NOT NULL REFERENCES notifications(id) ON DELETE CASCADE DEFAULT '00000000-0000-0000-0000-000000000000';
        END IF;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'notification_reads' AND column_name = 'user_id') THEN
        -- 版本迁移注意：旧数据的外键需要手动处理
        IF EXISTS (SELECT 1 FROM notification_reads LIMIT 1) THEN
            ALTER TABLE notification_reads ADD COLUMN user_id UUID REFERENCES users(id) ON DELETE CASCADE;
        ELSE
            ALTER TABLE notification_reads ADD COLUMN user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE DEFAULT '00000000-0000-0000-0000-000000000000';
        END IF;
    END IF;
END $$;

-- 添加唯一约束
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'notification_reads_notification_id_user_id_key' AND conrelid = 'notification_reads'::regclass
    ) THEN
        ALTER TABLE notification_reads ADD CONSTRAINT notification_reads_notification_id_user_id_key UNIQUE (notification_id, user_id);
    END IF;
EXCEPTION
    WHEN unique_violation THEN
        RAISE NOTICE '无法添加唯一约束：存在重复数据';
END $$;

-- ============================================
-- 11. 审计日志表
-- ============================================
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'audit_logs' AND column_name = 'user_id') THEN
        ALTER TABLE audit_logs ADD COLUMN user_id UUID REFERENCES users(id);
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'audit_logs' AND column_name = 'action') THEN
        ALTER TABLE audit_logs ADD COLUMN action VARCHAR(100) NOT NULL DEFAULT '';
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'audit_logs' AND column_name = 'target_type') THEN
        ALTER TABLE audit_logs ADD COLUMN target_type VARCHAR(50);
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'audit_logs' AND column_name = 'target_id') THEN
        ALTER TABLE audit_logs ADD COLUMN target_id UUID;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'audit_logs' AND column_name = 'details') THEN
        ALTER TABLE audit_logs ADD COLUMN details JSONB DEFAULT '{}';
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'audit_logs' AND column_name = 'ip_address') THEN
        ALTER TABLE audit_logs ADD COLUMN ip_address INET;
    END IF;
END $$;

-- ============================================
-- 12. 下载记录表
-- ============================================
CREATE TABLE IF NOT EXISTS download_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    downloaded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'download_logs' AND column_name = 'resource_id') THEN
        -- 版本迁移注意：旧数据的外键需要手动处理
        IF EXISTS (SELECT 1 FROM download_logs LIMIT 1) THEN
            ALTER TABLE download_logs ADD COLUMN resource_id UUID REFERENCES resources(id) ON DELETE CASCADE;
        ELSE
            ALTER TABLE download_logs ADD COLUMN resource_id UUID NOT NULL REFERENCES resources(id) ON DELETE CASCADE DEFAULT '00000000-0000-0000-0000-000000000000';
        END IF;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'download_logs' AND column_name = 'user_id') THEN
        ALTER TABLE download_logs ADD COLUMN user_id UUID REFERENCES users(id) ON DELETE SET NULL;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'download_logs' AND column_name = 'ip_address') THEN
        ALTER TABLE download_logs ADD COLUMN ip_address INET NOT NULL DEFAULT '0.0.0.0';
    END IF;
END $$;

-- ============================================
-- 13. 图片表
-- ============================================
CREATE TABLE IF NOT EXISTS images (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'images' AND column_name = 'uploader_id') THEN
        -- 版本迁移注意：旧数据的外键需要手动处理
        IF EXISTS (SELECT 1 FROM images LIMIT 1) THEN
            ALTER TABLE images ADD COLUMN uploader_id UUID REFERENCES users(id);
        ELSE
            ALTER TABLE images ADD COLUMN uploader_id UUID NOT NULL REFERENCES users(id) DEFAULT '00000000-0000-0000-0000-000000000000';
        END IF;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'images' AND column_name = 'file_path') THEN
        ALTER TABLE images ADD COLUMN file_path VARCHAR(500) NOT NULL DEFAULT '';
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'images' AND column_name = 'original_name') THEN
        ALTER TABLE images ADD COLUMN original_name VARCHAR(255);
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'images' AND column_name = 'file_size') THEN
        ALTER TABLE images ADD COLUMN file_size INTEGER;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'images' AND column_name = 'mime_type') THEN
        ALTER TABLE images ADD COLUMN mime_type VARCHAR(50);
    END IF;
END $$;

-- ============================================
-- 为现有用户分配 sn（增量更新支持）
-- ============================================
DO $$
DECLARE
    user_record RECORD;
    current_sn BIGINT := 1;
BEGIN
    FOR user_record IN
        SELECT id FROM users WHERE sn IS NULL ORDER BY created_at ASC
    LOOP
        UPDATE users SET sn = current_sn WHERE id = user_record.id;
        current_sn := current_sn + 1;
    END LOOP;

    -- 如果分配了 sn，更新序列的下一个值
    IF current_sn > 1 THEN
        PERFORM setval('user_sn_seq', current_sn - 1, true);
    END IF;
END $$;

-- ============================================
-- 创建索引
-- ============================================

-- 用户表索引
CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);
CREATE INDEX IF NOT EXISTS idx_users_is_verified ON users(is_verified);
CREATE INDEX IF NOT EXISTS idx_users_sn ON users(sn);

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

DROP TRIGGER IF EXISTS update_ratings_updated_at ON ratings;
CREATE TRIGGER update_ratings_updated_at
    BEFORE UPDATE ON ratings
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_comments_updated_at ON comments;
CREATE TRIGGER update_comments_updated_at
    BEFORE UPDATE ON comments
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
SELECT 'notification_reads', COUNT(*) FROM information_schema.columns WHERE table_name = 'notification_reads'
UNION ALL
SELECT 'audit_logs', COUNT(*) FROM information_schema.columns WHERE table_name = 'audit_logs'
UNION ALL
SELECT 'download_logs', COUNT(*) FROM information_schema.columns WHERE table_name = 'download_logs'
UNION ALL
SELECT 'images', COUNT(*) FROM information_schema.columns WHERE table_name = 'images';
EOF

echo ""
echo -e "${GREEN}=== 表结构增量更新完成 ===${NC}"
echo ""
echo "已创建/更新的表:"
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
echo "  - notification_reads (通知已读记录表)"
echo "  - audit_logs (审计日志表)"
echo "  - download_logs (下载记录表)"
echo "  - images (图片表)"
echo ""
echo "创建的索引: 30+ 个"
echo "创建的触发器: 4 个 (自动更新 updated_at)"
echo ""
echo -e "${YELLOW}说明:${NC}"
echo "  此脚本支持增量更新，可重复执行。"
echo "  新增列时会自动添加，不会影响已有数据。"
