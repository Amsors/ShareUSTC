#!/bin/bash
# ============================================
# ShareUSTC 数据库表结构初始化脚本
# 不需要 sudo，普通用户执行
#
# 表结构已统一由 sqlx 迁移管理（backend/migrations/），本脚本调用
# `sqlx migrate run` 应用迁移，不再内嵌建表 SQL。
# 前置：数据库与用户已创建（见 db_create_system.sh 或 docs/deploy_guide.md）。
#
# 说明：后端进程启动时也会自动执行迁移（见 backend/src/main.rs），
# 因此本脚本主要用于「未启动后端就先把表建好」的场景（如 CI、批量导入前置）。
#
# 适用范围：仅裸机部署。容器部署时迁移由后端启动自动执行，无需运行本脚本
# （见 docs/deploy_guide.md「容器部署」）。
# ============================================

set -e

# 配置变量（应与 db_create_system.sh 保持一致）
DB_NAME="shareustc"
DB_USER="shareustc_app"
DB_PASSWORD="ShareUSTC_default_pwd"
DB_HOST="localhost"
DB_PORT="5432"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# 定位 backend 目录（迁移文件所在）
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKEND_DIR="$(cd "${SCRIPT_DIR}/../../backend" && pwd)"
MIGRATIONS_DIR="${BACKEND_DIR}/migrations"

DATABASE_URL="postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}"

echo -e "${GREEN}=== ShareUSTC 数据库表结构初始化（sqlx 迁移）===${NC}"
echo ""

# 检查 psql 是否可用
if ! command -v psql &> /dev/null; then
    echo -e "${RED}错误: 未找到 psql 命令，请安装 PostgreSQL 客户端${NC}"
    exit 1
fi

# 测试数据库连接
echo -e "${YELLOW}测试数据库连接...${NC}"
if ! PGPASSWORD="${DB_PASSWORD}" psql -h "${DB_HOST}" -p "${DB_PORT}" -U "${DB_USER}" -d "${DB_NAME}" -c "SELECT 1;" > /dev/null 2>&1; then
    echo -e "${RED}错误: 无法连接到数据库，请检查:${NC}"
    echo "  1. 数据库是否已创建 (运行 db_create_system.sh)"
    echo "  2. 用户名、密码是否正确"
    echo "  3. PostgreSQL 服务是否运行"
    exit 1
fi
echo -e "${GREEN}  数据库连接成功${NC}"
echo ""

# 检查 sqlx-cli
if ! command -v sqlx &> /dev/null; then
    echo -e "${RED}错误: 未找到 sqlx-cli${NC}"
    echo "  安装: cargo install sqlx-cli --no-default-features --features native-tls,postgres"
    echo "  或直接启动后端 (cd backend && cargo run)，后端会在启动时自动应用迁移。"
    exit 1
fi

# 存量库保护：已有业务表但无 sqlx 迁移记录时，先标记基线再运行迁移，
# 否则 sqlx migrate run 会在已存在的表上重复建表而失败。
HAS_USERS=$(PGPASSWORD="${DB_PASSWORD}" psql -h "${DB_HOST}" -p "${DB_PORT}" -U "${DB_USER}" -d "${DB_NAME}" -tAc "SELECT to_regclass('public.users') IS NOT NULL;")
HAS_MIG=$(PGPASSWORD="${DB_PASSWORD}" psql -h "${DB_HOST}" -p "${DB_PORT}" -U "${DB_USER}" -d "${DB_NAME}" -tAc "SELECT to_regclass('public._sqlx_migrations') IS NOT NULL;")
if [ "${HAS_USERS}" = "t" ] && [ "${HAS_MIG}" != "t" ]; then
    echo -e "${YELLOW}检测到存量库（已有 users 表但无迁移记录）。${NC}"
    echo "  这是从旧初始化脚本迁移而来的数据库，请先标记基线（不重复建表）："
    echo -e "    ${GREEN}cd \"${BACKEND_DIR}\" && DATABASE_URL=\"${DATABASE_URL}\" sqlx migrate resolve --version 1${NC}"
    echo "  标记后再次运行本脚本即可应用后续迁移。"
    exit 1
fi

# 应用迁移
echo -e "${YELLOW}应用数据库迁移...${NC}"
DATABASE_URL="${DATABASE_URL}" sqlx migrate run --source "${MIGRATIONS_DIR}"

echo ""
echo -e "${GREEN}=== 表结构初始化完成 ===${NC}"
echo "已应用 backend/migrations/ 下的全部迁移。"
