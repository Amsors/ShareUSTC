// API 地址工具

/**
 * 获取后端服务源地址（去掉 API 基础路径末尾的 `/api`）。
 *
 * 用于拼接不经过 `src/api/request.ts` 的直链场景（如文件下载、预览、OSS 直链等），
 * 避免在多处重复 `baseUrl.replace(/\/api$/, '')`。
 *
 * - 未设置 `VITE_API_BASE_URL`（同域反代/开发代理）：返回空串，拼出的 URL 为同域相对路径
 *   （如 `/api/resources/...`、`/images/...`），浏览器按当前站点 origin 解析。
 * - 设置了 `VITE_API_BASE_URL`（分域名部署）：返回去掉末尾 `/api` 的服务源
 *   （如 `https://api.example.com`）。
 *
 * @returns 服务源地址（同域时为空串），不含末尾 `/api`
 */
export function getServerOrigin(): string {
  const baseUrl = import.meta.env.VITE_API_BASE_URL;
  if (!baseUrl) return '';
  return baseUrl.replace(/\/api$/, '');
}
