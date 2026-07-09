// API 地址工具

/**
 * 获取后端服务源地址（去掉 API 基础路径末尾的 `/api`）。
 *
 * 用于拼接不经过 `src/api/request.ts` 的直链场景（如文件下载、预览、OSS 直链等），
 * 避免在多处重复 `baseUrl.replace(/\/api$/, '')`。
 *
 * @returns 形如 `http://localhost:8080` 的服务源地址（不含末尾 `/api`）
 */
export function getServerOrigin(): string {
  const baseUrl = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080/api';
  return baseUrl.replace(/\/api$/, '');
}
