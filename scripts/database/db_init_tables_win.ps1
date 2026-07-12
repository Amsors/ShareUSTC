# ============================================
# ShareUSTC 数据库表结构初始化脚本 (Windows PowerShell)
#
# 表结构已统一由 sqlx 迁移管理（backend/migrations/），本脚本调用
# `sqlx migrate run` 应用迁移，不再内嵌建表 SQL。
# 前置：数据库与用户已创建（见 db_create_system_win.ps1 或 docs/deploy_guide.md）。
#
# 说明：后端进程启动时也会自动执行迁移（见 backend/src/main.rs）。
#
# 适用范围：仅裸机部署。容器部署时迁移由后端启动自动执行，无需运行本脚本
# （见 docs/deploy_guide.md「容器部署」）。
# ============================================

# 配置变量（应与 db_create_system_win.ps1 保持一致）
$DB_NAME = "shareustc"
$DB_USER = "shareustc_app"
$DB_PASSWORD = "ShareUSTC_default_pwd"
$DB_HOST = "localhost"
$DB_PORT = "5432"

# 颜色输出
function Write-Green($msg) { Write-Host $msg -ForegroundColor Green }
function Write-Yellow($msg) { Write-Host $msg -ForegroundColor Yellow }
function Write-Red($msg) { Write-Host $msg -ForegroundColor Red }

Write-Green "=== ShareUSTC 数据库表结构初始化（sqlx 迁移）==="
Write-Host ""

# 定位 backend 目录（迁移文件所在）
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$BackendDir = (Resolve-Path (Join-Path $ScriptDir "..\..\backend")).Path
$MigrationsDir = Join-Path $BackendDir "migrations"

$DatabaseUrl = "postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}"

# 检查 psql 是否可用
$psqlPath = Get-Command psql -ErrorAction SilentlyContinue
if (-not $psqlPath) {
    $candidates = @(
        "C:\Program Files\PostgreSQL\*\bin\psql.exe",
        "C:\Program Files (x86)\PostgreSQL\*\bin\psql.exe"
    )
    foreach ($pattern in $candidates) {
        $found = Get-ChildItem $pattern -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($found) {
            $env:Path = "$($found.DirectoryName);$env:Path"
            break
        }
    }
    if (-not (Get-Command psql -ErrorAction SilentlyContinue)) {
        Write-Red "错误: 未找到 psql 命令，请安装 PostgreSQL 客户端"
        exit 1
    }
}

# 测试数据库连接
Write-Yellow "测试数据库连接..."
$env:PGPASSWORD = $DB_PASSWORD
$null = psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -c "SELECT 1;" 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Red "错误: 无法连接到数据库，请检查:"
    Write-Host "  1. 数据库是否已创建 (运行 db_create_system_win.ps1)"
    Write-Host "  2. 用户名、密码是否正确"
    Write-Host "  3. PostgreSQL 服务是否运行"
    exit 1
}
Write-Green "  数据库连接成功"
Write-Host ""

# 检查 sqlx-cli
if (-not (Get-Command sqlx -ErrorAction SilentlyContinue)) {
    Write-Red "错误: 未找到 sqlx-cli"
    Write-Host "  安装: cargo install sqlx-cli --no-default-features --features native-tls,postgres"
    Write-Host "  或直接启动后端 (cd backend; cargo run)，后端会在启动时自动应用迁移。"
    exit 1
}

# 存量库保护：已有业务表但无 sqlx 迁移记录时，先标记基线再运行迁移
$hasUsers = (psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -tAc "SELECT to_regclass('public.users') IS NOT NULL;").Trim()
$hasMig = (psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -tAc "SELECT to_regclass('public._sqlx_migrations') IS NOT NULL;").Trim()
if ($hasUsers -eq "t" -and $hasMig -ne "t") {
    Write-Yellow "检测到存量库（已有 users 表但无迁移记录）。"
    Write-Host "  这是从旧初始化脚本迁移而来的数据库，请先标记基线（不重复建表）："
    Write-Green "    cd `"$BackendDir`"; `$env:DATABASE_URL=`"$DatabaseUrl`"; sqlx migrate resolve --version 1"
    Write-Host "  标记后再次运行本脚本即可应用后续迁移。"
    exit 1
}

# 应用迁移
Write-Yellow "应用数据库迁移..."
$env:DATABASE_URL = $DatabaseUrl
sqlx migrate run --source $MigrationsDir
if ($LASTEXITCODE -ne 0) {
    Write-Red "迁移失败"
    exit 1
}

Write-Host ""
Write-Green "=== 表结构初始化完成 ==="
Write-Host "已应用 backend/migrations/ 下的全部迁移。"
