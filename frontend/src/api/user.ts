import request from './request';
import type { AxiosRequestConfig } from 'axios';
import type { User } from '../types/auth';
import type { PaginationQuery } from '@/types/common';
import type {
  UpdateProfileRequest,
  VerificationRequest,
  UserProfile,
  UserHomepage,
  SiteConfig,
  ChangePasswordRequest,
  LeaderboardResponse,
  LeaderboardQuery,
} from '../types/user';

// 获取当前用户信息
// config 可传入 skipAuthError 等自定义字段（如会话检查时静默处理 401）
export const getCurrentUser = (config?: AxiosRequestConfig): Promise<User> => {
  return request({ url: '/users/me', method: 'get', ...config });
};

// 更新当前用户资料
export const updateProfile = (data: UpdateProfileRequest): Promise<User> => {
  return request({ url: '/users/me', method: 'put', data });
};

// 实名认证（后端会设置新的 HttpOnly Cookie）
// 返回更新后的 User 对象（API 直接返回，不再包装在 {user: ...} 中）
export const verifyUser = (data: VerificationRequest): Promise<User> => {
  return request({ url: '/users/verify', method: 'post', data });
};

// 获取用户公开资料
export const getUserProfile = (userId: string): Promise<UserProfile> => {
  return request({ url: `/users/${userId}`, method: 'get' });
};

// 获取用户主页数据（包含资源列表）
export const getUserHomepage = (userId: string, query?: PaginationQuery): Promise<UserHomepage> => {
  return request({ url: `/users/${userId}/homepage`, method: 'get', params: query });
};

// 获取站点公开配置
export const getSiteConfig = (): Promise<SiteConfig> => {
  return request({ url: '/config', method: 'get' });
};

// 修改密码
export const changePassword = (data: ChangePasswordRequest): Promise<void> => {
  return request({ url: '/users/me/password', method: 'put', data });
};

// 获取贡献榜单
export const getLeaderboard = (query?: LeaderboardQuery): Promise<LeaderboardResponse> => {
  return request({ url: '/users/leaderboard', method: 'get', params: query });
};
