// 用户相关类型定义
import type { ResourceListItem } from './resource';

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

// 站点公开配置
export interface SiteConfig {
  requireEmailOnRegister: boolean;
  allowUsernameChange: boolean;
  allowEmailChange: boolean;
}

// 修改密码请求
export interface ChangePasswordRequest {
  oldPassword: string;
  newPassword: string;
}

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
