# 部署指南

version 1.3

2025/2/12/19:52

以Ubuntu 25.10为例



## 安装npm与nodejs

```bash
sudo apt install npm
```

安装较新版本的nodejs：[教程](https://nodejs.cn/en/download)



## 安装rust

```bash
sudo apt install rustup pkg-config libssl-dev
```

重启终端以加载环境变量，然后安装rust：

```bash
rustup install stable
rustup default stable
```



## 安装与初始化数据库

安装postgresql：

```bash
sudo apt install postgresql
```

登录到postgres管理员用户：

```bash
sudo -u postgres psql
```

创建软件所使用数据库与用户：

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

在项目**根目录**执行数据库初始化脚本：

```bash
./scripts/database/db_init_tables.sh
```



## 启动前后端服务

在根目录下开启一个终端，然后执行：

```bash
cd frontend/
npm install
npm run dev
```

在根目录下开启另一个终端，然后执行：

```bash
cd backend/
cp .env.example .env
cargo run
```

**如有必要，请修改前端和后端的两个`.env`文件**



## 开始使用

[http://localhost:5173](http://localhost:5173)

之后将更新将网站部署到服务器的详细教程，敬请期待



## 注意

在生产环境请修改postgresql数据库用户`shareustc_app`的密码，并同步修改`./backend/.env`中填入的密码

