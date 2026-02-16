#!/bin/bash
# ============================================
# ShareUSTC 数据库系统级初始化脚本
# 需要 sudo 权限执行
# 功能: 创建数据库和用户
# ============================================

set -e

# 配置变量
DB_NAME="shareustc"
DB_USER="shareustc_app"
DB_PASSWORD="114514"  # 生产环境请修改此密码
POSTGRES_USER="postgres"  # PostgreSQL 超级用户

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== ShareUSTC 数据库系统级初始化 ===${NC}"
echo ""

# 检查是否以 root 或 postgres 用户运行
if [ "$EUID" -ne 0 ] && [ "$(whoami)" != "postgres" ]; then
    echo -e "${RED}错误: 请以 root 用户或使用 sudo 运行此脚本${NC}"
    echo "例如: sudo ./db_create_system.sh"
    exit 1
fi

echo -e "${YELLOW}步骤 1/4: 检查 PostgreSQL 服务状态...${NC}"
if systemctl is-active --quiet postgresql; then
    echo -e "${GREEN}  PostgreSQL 服务正在运行${NC}"
else
    echo -e "${YELLOW}  启动 PostgreSQL 服务...${NC}"
    systemctl start postgresql
    systemctl enable postgresql
    echo -e "${GREEN}  PostgreSQL 服务已启动${NC}"
fi

echo ""
echo -e "${YELLOW}步骤 2/4: 创建数据库用户 '${DB_USER}'...${NC}"
# 检查用户是否已存在
USER_EXISTS=$(sudo -u postgres psql -t -c "SELECT 1 FROM pg_roles WHERE rolname='${DB_USER}'" 2>/dev/null || echo "")
if [ -n "$USER_EXISTS" ]; then
    echo -e "${YELLOW}  用户 '${DB_USER}' 已存在，跳过创建${NC}"
else
    sudo -u postgres psql -c "CREATE USER ${DB_USER} WITH PASSWORD '${DB_PASSWORD}';"
    echo -e "${GREEN}  用户 '${DB_USER}' 创建成功${NC}"
fi

echo ""
echo -e "${YELLOW}步骤 3/4: 创建数据库 '${DB_NAME}'...${NC}"
# 检查数据库是否已存在
DB_EXISTS=$(sudo -u postgres psql -t -c "SELECT 1 FROM pg_database WHERE datname='${DB_NAME}'" 2>/dev/null || echo "")
if [ -n "$DB_EXISTS" ]; then
    echo -e "${YELLOW}  数据库 '${DB_NAME}' 已存在，跳过创建${NC}"
else
    sudo -u postgres psql -c "CREATE DATABASE ${DB_NAME} OWNER ${DB_USER} ENCODING 'UTF8' LC_COLLATE 'C.UTF-8' LC_CTYPE 'C.UTF-8' TEMPLATE template0;"
    echo -e "${GREEN}  数据库 '${DB_NAME}' 创建成功${NC}"
fi

echo ""
echo -e "${YELLOW}步骤 4/4: 授予权限...${NC}"
# 授予数据库连接权限
sudo -u postgres psql -c "GRANT CONNECT ON DATABASE ${DB_NAME} TO ${DB_USER};"

# 在数据库内授予 schema 权限
sudo -u postgres psql -d ${DB_NAME} -c "GRANT USAGE ON SCHEMA public TO ${DB_USER};"
sudo -u postgres psql -d ${DB_NAME} -c "GRANT CREATE ON SCHEMA public TO ${DB_USER};"

# 启用 pgcrypto 扩展
sudo -u postgres psql -d ${DB_NAME} -c "CREATE EXTENSION IF NOT EXISTS pgcrypto;"

echo -e "${GREEN}  权限授予完成${NC}"

echo ""
echo -e "${GREEN}=== 系统级初始化完成 ===${NC}"
echo ""
echo "数据库信息:"
echo "  数据库名: ${DB_NAME}"
echo "  用户名:   ${DB_USER}"
echo "  密码:     ${DB_PASSWORD}"
echo ""
echo -e "${YELLOW}提示: 请修改脚本中的 DB_PASSWORD 变量或使用更安全的密码生成方式${NC}"
echo ""
echo "下一步: 执行数据库表结构初始化"
echo "  ./db_init_tables.sh"
