# ShareUSTC 数据库初始化脚本使用说明

## 脚本说明

本目录包含两个数据库初始化脚本：

| 脚本 | 用途 | 执行权限 | 执行用户 |
|------|------|----------|----------|
| `db_create_system.sh` | 创建数据库和用户 | 需要 sudo | root 或 sudo |
| `db_init_tables.sh` | 创建表、索引、触发器 | 不需要 sudo | 普通用户 |

## 前置要求

1. **PostgreSQL 已安装并运行** (版本 15+)
2. **当前用户有 sudo 权限** (用于执行系统级脚本)

## 执行步骤

### 第一步：执行系统级初始化脚本 (需要 sudo)

此脚本需要 **sudo 权限**，执行以下操作：

```bash
sudo ./db_create_system.sh
```

**脚本功能详解：**

| 步骤 | sudo 命令 | 功能说明 |
|------|-----------|----------|
| 检查服务 | `systemctl is-active postgresql` | 检查 PostgreSQL 是否运行 |
| 启动服务 | `systemctl start postgresql` | 启动 PostgreSQL 服务 |
| 开机启动 | `systemctl enable postgresql` | 设置 PostgreSQL 开机自启 |
| 创建用户 | `sudo -u postgres psql -c "CREATE USER..."` | 以 postgres 用户执行 SQL，创建数据库用户 shareustc_app |
| 创建数据库 | `sudo -u postgres psql -c "CREATE DATABASE..."` | 以 postgres 用户执行 SQL，创建 shareustc 数据库 |
| 授权 | `sudo -u postgres psql -c "GRANT..."` | 授予 shareustc_app 用户数据库访问权限 |

**脚本输出示例：**
```
=== ShareUSTC 数据库系统级初始化 ===

步骤 1/4: 检查 PostgreSQL 服务状态...
  PostgreSQL 服务正在运行

步骤 2/4: 创建数据库用户 'shareustc_app'...
  用户 'shareustc_app' 创建成功

步骤 3/4: 创建数据库 'shareustc'...
  数据库 'shareustc' 创建成功

步骤 4/4: 授予权限...
  权限授予完成

=== 系统级初始化完成 ===

数据库信息:
  数据库名: shareustc
  用户名: shareustc_app
  密码: shareustc_app_password
```

### 第二步：执行表结构初始化脚本 (不需要 sudo)

此脚本 **不需要 sudo**，普通用户即可执行：

```bash
./db_init_tables.sh
```

**脚本功能：**
- 连接到 shareustc 数据库
- 创建 13 张数据表
- 创建 30+ 个索引
- 创建自动更新时间的触发器
- 启用 pgcrypto 扩展

## 配置修改

### 修改数据库密码

编辑两个脚本文件，修改 `DB_PASSWORD` 变量：

```bash
# db_create_system.sh 和 db_init_tables.sh
DB_PASSWORD="你的安全密码"
```

### 修改数据库连接信息

如果 PostgreSQL 不在本机或端口不同，编辑 `db_init_tables.sh`：

```bash
DB_HOST="localhost"    # 修改为数据库主机
DB_PORT="5432"         # 修改为数据库端口
```

## 后端 .env 配置

数据库初始化完成后，配置后端 `.env` 文件：

```bash
# backend/.env
DATABASE_URL=postgres://shareustc_app:your_password@localhost:5432/shareustc
```

## 验证安装

### 验证数据库和用户

```bash
# 以 postgres 用户登录
sudo -u postgres psql

# 列出数据库
\l

# 列出用户
\du

# 退出
\q
```

### 验证表结构

```bash
# 连接到 shareustc 数据库
psql -h localhost -U shareustc_app -d shareustc

# 输入密码后，列出所有表
\dt

# 查看表结构
\d users
\d resources

# 退出
\q
```

## 常见问题

### 1. "sudo: ./db_create_system.sh: command not found"

确保脚本有执行权限：
```bash
chmod +x db_create_system.sh db_init_tables.sh
```

### 2. "psql: command not found"

安装 PostgreSQL 客户端：
```bash
# Ubuntu/Debian
sudo apt-get install postgresql-client

# CentOS/RHEL
sudo yum install postgresql
```

### 3. "FATAL: password authentication failed"

检查密码是否正确，或手动创建用户：
```bash
sudo -u postgres psql
CREATE USER shareustc_app WITH PASSWORD '你的密码';
CREATE DATABASE shareustc OWNER shareustc_app;
\q
```

### 4. 需要重置数据库

```bash
# 停止应用，然后删除并重建数据库
sudo -u postgres psql -c "DROP DATABASE IF EXISTS shareustc;"
sudo -u postgres psql -c "DROP USER IF EXISTS shareustc_app;"

# 重新执行脚本
sudo ./db_create_system.sh
./db_init_tables.sh
```

## 安全建议

1. **生产环境务必修改默认密码**
2. **限制数据库用户权限** - 脚本已设置最小权限
3. **配置防火墙** - 限制数据库端口 (5432) 访问
4. **定期备份** - 配置数据库自动备份

## 手动执行 SQL (备用方案)

如果脚本执行失败，可以手动执行 SQL：

```bash
# 1. 以 postgres 用户登录
sudo -u postgres psql

# 2. 执行 init.sql 中的 SQL 语句
# 复制 init.sql 内容粘贴执行
```
