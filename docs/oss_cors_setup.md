# OSS CORS 配置指南

> 状态：生效
> 创建日期：2026-02-20
> 最后更新：2026-02-20
> 适用范围：部署与运维

本指南说明如何配置阿里云 OSS 的 CORS 规则，以支持浏览器直接从 OSS 获取资源进行预览。

在本项目中：
- **下载功能**：使用 302 重定向到 OSS 签名 URL（浏览器导航模式，不触发 CORS）
- **预览功能**：需要浏览器直接从 OSS 获取内容（AJAX/fetch 或 `<img>`/`<iframe>` 标签），这会触发 CORS



## CORS 配置步骤

1. 登录 [阿里云控制台](https://oss.console.aliyun.com/)
2. 进入你的 OSS Bucket
3. 点击左侧菜单 **"数据安全" → "跨域设置"**
4. 点击 **"创建规则"**，填写以下信息：

**CORS 规则配置：**

| 字段 | 值 | 说明 |
|------|-----|------|
| 来源 (Allowed Origins) | `https://share.ustcer.top` | 你的前端域名，本地开发可加 `http://localhost:5173` |
| 允许 Methods | `GET`, `HEAD` | 预览只需要读取权限 |
| 允许 Headers | `*` | 允许所有请求头 |
| 暴露 Headers | `ETag`, `x-oss-request-id`, `x-oss-hash-crc64ecma` | 建议暴露这些头 |
| 缓存时间 | `3600` | 预检请求缓存1小时 |

**多域名配置示例：**
```
来源: https://share.ustcer.top
来源: http://localhost:5173
来源: http://127.0.0.1:5173
```

5. 点击 **"确定"** 保存规则
