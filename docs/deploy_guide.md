# ShareUSTC 开源部署与开发指南

版本：3.0  
更新日期：2026-02-13  
适用范围：开源贡献者本地开发、测试环境、基础生产部署

## 1. 先看这里（开源项目建议）

本文按两条路径组织，降低首次接入门槛：

- 路径 A（推荐新贡献者）：最小可运行环境，不依赖 OSS。
- 路径 B（完整功能）：接入阿里云 OSS + STS，支持前端上传链路。

说明：

1. 示例账号、密码仅用于本地开发演示，不能用于生产。
2. `.env` 文件仅本地使用，不要提交到仓库。
3. 当前前端上传页面依赖 OSS；未配置 OSS 时，注册/登录/浏览可用，但上传能力不可用。

## 2. 环境要求

- Node.js >= 18（建议 20+）
- Rust stable（2021 edition）
- PostgreSQL >= 14
- 阿里云 OSS + RAM（仅路径 B 必需）

## 3. 安装基础依赖（Ubuntu 示例）

### 3.1 Node.js / npm（推荐 nvm）

```bash
sudo apt update
sudo apt install -y curl ca-certificates
curl -fsSL https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.3/install.sh | bash

export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"

nvm install --lts
nvm use --lts
node -v
npm -v
```

如需 apt 兜底安装：

```bash
sudo apt install -y nodejs npm
```

### 3.2 Rust

```bash
sudo apt install -y build-essential pkg-config libssl-dev
curl https://sh.rustup.rs -sSf | sh -s -- -y
source "$HOME/.cargo/env"
rustup default stable
rustc -V
cargo -V
```

### 3.3 PostgreSQL

```bash
sudo apt install -y postgresql postgresql-contrib
sudo systemctl enable --now postgresql
psql --version
```

## 4. 路径 A：最小可运行（无 OSS）

### 4.1 初始化数据库（本地开发示例）

```bash
sudo -u postgres psql
```

```sql
CREATE USER shareustc_app WITH PASSWORD 'ShareUSTC_default_pwd';

CREATE DATABASE shareustc
    WITH
    OWNER = shareustc_app
    ENCODING = 'UTF8'
    LC_COLLATE = 'C.UTF-8'
    LC_CTYPE = 'C.UTF-8'
    TEMPLATE = template0;

GRANT ALL PRIVILEGES ON DATABASE shareustc TO shareustc_app;
\q
```

```bash
./scripts/db_init_tables.sh
```

注意：`scripts/db_init_tables.sh` 当前使用固定连接参数。若你修改数据库账号或密码，需要同步调整脚本中的 `DB_*` 变量，并与 `backend/.env` 的 `DATABASE_URL` 保持一致。

### 4.2 配置后端环境变量

```bash
cd backend
cp .env.example .env
```

至少确认：

- `DATABASE_URL`
- `JWT_SECRET`
- `SERVER_HOST`
- `SERVER_PORT`
- `IMAGE_BASE_URL`
- `ADMIN_USERNAME`
- `RUST_LOG`

说明：无 OSS 场景下，OSS 相关变量可先保留示例占位值；仅在调用 OSS 相关接口时才会报配置错误。

### 4.3 配置前端环境变量

```bash
cd frontend
cp .env.example .env
```

至少确认：

- `VITE_API_BASE_URL`

本地建议值：

```env
VITE_API_BASE_URL=http://127.0.0.1:8080/api
```

注意：不要以 `/` 结尾。

### 4.4 启动服务

```bash
cd backend
cargo run
```

```bash
cd frontend
npm install
npm run dev
```

默认访问：

- 前端：`http://localhost:5173`
- 后端：`http://127.0.0.1:8080`

## 5. 路径 B：完整功能（含 OSS + STS）

在路径 A 基础上，补齐 OSS 配置。

### 5.1 OSS / RAM 最小准备清单

1. 创建私有 Bucket（建议开启阻止公共访问）。
2. 创建可 AssumeRole 的 RAM 角色。
3. 为后端长期 AK 配置最小权限：
   - `sts:AssumeRole`
   - OSS 对象签名/删除所需权限
4. 为 STS 角色配置上传权限，仅允许：
   - `resources/*`
   - `images/*`
5. 为 Bucket 配置 CORS，至少允许前端域名对对象执行 `PUT/POST/GET/HEAD`。

### 5.2 配置后端 OSS 环境变量

在 `backend/.env` 中补齐：

- `ALIYUN_ACCESS_KEY_ID`
- `ALIYUN_ACCESS_KEY_SECRET`
- `OSS_BUCKET`
- `OSS_REGION`
- `OSS_ENDPOINT`
- `OSS_PUBLIC_URL`
- `STS_ROLE_ARN`
- `STS_SESSION_DURATION`

详细策略和样例参考：`dev_doc/oss_moderation_plan.md`

## 6. 构建检查（推荐）

```bash
cd frontend && npm run build
cd ../backend && cargo check
```

## 7. 当前已知限制（对开源贡献者透明）

1. 后端 CORS 白名单当前为硬编码（仅 `localhost:5173` 与 `127.0.0.1:5173`），生产部署前需要修改 `backend/src/main.rs` 并重新构建。
2. 前端上传页面当前走 OSS 直传流程，未配置 OSS 时上传功能不可用。
3. 数据库初始化脚本当前使用固定连接参数，跨环境复用性有限。

## 8. 生产部署注意事项

1. 修改 `JWT_SECRET` 与数据库密码为强随机值。
2. 后端仅保留必要 CORS 域名。
3. 使用 Nginx 反向代理并启用 HTTPS，至少覆盖 `/api` 与 `/images` 路由。
4. OSS Bucket 保持私有，图片通过 `/images/{id}` 由后端签名跳转访问。
5. 建议将日志级别设置为 `info`，并接入集中化日志系统。

Nginx 反向代理示例（最小可用）：

```nginx
server {
    listen 443 ssl;
    server_name share.example.com;

    root /var/www/shareustc/dist;
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }

    location /api/ {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location /images/ {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```
