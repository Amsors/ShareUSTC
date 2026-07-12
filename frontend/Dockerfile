# ShareUSTC 前端镜像（多阶段构建）
# 构建层用 node 编译 dist，运行层用 nginx 托管静态资源并反代 /api、/images。
# 参见 dev_docs/records/containerization_remediation_guide.md 阶段 5.2。

# ============ 构建层 ============
FROM node:24-alpine AS builder

WORKDIR /app

# 先复制依赖清单，利用 layer 缓存
COPY package.json package-lock.json ./
RUN npm ci

# 复制源码并构建。生产不注入 VITE_API_BASE_URL，前端走同域相对路径 /api，
# 由本镜像内的 nginx 反代到后端，镜像可跨环境复用。
COPY . .
RUN npm run build

# ============ 运行层 ============
FROM nginx:alpine AS runtime

# 站点配置（SPA fallback + /api、/images 反代）
COPY nginx.conf /etc/nginx/conf.d/default.conf

# 托管构建产物
COPY --from=builder /app/dist /usr/share/nginx/html

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
