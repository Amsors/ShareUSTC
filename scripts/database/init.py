#!/usr/bin/env python3
"""
ShareUSTC 数据库初始化脚本
跨平台: 支持 Windows, Linux, macOS
功能: 创建数据库/用户，并通过 sqlx 迁移创建表结构

表结构已统一由 sqlx 迁移管理（backend/migrations/），本脚本的建表步骤
调用 `sqlx migrate run`，不再内嵌建表 SQL。后端进程启动时也会自动执行迁移。
"""

import subprocess
import sys
import os
import shutil
import tempfile
import platform

# ============================================
# 配置变量
# ============================================
DB_NAME = "shareustc"
DB_USER = "shareustc_app"
DB_PASSWORD = "ShareUSTC_default_pwd"  # 生产环境请修改此密码
DB_HOST = "localhost"
DB_PORT = "5432"
POSTGRES_USER = "postgres"  # PostgreSQL 超级用户
# PostgreSQL 'postgres' 用户的密码（修改以下值后再执行脚本）
POSTGRES_PASSWORD = "postgres"

# ============================================
# 数据库创建部分的 SQL
# ============================================
DB_CREATION_SQL = '''
-- 检查并创建用户
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = '{db_user}') THEN
        CREATE USER {db_user} WITH PASSWORD '{db_password}';
        RAISE NOTICE '用户 {db_user} 创建成功';
    ELSE
        RAISE NOTICE '用户 {db_user} 已存在，跳过创建';
    END IF;
END $$;

-- 检查并创建数据库
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_database WHERE datname = '{db_name}') THEN
        CREATE DATABASE {db_name} OWNER {db_user} ENCODING 'UTF8' LC_COLLATE '{locale}' LC_CTYPE '{locale}' TEMPLATE template0;
        RAISE NOTICE '数据库 {db_name} 创建成功';
    ELSE
        RAISE NOTICE '数据库 {db_name} 已存在，跳过创建';
    END IF;
END $$;
'''

# ============================================
# 权限授予部分的 SQL
# ============================================
DB_PERMISSION_SQL = '''
-- 授予数据库连接权限
GRANT CONNECT ON DATABASE {db_name} TO {db_user};

-- 在数据库内授予 schema 权限
GRANT USAGE ON SCHEMA public TO {db_user};
GRANT CREATE ON SCHEMA public TO {db_user};

-- 启用 pgcrypto 扩展
CREATE EXTENSION IF NOT EXISTS pgcrypto;

SELECT '权限授予完成' as status;
'''


def find_psql():
    """查找 psql 可执行文件"""
    system = platform.system()

    # 首先检查 PATH
    try:
        if system == "Windows":
            result = subprocess.run(['where', 'psql'], capture_output=True, text=True)
        else:
            result = subprocess.run(['which', 'psql'], capture_output=True, text=True)
        if result.returncode == 0:
            return 'psql'
    except Exception:
        pass

    # 检查常见 PostgreSQL 安装路径
    if system == "Windows":
        common_paths = [
            r"C:\Program Files\PostgreSQL",
            r"C:\Program Files (x86)\PostgreSQL"
        ]
        for base_path in common_paths:
            if os.path.exists(base_path):
                for version in os.listdir(base_path):
                    psql_path = os.path.join(base_path, version, 'bin', 'psql.exe')
                    if os.path.exists(psql_path):
                        return psql_path
    else:
        common_paths = [
            "/usr/bin/psql",
            "/usr/local/bin/psql",
            "/opt/homebrew/bin/psql",
            "/Applications/Postgres.app/Contents/Versions/latest/bin/psql"
        ]
        for path in common_paths:
            if os.path.exists(path):
                return path

    return None


def execute_sql_with_superuser(psql, sql, password, database="postgres"):
    """使用超级用户执行 SQL"""
    env = os.environ.copy()
    env['PGPASSWORD'] = password

    with tempfile.NamedTemporaryFile(mode='w', suffix='.sql', delete=False, encoding='utf-8') as f:
        f.write(sql)
        temp_file = f.name

    try:
        result = subprocess.run(
            [psql, '-h', DB_HOST, '-p', DB_PORT, '-U', POSTGRES_USER, '-d', database, '-f', temp_file, '-q'],
            capture_output=True,
            text=True,
            env=env
        )
        return result
    finally:
        os.unlink(temp_file)


def create_database(psql, postgres_password):
    """步骤1: 创建数据库和用户"""
    print("=== 步骤 1/3: 创建数据库和用户 ===")
    print()

    # 检测操作系统以设置正确的 locale
    system = platform.system()
    locale = "C" if system == "Windows" else "C.UTF-8"

    sql = DB_CREATION_SQL.format(
        db_name=DB_NAME,
        db_user=DB_USER,
        db_password=DB_PASSWORD,
        locale=locale
    )

    result = execute_sql_with_superuser(psql, sql, postgres_password)

    if result.returncode == 0:
        print(f"  数据库 '{DB_NAME}' 和用户 '{DB_USER}' 检查/创建完成")
        return True
    else:
        print("  错误: 创建数据库失败")
        print(f"  {result.stderr}")
        return False


def grant_permissions(psql, postgres_password):
    """步骤2: 授予权限"""
    print("=== 步骤 2/3: 授予权限 ===")
    print()

    sql = DB_PERMISSION_SQL.format(
        db_name=DB_NAME,
        db_user=DB_USER
    )

    result = execute_sql_with_superuser(psql, sql, postgres_password, database=DB_NAME)

    if result.returncode == 0:
        print("  权限授予完成")
        return True
    else:
        print("  错误: 授予权限失败")
        print(f"  {result.stderr}")
        return False


def create_tables(psql):
    """步骤3: 通过 sqlx 迁移创建表结构"""
    print("=== 步骤 3/3: 应用数据库迁移 ===")
    print()

    # 定位 backend/migrations 目录（相对本脚本）
    script_dir = os.path.dirname(os.path.abspath(__file__))
    backend_dir = os.path.normpath(os.path.join(script_dir, '..', '..', 'backend'))
    migrations_dir = os.path.join(backend_dir, 'migrations')

    database_url = f"postgres://{DB_USER}:{DB_PASSWORD}@{DB_HOST}:{DB_PORT}/{DB_NAME}"

    # 检查 sqlx-cli
    sqlx = shutil.which('sqlx')
    if not sqlx:
        print("  错误: 未找到 sqlx-cli")
        print("  安装: cargo install sqlx-cli --no-default-features --features native-tls,postgres")
        print("  或直接启动后端 (cd backend && cargo run)，后端会在启动时自动应用迁移。")
        return False

    env = os.environ.copy()
    env['DATABASE_URL'] = database_url

    result = subprocess.run(
        [sqlx, 'migrate', 'run', '--source', migrations_dir],
        capture_output=True,
        text=True,
        env=env
    )

    if result.returncode == 0:
        print(result.stdout)
        return True
    else:
        print("  错误: 迁移失败")
        print(f"  {result.stderr}")
        print()
        print("  若数据库由旧初始化脚本创建（已有表但无迁移记录），请先标记基线：")
        print(f"    cd {backend_dir} && DATABASE_URL={database_url} sqlx migrate resolve --version 1")
        return False


def test_connection(psql, user, password, database):
    """测试数据库连接"""
    env = os.environ.copy()
    env['PGPASSWORD'] = password

    try:
        result = subprocess.run(
            [psql, '-h', DB_HOST, '-p', DB_PORT, '-U', user, '-d', database, '-c', 'SELECT 1;', '-q'],
            capture_output=True,
            text=True,
            env=env
        )
        return result.returncode == 0 and '1' in result.stdout
    except Exception:
        return False


def main():
    print("=" * 50)
    print("ShareUSTC 数据库初始化脚本")
    print("=" * 50)
    print()
    print("功能: 创建数据库、用户，并通过 sqlx 迁移创建表结构")
    print("注意: 需要 PostgreSQL 超级用户权限，以及 sqlx-cli")
    print()

    # 查找 psql
    psql = find_psql()
    if not psql:
        print("错误: 未找到 psql 命令。请安装 PostgreSQL 并确保它在 PATH 中。")
        sys.exit(1)
    print(f"使用 psql: {psql}")
    print()

    # 从配置变量获取 postgres 密码
    postgres_password = POSTGRES_PASSWORD

    # 测试 postgres 连接
    print("测试 postgres 用户连接...")
    if not test_connection(psql, POSTGRES_USER, postgres_password, "postgres"):
        print("错误: 无法连接到 PostgreSQL。请检查密码和服务状态。")
        sys.exit(1)
    print("  连接成功")
    print()

    # 步骤1: 创建数据库和用户
    if not create_database(psql, postgres_password):
        sys.exit(1)
    print()

    # 步骤2: 授予权限
    if not grant_permissions(psql, postgres_password):
        sys.exit(1)
    print()

    # 步骤3: 应用迁移（建表）
    if not create_tables(psql):
        sys.exit(1)
    print()

    print("=" * 50)
    print("数据库初始化完成！")
    print("=" * 50)
    print()
    print("数据库信息:")
    print(f"  数据库名: {DB_NAME}")
    print(f"  用户名:   {DB_USER}")
    print(f"  密码:     {DB_PASSWORD}")
    print()
    print("表结构由 backend/migrations/ 下的 sqlx 迁移管理。")
    print()


if __name__ == '__main__':
    main()
