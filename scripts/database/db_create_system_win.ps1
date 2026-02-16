# ============================================
# ShareUSTC 数据库系统级初始化脚本 (Windows版本)
# 需要管理员权限执行
# 功能: 创建数据库和用户
# ============================================

#Requires -RunAsAdministrator

# 配置变量
$DB_NAME = "shareustc"
$DB_USER = "shareustc_app"
$DB_PASSWORD = "ShareUSTC_default_pwd"  # 生产环境请修改此密码
$POSTGRES_USER = "postgres"  # PostgreSQL 超级用户

# 颜色输出函数
function Write-ColorOutput($ForegroundColor) {
    $fc = $host.UI.RawUI.ForegroundColor
    $host.UI.RawUI.ForegroundColor = $ForegroundColor
    if ($args) {
        Write-Output $args
    }
    $host.UI.RawUI.ForegroundColor = $fc
}

Write-ColorOutput Green "=== ShareUSTC 数据库系统级初始化 (Windows) ==="
Write-Output ""

# 检查 psql 是否可用
$psqlPath = Get-Command psql -ErrorAction SilentlyContinue
if (-not $psqlPath) {
    # 尝试常见安装路径
    $commonPaths = @(
        "C:\Program Files\PostgreSQL\*\bin\psql.exe",
        "C:\Program Files (x86)\PostgreSQL\*\bin\psql.exe"
    )
    $found = $false
    foreach ($path in $commonPaths) {
        $matches = Get-ChildItem -Path $path -ErrorAction SilentlyContinue
        if ($matches) {
            $env:Path += ";" + $matches[0].DirectoryName
            $found = $true
            break
        }
    }
    if (-not $found) {
        Write-ColorOutput Red "错误: 未找到 psql 命令，请安装 PostgreSQL 并添加到 PATH"
        exit 1
    }
}

# 检查 PostgreSQL 服务状态
Write-ColorOutput Yellow "步骤 1/4: 检查 PostgreSQL 服务状态..."
$service = Get-Service -Name "postgresql*" -ErrorAction SilentlyContinue | Select-Object -First 1

if ($service -and $service.Status -eq "Running") {
    Write-ColorOutput Green "  PostgreSQL 服务正在运行"
} else {
    Write-ColorOutput Yellow "  启动 PostgreSQL 服务..."
    if ($service) {
        Start-Service $service.Name
        Set-Service $service.Name -StartupType Automatic
        Write-ColorOutput Green "  PostgreSQL 服务已启动"
    } else {
        Write-ColorOutput Red "  错误: 未找到 PostgreSQL 服务，请确保 PostgreSQL 已安装"
        exit 1
    }
}

Write-Output ""

# 创建数据库用户
Write-ColorOutput Yellow "步骤 2/4: 创建数据库用户 '$DB_USER'..."
$env:PGPASSWORD = $DB_PASSWORD
try {
    $userExists = psql -U $POSTGRES_USER -d postgres -t -c "SELECT 1 FROM pg_roles WHERE rolname='$DB_USER';" 2>$null | Out-String
    if ($userExists.Trim() -eq "1") {
        Write-ColorOutput Yellow "  用户 '$DB_USER' 已存在，跳过创建"
    } else {
        psql -U $POSTGRES_USER -d postgres -c "CREATE USER $DB_USER WITH PASSWORD '$DB_PASSWORD';" 2>$null
        if ($LASTEXITCODE -eq 0) {
            Write-ColorOutput Green "  用户 '$DB_USER' 创建成功"
        } else {
            throw "创建用户失败"
        }
    }
} catch {
    Write-ColorOutput Red "  错误: 创建用户失败，请检查 PostgreSQL 是否运行以及用户名密码是否正确"
    Write-ColorOutput Red "  $_"
    exit 1
}

Write-Output ""

# 创建数据库
Write-ColorOutput Yellow "步骤 3/4: 创建数据库 '$DB_NAME'..."
try {
    $dbExists = psql -U $POSTGRES_USER -d postgres -t -c "SELECT 1 FROM pg_database WHERE datname='$DB_NAME';" 2>$null | Out-String
    if ($dbExists.Trim() -eq "1") {
        Write-ColorOutput Yellow "  数据库 '$DB_NAME' 已存在，跳过创建"
    } else {
        psql -U $POSTGRES_USER -d postgres -c "CREATE DATABASE $DB_NAME OWNER $DB_USER ENCODING 'UTF8' LC_COLLATE 'C' LC_CTYPE 'C' TEMPLATE template0;" 2>$null
        if ($LASTEXITCODE -eq 0) {
            Write-ColorOutput Green "  数据库 '$DB_NAME' 创建成功"
        } else {
            throw "创建数据库失败"
        }
    }
} catch {
    Write-ColorOutput Red "  错误: 创建数据库失败"
    Write-ColorOutput Red "  $_"
    exit 1
}

Write-Output ""

# 授予权限
Write-ColorOutput Yellow "步骤 4/4: 授予权限..."
try {
    # 授予数据库连接权限
    psql -U $POSTGRES_USER -d postgres -c "GRANT CONNECT ON DATABASE $DB_NAME TO $DB_USER;" 2>$null | Out-Null

    # 在数据库内授予 schema 权限
    psql -U $POSTGRES_USER -d $DB_NAME -c "GRANT USAGE ON SCHEMA public TO $DB_USER;" 2>$null | Out-Null
    psql -U $POSTGRES_USER -d $DB_NAME -c "GRANT CREATE ON SCHEMA public TO $DB_USER;" 2>$null | Out-Null

    # 启用 pgcrypto 扩展
    psql -U $POSTGRES_USER -d $DB_NAME -c "CREATE EXTENSION IF NOT EXISTS pgcrypto;" 2>$null | Out-Null

    Write-ColorOutput Green "  权限授予完成"
} catch {
    Write-ColorOutput Red "  错误: 授予权限失败"
    Write-ColorOutput Red "  $_"
    exit 1
}

Write-Output ""
Write-ColorOutput Green "=== 系统级初始化完成 ==="
Write-Output ""
Write-Output "数据库信息:"
Write-Output "  数据库名: $DB_NAME"
Write-Output "  用户名:   $DB_USER"
Write-Output "  密码:     $DB_PASSWORD"
Write-Output ""
Write-ColorOutput Yellow "提示: 请修改脚本中的 DB_PASSWORD 变量或使用更安全的密码生成方式"
Write-Output ""
Write-Output "下一步: 执行数据库表结构初始化"
Write-Output "  .\db_init_tables_win.ps1"
