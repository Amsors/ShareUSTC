# OSS 可选配置指南

> 状态：生效
> 创建日期：2026-02-18
> 最后更新：2026-07-12
> 适用范围：部署与运维

本文档用于在 ShareUSTC 中启用阿里云 OSS 存储。OSS 是可选项，不配置时系统默认使用本地存储（`STORAGE_BACKEND=local`）。

## 1. 适用范围与现状

- 默认模式：`local`（无需 OSS）。
- 可选模式：`oss`。
- 当前实现：
  - 上传：前端通过 STS 临时凭证直传 OSS。
  - 下载/预览：后端生成 OSS 预签名 URL。
  - 资源 `type` 字段：使用 OSS 时写入 `oss`；默认本地为 `local`。

## 2. OSS 基础配置

1. 创建 Bucket。
2. Bucket ACL 设置为私有（Private）。
3. 记录以下信息：
- Bucket 名称（如 `your-bucket`）
- Endpoint（如 `oss-cn-shanghai.aliyuncs.com`）
- Region（如 `cn-shanghai`）

注意：`OSS_REGION` 请填写 `cn-shanghai` 这种格式，不要写 `oss-cn-shanghai`。

## 3. CORS 配置

请在 OSS Bucket 的 CORS 中添加规则（可按你的实际域名调整）：

- 允许来源（Origins）：
  - `http://localhost:5173`
  - `http://share.example.com`
  - `https://share.example.com`
- 允许方法（Methods）：`POST, GET, PUT, DELETE, HEAD`
- 允许 Headers：`*`
- 暴露 Headers：`ETag, x-oss-request-id, x-oss-hash-crc64ecma`
- 缓存时间：`3600`

## 4. RAM 权限建议

当前项目约束上传路径为 `resources/*` 与 `images/*`，建议权限也限制在这两类路径。

### 4.1 后端 AK/SK（服务端签名与读取）

后端使用 `OSS_ACCESS_KEY_ID/OSS_ACCESS_KEY_SECRET` 的 RAM 身份需要至少这些权限：

- `oss:GetObject`
- `oss:PutObject`
- `oss:DeleteObject`

资源建议限制为：

- `acs:oss:*:*:<bucket>/resources/*`
- `acs:oss:*:*:<bucket>/images/*`

如果缺少 `oss:GetObject`，会在上传回调校验元信息时出现 `403 AccessDenied`。

### 4.2 STS 角色（前端直传）

`OSS_STS_ROLE_ARN` 指向的角色建议最小权限：

- `oss:PutObject`

资源限制同上，仅 `resources/*`、`images/*`。

## 5. 后端环境变量

在 `backend/.env` 中配置（仅示例）：

```env
STORAGE_BACKEND=oss

OSS_ACCESS_KEY_ID=your_backend_ak
OSS_ACCESS_KEY_SECRET=your_backend_sk
OSS_ENDPOINT=oss-cn-shanghai.aliyuncs.com
OSS_BUCKET=your-bucket
OSS_REGION=cn-shanghai

# STS 角色 ARN（启用前端 STS 直传）
OSS_STS_ROLE_ARN=acs:ram::<your-account-id>:role/your-oss-upload-role
OSS_STS_SESSION_DURATION=900

# 可选：对象 key 前缀
OSS_KEY_PREFIX=

# 下载签名有效期（秒）
OSS_SIGNED_URL_EXPIRY=600
```

说明：

- `OSS_STS_SESSION_DURATION` 建议 900~3600 秒。
- `OSS_SIGNED_URL_EXPIRY` 是下载签名链接有效期。

## 6. 前端配置

存储后端切换是纯后端行为，前端无需为启用 OSS 做任何改动。

前端默认走**同域相对路径 `/api`**（生产由前端容器 nginx 反代、开发由 vite proxy 代理到后端），**无需设置 `VITE_API_BASE_URL`**。仅「前后端分域名部署」时才设置该变量，且值需含 `/api` 后缀：

```env
# 仅前后端分域名部署时需要
VITE_API_BASE_URL=https://api.yourdomain.com/api
```

详见 `frontend/.env.example` 与 `docs/deploy_guide.md`。

## 7. 启动与验证

1. 重启后端服务。
2. 上传一个资源（PDF/图片均可）。
3. 验证：
- 上传成功（数据库记录存在，`type=oss`）。
- 预览成功。
- 下载成功。

## 8. 常见问题排查

### 8.1 `SignatureDoesNotMatch`

优先检查：

- `OSS_REGION` 是否为 `cn-xxx` 格式。
- `OSS_ENDPOINT` 与 Bucket 所在地域是否一致。
- 服务器时间是否准确（NTP）。

### 8.2 `AccessDenied` + `AuthAction=oss:GetObject`

说明后端 RAM 身份缺少读取权限。请为后端 AK/SK 所属身份补充 `oss:GetObject` 到对应路径。

### 8.3 上传成功但回调失败“文件不存在或不可访问”

通常是：

- 上传 key 路径与回调 key 不一致；
- 或后端读取元信息权限不足（见 8.2）。

## 9. 回滚到本地存储

如需临时回滚：

1. 将 `STORAGE_BACKEND` 改回 `local`。
2. 重启后端。
3. 保留 OSS 配置不影响启动（仅不会被使用）。
