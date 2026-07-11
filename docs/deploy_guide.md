# 部署指南

> 状态：生效
> 创建日期：2026-02-06
> 最后更新：2026-02-18
> 适用范围：部署与运维

以 Ubuntu 为例。

## 1. 安装 Node.js 与 npm

```bash
sudo apt update
sudo apt install -y npm
```

安装较新版本 Node.js 可参考：[Node.js 下载说明](https://nodejs.cn/en/download)

## 2. 安装 Rust

```bash
sudo apt install -y rustup pkg-config libssl-dev
```

重启终端后执行：

```bash
rustup install stable
rustup default stable
```

## 3. 安装与初始化 PostgreSQL

安装 PostgreSQL：

```bash
sudo apt install -y postgresql
```

进入 PostgreSQL 管理终端：

```bash
sudo -u postgres psql
```

创建数据库与用户：

```sql
-- 创建用户
CREATE USER shareustc_app WITH PASSWORD 'ShareUSTC_default_pwd';

-- 创建数据库
CREATE DATABASE shareustc
    WITH
    OWNER = shareustc_app
    ENCODING = 'UTF8'
    LC_COLLATE = 'C.UTF-8'
    LC_CTYPE = 'C.UTF-8'
    TEMPLATE = template0;

-- 授予权限
GRANT ALL PRIVILEGES ON DATABASE shareustc TO shareustc_app;

-- 退出
\q
```

### 3.1 初始化表结构

表结构统一由 sqlx 迁移管理（`backend/migrations/`）。**后端进程启动时会自动应用迁移**，
全新数据库无需额外操作——直接进入下文启动后端即可。

若需在启动后端之前先建好表（如 CI、批量导入前置），可显式应用迁移，两种方式任选其一：

```bash
# 方式一：脚本（内部调用 sqlx migrate run，需已安装 sqlx-cli）
./scripts/database/db_init_tables.sh

# 方式二：直接用 sqlx-cli
cd backend
export DATABASE_URL="postgres://shareustc_app:ShareUSTC_default_pwd@localhost:5432/shareustc"
sqlx migrate run
```

sqlx-cli 安装：`cargo install sqlx-cli --no-default-features --features native-tls,postgres`。

> **存量库（由旧初始化脚本创建、已有表但无迁移记录）**：首次接入迁移前需标记基线，
> 避免在已存在的表上重复建表：
> ```bash
> cd backend
> export DATABASE_URL="postgres://shareustc_app:ShareUSTC_default_pwd@localhost:5432/shareustc"
> sqlx migrate resolve --version 1
> ```
> 标记后再启动后端或运行 `sqlx migrate run` 即可。

## 4. 存储后端选择

项目支持两种存储后端：

- `local`：默认模式，本地文件系统存储。
- `oss`：可选模式，阿里云 OSS 存储。

默认无需 OSS 配置即可运行。若需启用 OSS，请按文档继续配置：

- `docs/oss_setup.md`

## 5. 配置环境变量

### 5.1 后端

```bash
cd backend
cp .env.example .env
```

按实际环境修改 `.env`（至少包括 `DATABASE_URL`、`JWT_SECRET`、`CORS_ALLOWED_ORIGINS`）。

### 5.2 前端

```bash
cd frontend
cp .env.example .env
```

按实际域名修改 `VITE_API_BASE_URL`。

## 6. 启动服务（开发模式）

终端 1（前端）：

```bash
cd frontend
npm install
npm run dev
```

终端 2（后端）：

```bash
cd backend
cargo run
```

## 7. 访问系统

开发环境默认地址：

- `http://localhost:5173`

## 8. 生产环境注意事项

1. 修改 PostgreSQL 用户 `shareustc_app` 的默认密码，并同步更新 `backend/.env`。
2. 修改 `JWT_SECRET` 为高强度随机值。
3. `CORS_ALLOWED_ORIGINS` 使用明确域名，避免使用 `*`。
4. 部署 HTTPS 后，设置 `COOKIE_SECURE=true`。
5. 如启用 OSS，优先使用最小权限策略（详见 `docs/oss_setup.md`）。

