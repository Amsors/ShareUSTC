import request from './request';
import type { AxiosRequestConfig } from 'axios';
import type { User } from '../types/auth';
import type { ResourceListItem } from '../types/resource';

// 用户资料更新请求
export interface UpdateProfileRequest {
  username?: string;
  bio?: string;
  email?: string;
  socialLinks?: Record<string, string>;
}

// 实名认证请求
export interface VerificationRequest {
  realName?: string;
  studentId?: string;
  major?: string;
  grade?: string;
}

// 用户公开资料
export interface UserProfile {
  id: string;
  sn?: number;
  username: string;
  bio?: string;
  role: string;
  isVerified: boolean;
  createdAt: string;
  uploadsCount: number;
  totalLikes: number;
  totalDownloads: number;
}

// 用户主页响应（包含资源列表）
export interface UserHomepage {
  id: string;
  sn?: number;
  username: string;
  bio?: string;
  email?: string;
  role: string;
  isVerified: boolean;
  createdAt: string;
  uploadsCount: number;
  totalLikes: number;
  totalDownloads: number;
  resources: ResourceListItem[];
  resourcesTotal: number;
}

// 用户主页查询参数
export interface UserHomepageQuery {
  page?: number;
  perPage?: number;
}

// 站点公开配置
export interface SiteConfig {
  requireEmailOnRegister: boolean;
  allowUsernameChange: boolean;
  allowEmailChange: boolean;
}

// 获取当前用户信息
// config 可传入 skipAuthError 等自定义字段（如会话检查时静默处理 401）
export const getCurrentUser = (config?: AxiosRequestConfig): Promise<User> => {
  return request.get('/users/me', config);
};

// 更新当前用户资料
export const updateProfile = (data: UpdateProfileRequest): Promise<User> => {
  return request.put('/users/me', data);
};

// 实名认证（后端会设置新的 HttpOnly Cookie）
// 返回更新后的 User 对象（API 直接返回，不再包装在 {user: ...} 中）
export const verifyUser = (data: VerificationRequest): Promise<User> => {
  return request.post('/users/verify', data);
};

// 获取用户公开资料
export const getUserProfile = (userId: string): Promise<UserProfile> => {
  return request.get(`/users/${userId}`);
};

// 获取用户主页数据（包含资源列表）
export const getUserHomepage = (
  userId: string,
  query?: UserHomepageQuery
): Promise<UserHomepage> => {
  return request.get(`/users/${userId}/homepage`, { params: query });
};

// 获取站点公开配置
export const getSiteConfig = (): Promise<SiteConfig> => {
  return request.get('/config');
};

// 修改密码请求
export interface ChangePasswordRequest {
  oldPassword: string;
  newPassword: string;
}

// 修改密码
export const changePassword = (data: ChangePasswordRequest): Promise<void> => {
  return request.put('/users/me/password', data);
};

// 贡献榜单用户信息
export interface LeaderboardUser {
  id: string;
  sn?: number;
  username: string;
  bio?: string;
  role: string;
  isVerified: boolean;
  uploadsCount: number;
  totalLikes: number;
  totalDownloads: number;
}

// 贡献榜单响应
export interface LeaderboardResponse {
  users: LeaderboardUser[];
  total: number;
}

// 贡献榜单查询参数
export interface LeaderboardQuery {
  limit?: number;
}

// 获取贡献榜单
export const getLeaderboard = (query?: LeaderboardQuery): Promise<LeaderboardResponse> => {
  return request.get('/users/leaderboard', { params: query });
};
