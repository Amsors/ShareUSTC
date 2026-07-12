# 部署指南

> 状态：生效
> 创建日期：2026-02-06
> 最后更新：2026-07-11
> 适用范围：部署与运维

本项目支持两种部署方式：

- **容器部署（推荐）**：`docker compose` 一键拉起 postgres + backend + frontend(nginx) 三容器。
- **裸机部署（备选）**：手动安装 Node/Rust/PostgreSQL 并分别启动。

---

## 容器部署（推荐）

三容器架构：浏览器经 `frontend` 容器（nginx）访问，nginx 托管前端 `dist` 并把 `/api`、`/images`
反代到 `backend` 容器；`backend` 连接 `postgres` 容器，上传文件落在 named volume。前端走同域相对
路径，无需构建时注入 API 地址，镜像可跨环境复用。

### 前置

宿主机安装 Docker 与 Docker Compose 插件（`docker compose version` 可用即可）。

### 步骤

```bash
# 1. 准备部署密钥文件（已 gitignore，不入库）
cp deploy/.env.example deploy/.env

# 2. 生成并填写 JWT_SECRET（高强度随机）
openssl rand -base64 48   # 将输出填入 deploy/.env 的 JWT_SECRET

# 3. 编辑 deploy/.env，至少设置：
#    - POSTGRES_PASSWORD，并让 DATABASE_URL 中的密码与之一致
#    - IMAGE_BASE_URL / CORS_ALLOWED_ORIGINS 为站点公开地址
#    - COOKIE_SECURE（经 HTTPS 对外时为 true）

# 4. 一键构建并启动
docker compose up -d --build

# 5. 查看状态与日志
docker compose ps
docker compose logs -f backend
```

启动后浏览器访问宿主机 **80 端口**即可（注册/登录/上传/下载/图片外链全流程同域完成）。
数据库建库由 `postgres` 镜像按 `POSTGRES_*` 自动完成，表结构迁移由后端启动时自动执行，无需手动建表。

### 必填环境变量清单（deploy/.env）

| 变量 | 说明 |
|------|------|
| `POSTGRES_DB` / `POSTGRES_USER` / `POSTGRES_PASSWORD` | postgres 容器初始化的库名/用户/密码 |
| `DATABASE_URL` | 后端连接串，主机名为服务名 `postgres`；用户名/密码/库名须与上面一致 |
| `JWT_SECRET` | 高强度随机串，长度 ≥16，拒绝空串与占位值（否则后端拒绝启动） |
| `IMAGE_BASE_URL` | 图床外链前缀，同域部署即站点公开 origin（不含末尾 `/images`） |
| `CORS_ALLOWED_ORIGINS` | 允许来源，配为站点 origin；**生产禁止 `*`** |
| `COOKIE_SECURE` | 经 HTTPS 对外时必须 `true` |
| `ADMIN_USERNAMES` | 管理员用户名列表（逗号分隔） |

### HTTPS 与 COOKIE_SECURE

`frontend` 容器对外暴露 80 端口的 HTTP。生产环境应在其外层再放一层网关（宿主机 nginx / Caddy /
云负载均衡）终结 HTTPS 并转发到容器 80 端口。启用 HTTPS 后，`deploy/.env` 中 `COOKIE_SECURE=true`
才能让认证 Cookie 正常下发（否则浏览器会因 Secure 标志在 HTTP 下丢弃 Cookie）。

### 存储与单副本约束

- 上传文件挂在 named volume `uploads`（容器内 `/data/uploads`，其下 `images/`、`resources/` 子目录由后端派生）。
- **local 存储模式下后端仅支持单副本**，不支持多实例共享本地卷（NFS 共享卷方案明确不采用）。
  需要多副本/弹性伸缩时先切换到对象存储（OSS），见 `docs/oss_setup.md` 与容器化改造记录中的「路线 B」。

### 卷备份与恢复

上传文件卷（`shareustc_uploads`，实际卷名以 `docker volume ls` 为准，通常为 `<项目名>_uploads`）：

```bash
# 备份
docker run --rm -v shareustc_uploads:/data -v "$(pwd)":/backup alpine \
  tar czf /backup/uploads_$(date +%F).tar.gz -C /data .
# 恢复
docker run --rm -v shareustc_uploads:/data -v "$(pwd)":/backup alpine \
  sh -c "cd /data && tar xzf /backup/uploads_YYYY-MM-DD.tar.gz"
```

数据库卷（`shareustc_pgdata`）建议用 `pg_dump` 逻辑备份：

```bash
docker compose exec postgres pg_dump -U "$POSTGRES_USER" "$POSTGRES_DB" > backup_$(date +%F).sql
# 恢复：docker compose exec -T postgres psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" < backup_YYYY-MM-DD.sql
```

### 排障

- 后端启动即退出并 crashloop：多为关键环境变量缺失/过弱（`DATABASE_URL` / `JWT_SECRET` /
  `IMAGE_BASE_URL`）或数据库未就绪。入口 `docker compose logs backend`，错误日志会指明缺失项。
- 健康检查：liveness `GET /api/health`（恒 200）；readiness `GET /api/health/ready`
  （数据库不可达返回 503）。compose 已用 readiness 作为 backend 的 healthcheck。
- 数据/文件是否持久：`docker compose down && docker compose up -d` 后上传文件与数据库数据应仍在
  （分别存于 `uploads`、`pgdata` 卷）；仅 `docker compose down -v` 才会删卷。

---

## 裸机部署（备选）

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

按实际环境修改 `.env`。其中 `DATABASE_URL`、`JWT_SECRET`、`IMAGE_BASE_URL` 为**必填项**
（缺失或为占位值时后端拒绝启动）；`CORS_ALLOWED_ORIGINS` 有默认值，生产环境建议显式配为站点 origin。
直接 `cp .env.example .env` 已预填可用的开发默认值。

### 5.2 前端

```bash
cd frontend
cp .env.example .env
```

前端默认走同域相对路径 `/api`：开发模式由 `vite` 的 `server.proxy` 把 `/api`、`/images`
代理到本地后端（`http://localhost:8080`），**无需设置 `VITE_API_BASE_URL`**。仅当前后端分域名
部署时才设置该变量（值需含 `/api` 后缀）。

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

