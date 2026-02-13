# ShareUSTC - AI 编码助手指南

本文件供 AI 编码助手阅读，用于快速理解项目架构和开发规范。



## 项目概述

**ShareUSTC** 是一个面向中国科学技术大学（USTC）学生的学习资源分享平台，旨在促进校内优质学习资源的共享与传承，打造互助学习社区。

### 核心功能

| 功能模块 | 描述 |
|---------|------|
| 资源管理 | 支持上传、浏览、搜索、预览、下载各类学习资料（PDF、Markdown、图片、文本等） |
| 互动系统 | 评论、点赞、多维度评分 |
| 收藏夹 | 创建个人收藏夹，支持打包下载 |
| 图床功能 | 在线上传图片，获取 Markdown 引用链接 |
| 个人主页 | 展示个人简介、上传资源、统计数据 |
| 管理后台 | 用户管理、资源审核、全站通知、数据统计 |

### 用户角色

| 功能 | 游客 | 注册用户 | 管理员 |
|------|------|----------|--------|
| 浏览/搜索/下载资源 | ✅ | ✅ | ✅ |
| 上传资源 | - | ✅ | ✅ |
| 评分/点赞/评论 | ✅ | ✅ | ✅ |
| 创建收藏夹 | - | ✅ | ✅ |
| 个人主页 | - | ✅ | ✅ |
| 管理后台 | - | - | ✅ |



## 技术栈

### 前端

| 技术 | 版本 | 用途 |
|------|------|------|
| Vue.js | 3.5+ | 前端框架 |
| TypeScript | 5.9+ | 类型安全 |
| Vite | 7.2+ | 构建工具 |
| Pinia | 2.3+ | 状态管理 |
| Element Plus | 2.13+ | UI 组件库 |
| Vue Router | 4.6+ | 路由管理 |
| Axios | 1.13+ | HTTP 客户端 |
| PDF.js | 5.4+ | PDF 预览 |
| Markdown-it | 14.1+ | Markdown 渲染 |
| md-editor-v3 | 6.3+ | Markdown 编辑器 |

### 后端

| 技术 | 版本 | 用途 |
|------|------|------|
| Rust | 1.75+ | 编程语言 |
| Actix-web | 4.x | Web 框架 |
| Actix-cors | 0.6+ | CORS 处理 |
| Actix-multipart | 0.6+ | 文件上传处理 |
| Tokio | 1.x | 异步运行时 |
| SQLx | 0.8+ | 异步 SQL 工具 |
| PostgreSQL | 15+ | 关系型数据库 |
| Argon2 | 0.5+ | 密码哈希 |
| jsonwebtoken | 9.x | JWT 身份验证 |
| Chrono | 0.4+ | 日期时间处理 |
| UUID | 1.x | 唯一标识符生成 |



## 项目结构

```
ShareUSTC/
├── frontend/              # Vue.js 前端项目
│   ├── src/
│   │   ├── api/           # API 接口层（按功能模块划分）
│   │   ├── components/    # 公共组件
│   │   │   ├── editor/    # 编辑器相关组件
│   │   │   ├── favorite/  # 收藏夹相关组件
│   │   │   ├── interaction/ # 互动组件（评论、点赞）
│   │   │   ├── notification/ # 通知组件
│   │   │   ├── preview/   # 文件预览组件
│   │   │   └── upload/    # 上传相关组件
│   │   ├── layouts/       # 布局组件
│   │   ├── router/        # 路由配置
│   │   ├── stores/        # Pinia 状态管理
│   │   ├── types/         # TypeScript 类型定义
│   │   ├── utils/         # 工具函数
│   │   └── views/         # 页面视图
│   │       ├── admin/     # 管理后台页面
│   │       ├── auth/      # 认证相关页面
│   │       ├── favorite/  # 收藏夹页面
│   │       ├── notification/ # 通知页面
│   │       ├── resource/  # 资源相关页面
│   │       └── user/      # 用户相关页面
│   ├── package.json
│   ├── vite.config.ts
│   └── tsconfig.json
├── backend/               # Rust 后端项目
│   ├── src/
│   │   ├── api/           # API 路由层
│   │   │   ├── auth.rs    # 认证路由
│   │   │   ├── user.rs    # 用户路由
│   │   │   ├── resource.rs # 资源路由
│   │   │   ├── favorite.rs # 收藏夹路由
│   │   │   ├── comment.rs # 评论路由
│   │   │   ├── notification.rs # 通知路由
│   │   │   ├── admin.rs   # 管理后台路由
│   │   │   └── image_host.rs # 图床路由
│   │   ├── services/      # 业务逻辑层
│   │   │   ├── auth_service.rs
│   │   │   ├── user_service.rs
│   │   │   ├── resource_service.rs
│   │   │   ├── favorite_service.rs
│   │   │   ├── comment_service.rs
│   │   │   ├── notification_service.rs
│   │   │   ├── admin_service.rs
│   │   │   ├── image_service.rs
│   │   │   ├── file_service.rs
│   │   │   └── audit_log_service.rs
│   │   ├── models/        # 数据模型层
│   │   │   ├── user.rs
│   │   │   ├── resource.rs
│   │   │   ├── comment.rs
│   │   │   ├── favorite.rs
│   │   │   ├── notification.rs
│   │   │   ├── image.rs
│   │   │   ├── like.rs
│   │   │   └── rating.rs
│   │   ├── middleware/    # 中间件
│   │   │   └── auth.rs    # JWT 认证中间件
│   │   ├── config/        # 配置管理
│   │   ├── db/            # 数据库连接
│   │   ├── utils/         # 工具函数
│   │   │   ├── jwt.rs     # JWT 处理
│   │   │   └── hash.rs    # 密码哈希
│   │   └── main.rs        # 应用入口
│   └── Cargo.toml
├── scripts/               # 部署脚本
│   └── database/          # 数据库初始化脚本
│       ├── db_create_system.sh  # 创建数据库和用户（需 sudo）
│       ├── db_init_tables.sh    # 创建表结构
│       ├── init.sql             # 完整 SQL 脚本
│       └── README.md
├── docs/                  # 文档
│   └── deploy_guide.md    # 部署指南
└── README.md
```



## 构建和运行

### 环境要求

- Node.js 18+ 
- Rust 1.75+
- PostgreSQL 15+

### 开发环境启动

**1. 启动前端（终端 1）**

```bash
cd frontend/
npm install
npm run dev
```

前端运行在 http://localhost:5173

**2. 启动后端（终端 2）**

```bash
cd backend/
cp .env.example .env
# 根据需要修改 .env 文件
cargo run
```

后端运行在 http://localhost:8080

**3. 数据库初始化**（首次运行）

```bash
# 需要 sudo 权限
sudo ./scripts/database/db_create_system.sh

# 不需要 sudo
./scripts/database/db_init_tables.sh
```



## 代码风格指南

### 前端（TypeScript/Vue）

1. **使用组合式 API**：所有 Vue 组件使用 `<script setup lang="ts">` 语法
2. **类型安全**：所有函数参数和返回值必须标注类型
3. **API 分层**：按功能模块划分 API 文件（`api/auth.ts`、`api/resource.ts` 等）
4. **状态管理**：使用 Pinia 管理全局状态，按功能模块划分 store
5. **组件命名**：PascalCase（如 `MarkdownEditor.vue`）
6. **路由守卫**：在 `router/index.ts` 中统一处理权限验证

### 后端（Rust）

1. **模块组织**：使用 `mod.rs` 统一导出模块
2. **错误处理**：使用自定义错误类型，通过 `Result<T, E>` 传递错误
3. **异步函数**：所有 IO 操作使用 `async/await`
4. **日志记录**：使用 `log` crate 记录关键操作
5. **函数命名**：snake_case（如 `get_resource_by_id`）
6. **结构体命名**：PascalCase（如 `ResourceService`）

### API 响应格式

统一返回 JSON 格式：

```json
{
  "code": 200,
  "message": "操作成功",
  "data": { /* 响应数据 */ }
}
```



## 认证系统

### JWT Token 机制

- **Access Token**：有效期 15 分钟（900 秒），用于 API 认证
- **Refresh Token**：有效期 7 天（604800 秒），用于刷新 Access Token
- Token 存储在前端 localStorage

### 认证中间件

- 使用自定义 `JwtAuth` 中间件处理认证
- 支持公开路径配置（如资源列表、详情等）
- 认证信息通过 `Authorization: Bearer <token>` 头传递

### 用户角色

```rust
enum UserRole {
    User,   // 普通用户
    Admin,  // 管理员
}
```

管理员用户名在 `backend/.env` 的 `ADMIN_USERNAMES` 中配置，后端启动时自动同步权限。



## 数据库设计

### 核心表

| 表名 | 描述 |
|------|------|
| users | 用户表 |
| resources | 资源表 |
| resource_stats | 资源统计表 |
| comments | 评论表 |
| likes | 点赞表 |
| ratings | 评分表 |
| favorites | 收藏夹表 |
| favorite_resources | 收藏夹资源关联表 |
| notifications | 通知表 |
| notification_reads | 通知已读记录表 |
| images | 图片表 |
| audit_logs | 审计日志表 |
| download_logs | 下载记录表 |
| claims | 申领表 |

### 数据库连接池配置

- 最大连接数：20
- 最小连接数：5
- 获取连接超时：3 秒
- 空闲连接超时：600 秒
- 连接最大生命周期：1800 秒



## 配置文件

### 后端 `.env`

```bash
# 数据库配置
DATABASE_URL=postgres://shareustc_app:password@localhost:5432/shareustc

# JWT 配置
JWT_SECRET=your-super-secret-jwt-key
JWT_EXPIRATION=900
JWT_REFRESH_EXPIRATION=604800

# 服务器配置
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# CORS 配置
CORS_ALLOWED_ORIGINS=http://localhost:5173,http://127.0.0.1:5173

# 文件上传配置
IMAGE_UPLOAD_PATH=./uploads/images
RESOURCE_UPLOAD_PATH=./uploads/resources
MAX_FILE_SIZE=104857600

# 日志级别
RUST_LOG=backend=info,actix_web=info,sqlx=warn

# 管理员配置
ADMIN_USERNAMES=admin,user1,user2
```

### 前端 `.env`

```bash
VITE_API_BASE_URL=http://localhost:8080/api
VITE_APP_NAME=ShareUSTC
VITE_APP_VERSION=0.1.0
```



## 开发注意事项

### 文件上传

- 支持 multipart/form-data 格式
- 最大文件大小：100MB
- 允许的文件类型：pdf, doc, docx, ppt, pptx, txt, md, jpg, jpeg, png, zip
- 文件存储在 `./uploads/resources/` 目录
- 图片存储在 `./uploads/images/` 目录

### API 请求处理

- 前端使用 Axios 拦截器统一处理 Token 刷新
- 401 错误自动尝试刷新 Token
- 刷新失败则清除登录状态并跳转登录页

### 路由权限

路由元信息字段：
- `public: true` - 公开访问
- `requiresAuth: true` - 需要登录
- `requiresAdmin: true` - 需要管理员权限
- `guestOnly: true` - 仅未登录用户可访问

### 错误处理

- 前端：使用 Element Plus 的 `ElMessage` 显示错误提示
- 后端：返回统一格式的错误响应，包含错误码和消息



## 安全考虑

1. **密码安全**：使用 Argon2 算法进行密码哈希
2. **JWT 安全**：使用强密钥，设置合理的过期时间
3. **CORS 配置**：生产环境限制允许的域名
4. **SQL 注入防护**：使用 SQLx 的参数化查询
5. **文件上传安全**：限制文件类型和大小
6. **审计日志**：记录关键操作（登录、注册、管理操作）



## 部署相关

### 生产环境检查清单

- [ ] 修改 JWT_SECRET 为强密钥
- [ ] 修改数据库密码
- [ ] 配置正确的 CORS_ALLOWED_ORIGINS
- [ ] 设置合适的日志级别（建议 info 或 warn）
- [ ] 配置管理员用户名
- [ ] 确保上传目录有正确的写入权限
- [ ] 配置反向代理（如 Nginx）

### 数据库备份

建议定期备份 PostgreSQL 数据库：

```bash
pg_dump -h localhost -U shareustc_app shareustc > backup.sql
```



## 常用命令

```bash
# 前端开发
npm run dev          # 启动开发服务器
npm run build        # 生产构建
npm run preview      # 预览生产构建

# 后端开发
cargo run            # 运行开发服务器
cargo build --release # 生产构建
cargo check          # 检查代码
cargo clippy         # 运行 Clippy 检查

# 数据库
sudo ./scripts/database/db_create_system.sh  # 创建数据库和用户
./scripts/database/db_init_tables.sh         # 初始化表结构
```



## 贡献规范

- 遵循 [Conventional Commits](https://www.conventionalcommits.org/) 规范
- 代码提交前运行测试
- 保持代码风格一致
