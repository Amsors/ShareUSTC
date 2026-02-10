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
 * 打包下载收藏夹
 * @param favoriteId 收藏夹ID
 * @param favoriteName 收藏夹名称（用于文件名）
 */
export const downloadFavorite = async (favoriteId: string, favoriteName?: string): Promise<void> => {
  try {
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
      throw new Error('下载失败');
    }

    // 获取文件名
    const contentDisposition = response.headers.get('content-disposition');
    let downloadFileName = `${favoriteName || '收藏夹'}.zip`;
    if (contentDisposition) {
      const match = contentDisposition.match(/filename="(.+)"/);
      if (match && match[1]) {
        downloadFileName = match[1];
      }
    }

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
  } catch (error) {
    console.error('下载失败:', error);
    throw error;
  }
};
