# ShareUSTC 数据库设计文档

> 本文档定义 ShareUSTC 项目的数据库设计规范、表结构说明和增量更新策略。

---

## 1. 设计原则

### 1.1 核心设计目标

- **数据完整性**：通过外键约束、唯一约束保证数据一致性
- **可扩展性**：表结构支持动态添加新列而不影响现有数据
- **性能优化**：合理的索引设计支持高效查询
- **可维护性**：清晰的命名规范和文档化

### 1.2 增量更新支持

**核心要求**：数据库初始化脚本必须支持增量更新，即：

1. **首次执行**：创建所有表、列、索引、触发器
2. **重复执行**：检测已存在的结构，自动添加新列，不删除或修改现有数据
3. **版本兼容**：新老数据结构可以共存，应用代码需兼容处理

---

## 2. 增量更新实现方案

### 2.1 技术策略

采用 **"两阶段创建法"** 实现增量更新：

```sql
-- 第一阶段：创建基础表（仅包含主键和创建时间）
CREATE TABLE IF NOT EXISTS example_table (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 第二阶段：使用 DO 语句块逐个添加列（如果不存在）
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'example_table' AND column_name = 'new_column'
    ) THEN
        ALTER TABLE example_table ADD COLUMN new_column VARCHAR(255);
    END IF;
END $$;
```

### 2.2 约束处理方式

| 约束类型 | 处理方式 | 说明 |
|---------|---------|------|
| PRIMARY KEY | 在 CREATE TABLE 中定义 | 建表时即确定，不更改 |
| FOREIGN KEY | 在 ADD COLUMN 时定义 | 新列引用其他表 |
| UNIQUE | 视情况在 ADD COLUMN 或后续添加 | 注意现有数据的唯一性 |
| NOT NULL | 配合 DEFAULT 值使用 | 避免现有数据冲突 |
| CHECK | 在 ADD COLUMN 时定义 | 新增列的校验规则 |

### 2.3 添加新列的规范模板

```sql
-- 模板：为现有表添加新列
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'table_name' AND column_name = 'column_name'
    ) THEN
        ALTER TABLE table_name ADD COLUMN column_name DATA_TYPE [CONSTRAINTS] [DEFAULT value];
    END IF;
END $$;
```

### 2.4 添加唯一约束的注意事项

由于唯一约束可能因现有数据冲突而失败，使用异常处理：

```sql
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'constraint_name' AND conrelid = 'table_name'::regclass
    ) THEN
        ALTER TABLE table_name ADD CONSTRAINT constraint_name UNIQUE (column1, column2);
    END IF;
EXCEPTION
    WHEN unique_violation THEN
        RAISE NOTICE '无法添加唯一约束：存在重复数据';
END $$;
```

---

## 3. 数据库表结构

### 3.1 表清单

| 序号 | 表名 | 说明 | 核心字段数 |
|-----|------|------|-----------|
| 1 | users | 用户表 | 14 |
| 2 | resources | 资源表 | 17 |
| 3 | resource_stats | 资源统计表 | 14 |
| 4 | ratings | 评分表 | 10 |
| 5 | likes | 点赞表 | 3 |
| 6 | comments | 评论表 | 7 |
| 7 | favorites | 收藏夹表 | 4 |
| 8 | favorite_resources | 收藏夹资源关联表 | 3 |
| 9 | claims | 申领表 | 9 |
| 10 | notifications | 通知表 | 9 |
| 11 | notification_reads | 通知已读记录表 | 4 |
| 12 | audit_logs | 审计日志表 | 7 |
| 13 | download_logs | 下载记录表 | 5 |
| 14 | images | 图片表 | 6 |

### 3.2 详细表结构

#### 3.2.1 用户表 (users)

| 字段名 | 数据类型 | 约束 | 默认值 | 说明 |
|--------|---------|------|--------|------|
| id | UUID | PRIMARY KEY | gen_random_uuid() | 主键 |
| sn | BIGINT | UNIQUE | NULL | 用户编号（自增） |
| username | VARCHAR(50) | UNIQUE, NOT NULL | - | 用户名 |
| password_hash | VARCHAR(255) | NOT NULL | - | 密码哈希(BCrypt/Argon2) |
| email | VARCHAR(255) | UNIQUE | NULL | 邮箱 |
| role | VARCHAR(20) | - | 'user' | 角色: guest/user/verified/admin |
| bio | TEXT | - | NULL | 个人简介(Markdown) |
| social_links | JSONB | - | '{}' | 社交链接 |
| real_info | JSONB | - | '{}' | 实名信息 |
| is_verified | BOOLEAN | - | FALSE | 是否实名认证 |
| is_active | BOOLEAN | - | TRUE | 是否启用 |
| avatar_url | VARCHAR(500) | - | NULL | 头像URL |
| created_at | TIMESTAMP | - | CURRENT_TIMESTAMP | 创建时间 |
| updated_at | TIMESTAMP | - | CURRENT_TIMESTAMP | 更新时间 |

**索引**：idx_users_role, idx_users_is_verified, idx_users_sn

#### 3.2.2 资源表 (resources)

| 字段名 | 数据类型 | 约束 | 默认值 | 说明 |
|--------|---------|------|--------|------|
| id | UUID | PRIMARY KEY | gen_random_uuid() | 主键 |
| title | VARCHAR(255) | NOT NULL | - | 资源标题 |
| author_id | UUID | FOREIGN KEY -> users(id) | NULL | 原作者ID |
| uploader_id | UUID | FOREIGN KEY -> users(id), NOT NULL | - | 上传者ID |
| course_name | VARCHAR(255) | - | NULL | 适用课程 |
| resource_type | VARCHAR(50) | - | NULL | 资源类型 |
| category | VARCHAR(50) | - | NULL | 资源分类 |
| tags | JSONB | - | '[]' | 标签数组 |
| file_path | VARCHAR(500) | - | NULL | 文件路径 |
| source_file_path | VARCHAR(500) | - | NULL | 源文件路径 |
| file_hash | VARCHAR(64) | - | NULL | 文件哈希(SHA256) |
| file_size | BIGINT | - | NULL | 文件大小(字节) |
| content_accuracy | FLOAT8 | - | NULL | AI内容准确性评分 |
| audit_status | VARCHAR(20) | - | 'pending' | 审核状态 |
| ai_reject_reason | TEXT | - | NULL | AI拒绝原因 |
| created_at | TIMESTAMP | - | CURRENT_TIMESTAMP | 创建时间 |
| updated_at | TIMESTAMP | - | CURRENT_TIMESTAMP | 更新时间 |

**索引**：idx_resources_uploader, idx_resources_author, idx_resources_course, idx_resources_type, idx_resources_category, idx_resources_audit_status, idx_resources_tags(GIN), idx_resources_created_at

#### 3.2.3 资源统计表 (resource_stats)

| 字段名 | 数据类型 | 约束 | 默认值 | 说明 |
|--------|---------|------|--------|------|
| resource_id | UUID | PRIMARY KEY, FOREIGN KEY -> resources(id) ON DELETE CASCADE | - | 资源ID |
| views | INTEGER | - | 0 | 浏览量 |
| downloads | INTEGER | - | 0 | 下载量 |
| likes | INTEGER | - | 0 | 点赞数 |
| rating_count | INTEGER | - | 0 | 评分人数（冗余字段，用于快速查询） |
| difficulty_total | INTEGER | - | 0 | 难度评分总分 |
| difficulty_count | INTEGER | - | 0 | 难度评分次数 |
| overall_quality_total | INTEGER | - | 0 | 总体质量评分总分 |
| overall_quality_count | INTEGER | - | 0 | 总体质量评分次数 |
| answer_quality_total | INTEGER | - | 0 | 参考答案质量评分总分 |
| answer_quality_count | INTEGER | - | 0 | 参考答案质量评分次数 |
| format_quality_total | INTEGER | - | 0 | 格式质量评分总分 |
| format_quality_count | INTEGER | - | 0 | 格式质量评分次数 |
| detail_level_total | INTEGER | - | 0 | 知识点详细程度评分总分 |
| detail_level_count | INTEGER | - | 0 | 知识点详细程度评分次数 |

**说明**：评分统计采用总分+计数方式存储，便于增量更新和实时计算平均分。

#### 3.2.4 评分表 (ratings)

| 字段名 | 数据类型 | 约束 | 默认值 | 说明 |
|--------|---------|------|--------|------|
| id | UUID | PRIMARY KEY | gen_random_uuid() | 主键 |
| resource_id | UUID | FOREIGN KEY -> resources(id) ON DELETE CASCADE, NOT NULL | - | 资源ID |
| user_id | UUID | FOREIGN KEY -> users(id) ON DELETE CASCADE, NOT NULL | - | 用户ID |
| difficulty | INTEGER | CHECK (1-10) | NULL | 难度评分(1-10) |
| overall_quality | INTEGER | CHECK (1-10) | NULL | 总体质量评分(1-10) |
| answer_quality | INTEGER | CHECK (1-10) | NULL | 参考答案质量(1-10) |
| format_quality | INTEGER | CHECK (1-10) | NULL | 格式质量/排版清晰度(1-10) |
| detail_level | INTEGER | CHECK (1-10) | NULL | 知识点详细程度(1-10) |
| created_at | TIMESTAMP | - | CURRENT_TIMESTAMP | 创建时间 |
| updated_at | TIMESTAMP | - | CURRENT_TIMESTAMP | 更新时间 |

**唯一约束**：UNIQUE(resource_id, user_id)

**说明**：评分系统采用5维度设计，每个维度1-10分。旧版的 `quality` 和 `detail` 字段已弃用，分别由 `overall_quality` 和 `detail_level` 替代。

#### 3.2.5 点赞表 (likes)

| 字段名 | 数据类型 | 约束 | 默认值 | 说明 |
|--------|---------|------|--------|------|
| resource_id | UUID | FOREIGN KEY -> resources(id) ON DELETE CASCADE, PRIMARY KEY | - | 资源ID |
| user_id | UUID | FOREIGN KEY -> users(id) ON DELETE CASCADE, PRIMARY KEY | - | 用户ID |
| created_at | TIMESTAMP | - | CURRENT_TIMESTAMP | 创建时间 |

**主键**：(resource_id, user_id)

#### 3.2.6 评论表 (comments)

| 字段名 | 数据类型 | 约束 | 默认值 | 说明 |
|--------|---------|------|--------|------|
| id | UUID | PRIMARY KEY | gen_random_uuid() | 主键 |
| resource_id | UUID | FOREIGN KEY -> resources(id) ON DELETE CASCADE, NOT NULL | - | 资源ID |
| user_id | UUID | FOREIGN KEY -> users(id) ON DELETE CASCADE, NOT NULL | - | 用户ID |
| content | TEXT | NOT NULL | - | 评论内容 |
| audit_status | VARCHAR(20) | - | 'approved' | 审核状态 |
| created_at | TIMESTAMP | - | CURRENT_TIMESTAMP | 创建时间 |
| updated_at | TIMESTAMP | - | CURRENT_TIMESTAMP | 更新时间 |

#### 3.2.7 收藏夹表 (favorites)

| 字段名 | 数据类型 | 约束 | 默认值 | 说明 |
|--------|---------|------|--------|------|
| id | UUID | PRIMARY KEY | gen_random_uuid() | 主键 |
| user_id | UUID | FOREIGN KEY -> users(id) ON DELETE CASCADE, NOT NULL | - | 用户ID |
| name | VARCHAR(255) | NOT NULL | - | 收藏夹名称 |
| created_at | TIMESTAMP | - | CURRENT_TIMESTAMP | 创建时间 |

#### 3.2.8 收藏夹资源关联表 (favorite_resources)

| 字段名 | 数据类型 | 约束 | 默认值 | 说明 |
|--------|---------|------|--------|------|
| favorite_id | UUID | FOREIGN KEY -> favorites(id) ON DELETE CASCADE, PRIMARY KEY | - | 收藏夹ID |
| resource_id | UUID | FOREIGN KEY -> resources(id) ON DELETE CASCADE, PRIMARY KEY | - | 资源ID |
| added_at | TIMESTAMP | - | CURRENT_TIMESTAMP | 添加时间 |

**主键**：(favorite_id, resource_id)

#### 3.2.9 申领表 (claims)

| 字段名 | 数据类型 | 约束 | 默认值 | 说明 |
|--------|---------|------|--------|------|
| id | UUID | PRIMARY KEY | gen_random_uuid() | 主键 |
| resource_id | UUID | FOREIGN KEY -> resources(id) ON DELETE CASCADE, NOT NULL | - | 资源ID |
| applicant_id | UUID | FOREIGN KEY -> users(id) ON DELETE CASCADE, NOT NULL | - | 申请人ID |
| claim_type | VARCHAR(20) | - | NULL | 申领类型 |
| reason | TEXT | NOT NULL | - | 申领理由 |
| proof_files | JSONB | - | '[]' | 证明文件列表 |
| status | VARCHAR(20) | - | 'pending' | 审核状态 |
| reviewer_id | UUID | FOREIGN KEY -> users(id) | NULL | 审核人ID |
| reviewed_at | TIMESTAMP | - | NULL | 审核时间 |
| created_at | TIMESTAMP | - | CURRENT_TIMESTAMP | 创建时间 |

#### 3.2.10 通知表 (notifications)

| 字段名 | 数据类型 | 约束 | 默认值 | 说明 |
|--------|---------|------|--------|------|
| id | UUID | PRIMARY KEY | gen_random_uuid() | 主键 |
| recipient_id | UUID | FOREIGN KEY -> users(id) ON DELETE CASCADE | NULL | 接收者ID(NULL为群发) |
| title | VARCHAR(255) | NOT NULL | - | 通知标题 |
| content | TEXT | NOT NULL | - | 通知内容 |
| notification_type | VARCHAR(50) | - | NULL | 通知类型 |
| priority | VARCHAR(20) | - | 'normal' | 优先级: high/normal |
| is_read | BOOLEAN | - | FALSE | 是否已读 |
| link_url | VARCHAR(500) | - | NULL | 跳转链接 |
| created_at | TIMESTAMP | - | CURRENT_TIMESTAMP | 创建时间 |

#### 3.2.11 通知已读记录表 (notification_reads)

| 字段名 | 数据类型 | 约束 | 默认值 | 说明 |
|--------|---------|------|--------|------|
| id | UUID | PRIMARY KEY | gen_random_uuid() | 主键 |
| notification_id | UUID | FOREIGN KEY -> notifications(id) ON DELETE CASCADE, NOT NULL | - | 通知ID |
| user_id | UUID | FOREIGN KEY -> users(id) ON DELETE CASCADE, NOT NULL | - | 用户ID |
| read_at | TIMESTAMP | - | CURRENT_TIMESTAMP | 已读时间 |

**唯一约束**：UNIQUE(notification_id, user_id)

#### 3.2.12 审计日志表 (audit_logs)

| 字段名 | 数据类型 | 约束 | 默认值 | 说明 |
|--------|---------|------|--------|------|
| id | UUID | PRIMARY KEY | gen_random_uuid() | 主键 |
| user_id | UUID | FOREIGN KEY -> users(id) | NULL | 操作用户ID |
| action | VARCHAR(100) | NOT NULL | - | 操作类型 |
| target_type | VARCHAR(50) | - | NULL | 目标类型 |
| target_id | UUID | - | NULL | 目标ID |
| details | JSONB | - | '{}' | 操作详情 |
| ip_address | INET | - | NULL | IP地址 |
| created_at | TIMESTAMP | - | CURRENT_TIMESTAMP | 创建时间 |

#### 3.2.13 下载记录表 (download_logs)

| 字段名 | 数据类型 | 约束 | 默认值 | 说明 |
|--------|---------|------|--------|------|
| id | UUID | PRIMARY KEY | gen_random_uuid() | 主键 |
| resource_id | UUID | FOREIGN KEY -> resources(id) ON DELETE CASCADE, NOT NULL | - | 资源ID |
| user_id | UUID | FOREIGN KEY -> users(id) ON DELETE SET NULL | NULL | 用户ID(游客为NULL) |
| ip_address | INET | NOT NULL | - | IP地址 |
| downloaded_at | TIMESTAMP | - | CURRENT_TIMESTAMP | 下载时间 |

#### 3.2.14 图片表 (images)

| 字段名 | 数据类型 | 约束 | 默认值 | 说明 |
|--------|---------|------|--------|------|
| id | UUID | PRIMARY KEY | gen_random_uuid() | 主键 |
| uploader_id | UUID | FOREIGN KEY -> users(id), NOT NULL | - | 上传者ID |
| file_path | VARCHAR(500) | NOT NULL | - | 文件路径 |
| original_name | VARCHAR(255) | - | NULL | 原始文件名 |
| file_size | INTEGER | - | NULL | 文件大小 |
| mime_type | VARCHAR(50) | - | NULL | MIME类型 |
| created_at | TIMESTAMP | - | CURRENT_TIMESTAMP | 创建时间 |

---

## 4. 触发器

### 4.1 自动更新时间触发器

```sql
-- 触发器函数
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- 应用到各表
CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_resources_updated_at
    BEFORE UPDATE ON resources
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_ratings_updated_at
    BEFORE UPDATE ON ratings
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_comments_updated_at
    BEFORE UPDATE ON comments
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
```

**触发器列表**：

| 触发器名 | 表名 | 触发时机 | 说明 |
|---------|------|---------|------|
| update_users_updated_at | users | BEFORE UPDATE | 自动更新 updated_at |
| update_resources_updated_at | resources | BEFORE UPDATE | 自动更新 updated_at |
| update_ratings_updated_at | ratings | BEFORE UPDATE | 自动更新 updated_at |
| update_comments_updated_at | comments | BEFORE UPDATE | 自动更新 updated_at |

---

## 5. 索引设计

### 5.1 索引清单

| 表名 | 索引名 | 字段 | 类型 | 说明 |
|------|--------|------|------|------|
| users | idx_users_role | role | B-tree | 按角色查询 |
| users | idx_users_is_verified | is_verified | B-tree | 按认证状态查询 |
| users | idx_users_sn | sn | B-tree | 按用户编号查询 |
| resources | idx_resources_uploader | uploader_id | B-tree | 按上传者查询 |
| resources | idx_resources_author | author_id | B-tree | 按作者查询 |
| resources | idx_resources_course | course_name | B-tree | 按课程查询 |
| resources | idx_resources_type | resource_type | B-tree | 按类型查询 |
| resources | idx_resources_category | category | B-tree | 按分类查询 |
| resources | idx_resources_audit_status | audit_status | B-tree | 按审核状态查询 |
| resources | idx_resources_tags | tags | GIN | JSONB数组搜索 |
| resources | idx_resources_created_at | created_at DESC | B-tree | 按时间倒序 |
| ratings | idx_ratings_resource | resource_id | B-tree | 按资源查询评分 |
| ratings | idx_ratings_user | user_id | B-tree | 按用户查询评分 |
| likes | idx_likes_user | user_id | B-tree | 按用户查询点赞 |
| comments | idx_comments_resource | resource_id | B-tree | 按资源查询评论 |
| comments | idx_comments_user | user_id | B-tree | 按用户查询评论 |
| comments | idx_comments_created_at | created_at DESC | B-tree | 按时间倒序 |
| favorites | idx_favorites_user | user_id | B-tree | 按用户查询收藏夹 |
| favorite_resources | idx_fav_res_resource | resource_id | B-tree | 按资源查询关联 |
| claims | idx_claims_resource | resource_id | B-tree | 按资源查询申领 |
| claims | idx_claims_applicant | applicant_id | B-tree | 按申请人查询 |
| claims | idx_claims_status | status | B-tree | 按状态查询 |
| notifications | idx_notifications_recipient | recipient_id | B-tree | 按接收者查询 |
| notifications | idx_notifications_priority | priority | B-tree | 按优先级查询 |
| notifications | idx_notifications_is_read | is_read | B-tree | 按已读状态查询 |
| notifications | idx_notifications_created_at | created_at DESC | B-tree | 按时间倒序 |
| notification_reads | idx_notification_reads_notification | notification_id | B-tree | 按通知查询 |
| notification_reads | idx_notification_reads_user | user_id | B-tree | 按用户查询 |
| audit_logs | idx_audit_logs_user | user_id | B-tree | 按用户查询日志 |
| audit_logs | idx_audit_logs_action | action | B-tree | 按操作类型查询 |
| audit_logs | idx_audit_logs_created_at | created_at DESC | B-tree | 按时间倒序 |
| download_logs | idx_download_logs_resource | resource_id | B-tree | 按资源查询 |
| download_logs | idx_download_logs_user | user_id | B-tree | 按用户查询 |
| download_logs | idx_download_logs_time | downloaded_at DESC | B-tree | 按时间倒序 |
| images | idx_images_uploader | uploader_id | B-tree | 按上传者查询 |

---

## 6. 增量更新操作指南

### 6.1 为新表添加增量支持

```sql
-- 1. 创建基础表
CREATE TABLE IF NOT EXISTS new_table (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 2. 添加各列（支持增量）
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'new_table' AND column_name = 'column1') THEN
        ALTER TABLE new_table ADD COLUMN column1 VARCHAR(255);
    END IF;
    -- 添加更多列...
END $$;

-- 3. 创建索引（支持重复执行）
CREATE INDEX IF NOT EXISTS idx_new_table_column1 ON new_table(column1);

-- 4. 创建触发器（支持重复执行）
DROP TRIGGER IF EXISTS update_new_table_updated_at ON new_table;
CREATE TRIGGER update_new_table_updated_at
    BEFORE UPDATE ON new_table
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
```

### 6.2 为现有表添加新列

```sql
-- 在 db_init_tables.sh 中对应的 DO 语句块中添加
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'table_name' AND column_name = 'new_column') THEN
        ALTER TABLE table_name ADD COLUMN new_column DATA_TYPE DEFAULT default_value;
    END IF;
END $$;
```

### 6.3 执行增量更新

```bash
# 进入数据库脚本目录
cd scripts/database

# 执行增量更新脚本（普通用户权限）
./db_init_tables.sh
```

---

## 7. 命名规范

### 7.1 表名命名

- 使用小写字母和下划线（snake_case）
- 使用复数形式：users, resources, comments
- 关联表使用组合名：favorite_resources

### 7.2 列名命名

- 使用小写字母和下划线（snake_case）
- 主键统一使用 `id`
- 外键格式：`表名单数_id`，如 user_id, resource_id
- 时间字段：created_at, updated_at
- 布尔字段：is_xxx 格式，如 is_verified, is_active

### 7.3 索引命名

- 格式：`idx_表名_字段名`
- 多字段索引：`idx_表名_字段1_字段2`
- 示例：idx_users_role, idx_resources_uploader

### 7.4 触发器命名

- 格式：`update_表名_updated_at`
- 示例：update_users_updated_at

---

## 8. 注意事项

### 8.1 NOT NULL 约束

为现有表添加 NOT NULL 列时，必须提供 DEFAULT 值：

```sql
-- 正确：提供默认值
ALTER TABLE users ADD COLUMN new_field VARCHAR(50) NOT NULL DEFAULT '';

-- 错误：没有默认值，会失败（如果表中有数据）
ALTER TABLE users ADD COLUMN new_field VARCHAR(50) NOT NULL;
```

### 8.2 外键约束

添加外键列时，注意 ON DELETE 行为：

- `ON DELETE CASCADE`：级联删除（如 favorite_resources）
- `ON DELETE SET NULL`：设为 NULL（如 download_logs.user_id）
- `ON DELETE RESTRICT`：阻止删除（默认）

### 8.3 唯一约束

添加唯一约束前，需确保现有数据满足唯一性，或使用异常处理：

```sql
DO $$
BEGIN
    ALTER TABLE table_name ADD CONSTRAINT constraint_name UNIQUE (column);
EXCEPTION
    WHEN unique_violation THEN
        RAISE NOTICE '数据已存在重复，跳过添加约束';
END $$;
```

### 8.4 序列管理

用户编号(sn)使用 PostgreSQL 序列管理：

```sql
-- 创建序列
CREATE SEQUENCE IF NOT EXISTS user_sn_seq START 1;

-- 为现有用户分配 sn（按创建时间排序）
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

    IF current_sn > 1 THEN
        PERFORM setval('user_sn_seq', current_sn - 1, true);
    END IF;
END $$;
```

---

## 9. 相关文件

| 文件 | 说明 | 使用场景 |
|------|------|---------|
| `db_create_system.sh` | 系统级初始化（需sudo） | 首次部署，创建数据库和用户 |
| `db_init_tables.sh` | 表结构初始化/增量更新 | 日常更新，添加新表/列 |
| `init.sql` | 完整SQL脚本（手动执行） | 手动维护或调试 |
| `database_design.md` | 本文档 | 设计参考和规范 |

---

*文档版本：1.0*
*更新日期：2026-02-15*
