import type { ResourceStats } from './resource';

// 收藏夹基础信息
export interface Favorite {
  id: string;
  name: string;
  resourceCount: number;
  createdAt: string;
}

// 收藏夹列表响应
export interface FavoriteListResponse {
  favorites: Favorite[];
  total: number;
}

// 收藏夹中的资源项
export interface FavoriteResourceItem {
  id: string;
  title: string;
  courseName?: string;
  resourceType: string;
  category: string;
  tags?: string[];
  fileSize?: number;
  addedAt: string;
  stats: ResourceStats;
}

// 收藏夹详情
export interface FavoriteDetail {
  id: string;
  name: string;
  createdAt: string;
  resourceCount: number;
  resources: FavoriteResourceItem[];
}

// 创建收藏夹请求
export interface CreateFavoriteRequest {
  name: string;
}

// 创建收藏夹响应
export interface CreateFavoriteResponse {
  id: string;
  name: string;
  createdAt: string;
}

// 更新收藏夹请求
export interface UpdateFavoriteRequest {
  name: string;
}

// 添加资源到收藏夹请求
export interface AddToFavoriteRequest {
  resourceId: string;
}

// 检查资源收藏状态响应
export interface CheckResourceInFavoriteResponse {
  inFavorites: string[];
  isFavorited: boolean;
}
