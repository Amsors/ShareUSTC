import request from './request';
import type {
  FavoriteListResponse,
  FavoriteDetail,
  CreateFavoriteRequest,
  CreateFavoriteResponse,
  UpdateFavoriteRequest,
  AddToFavoriteRequest,
  CheckResourceInFavoriteResponse
} from '../types/favorite';

/**
 * 创建收藏夹
 * @param data 创建请求
 * @returns 创建的收藏夹信息
 */
export const createFavorite = async (data: CreateFavoriteRequest): Promise<CreateFavoriteResponse> => {
  return request({
    url: '/favorites',
    method: 'post',
    data
  }) as Promise<CreateFavoriteResponse>;
};

/**
 * 获取我的收藏夹列表
 * @returns 收藏夹列表
 */
export const getFavorites = async (): Promise<FavoriteListResponse> => {
  return request({
    url: '/favorites',
    method: 'get'
  }) as Promise<FavoriteListResponse>;
};

/**
 * 获取收藏夹详情
 * @param favoriteId 收藏夹ID
 * @returns 收藏夹详情
 */
export const getFavoriteDetail = async (favoriteId: string): Promise<FavoriteDetail> => {
  return request({
    url: `/favorites/${favoriteId}`,
    method: 'get'
  }) as Promise<FavoriteDetail>;
};

/**
 * 更新收藏夹
 * @param favoriteId 收藏夹ID
 * @param data 更新请求
 */
export const updateFavorite = async (favoriteId: string, data: UpdateFavoriteRequest): Promise<void> => {
  return request({
    url: `/favorites/${favoriteId}`,
    method: 'put',
    data
  }) as Promise<void>;
};

/**
 * 删除收藏夹
 * @param favoriteId 收藏夹ID
 */
export const deleteFavorite = async (favoriteId: string): Promise<void> => {
  return request({
    url: `/favorites/${favoriteId}`,
    method: 'delete'
  }) as Promise<void>;
};

/**
 * 添加资源到收藏夹
 * @param favoriteId 收藏夹ID
 * @param data 添加请求
 */
export const addToFavorite = async (favoriteId: string, data: AddToFavoriteRequest): Promise<void> => {
  return request({
    url: `/favorites/${favoriteId}/resources`,
    method: 'post',
    data
  }) as Promise<void>;
};

/**
 * 从收藏夹移除资源
 * @param favoriteId 收藏夹ID
 * @param resourceId 资源ID
 */
export const removeFromFavorite = async (favoriteId: string, resourceId: string): Promise<void> => {
  return request({
    url: `/favorites/${favoriteId}/resources/${resourceId}`,
    method: 'delete'
  }) as Promise<void>;
};

/**
 * 检查资源收藏状态
 * @param resourceId 资源ID
 * @returns 收藏状态
 */
export const checkResourceInFavorite = async (resourceId: string): Promise<CheckResourceInFavoriteResponse> => {
  return request({
    url: `/favorites/check/${resourceId}`,
    method: 'get'
  }) as Promise<CheckResourceInFavoriteResponse>;
};

/**
 * 从 Content-Disposition 头部解析文件名
 * 支持 RFC 5987 编码的 filename* 格式
 * @param contentDisposition Content-Disposition 头部值
 * @param fallbackName 默认文件名
 * @returns 解析后的文件名
 */
function parseFilenameFromContentDisposition(
  contentDisposition: string | null,
  fallbackName: string
): string {
  if (!contentDisposition) {
    return fallbackName;
  }

  // 首先尝试解析 RFC 5987 格式的 filename*=UTF-8''xxx
  const filenameStarMatch = contentDisposition.match(/filename\*=UTF-8''([^;]+)/i);
  if (filenameStarMatch && filenameStarMatch[1]) {
    try {
      // 解码 percent-encoded 字符串
      return decodeURIComponent(filenameStarMatch[1]);
    } catch {
      // 解码失败，继续尝试其他格式
    }
  }

  // 尝试解析标准的 filename="xxx"
  const filenameMatch = contentDisposition.match(/filename="([^"]+)"/);
  if (filenameMatch && filenameMatch[1]) {
    return filenameMatch[1];
  }

  return fallbackName;
}

/**
 * 打包下载收藏夹
 * @param favoriteId 收藏夹ID
 * @param favoriteName 收藏夹名称（用于文件名）
 */
export const downloadFavorite = async (favoriteId: string, favoriteName?: string): Promise<void> => {
  const baseUrl = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080/api';
  const cleanBaseUrl = baseUrl.replace(/\/api$/, '');
  const response = await fetch(
    `${cleanBaseUrl}/api/favorites/${favoriteId}/download`,
    {
      headers: {
        'Authorization': `Bearer ${localStorage.getItem('access_token') || ''}`
      }
    }
  );

  if (!response.ok) {
    // 尝试解析错误消息
    let errorMessage = '下载失败';
    try {
      const errorData = await response.json();
      if (errorData.message) {
        errorMessage = errorData.message;
      }
    } catch {
      // 如果解析失败，使用状态码文本
      errorMessage = response.statusText || '下载失败';
    }
    throw new Error(errorMessage);
  }

  // 获取文件名
  const contentDisposition = response.headers.get('content-disposition');
  const fallbackName = `${favoriteName || '收藏夹'}.zip`;
  const downloadFileName = parseFilenameFromContentDisposition(contentDisposition, fallbackName);

  // 创建下载链接
  const blob = await response.blob();
  const url = window.URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = downloadFileName;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  window.URL.revokeObjectURL(url);
};
