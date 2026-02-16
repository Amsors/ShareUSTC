// 用户角色
export const UserRole = {
  Guest: 'guest',
  User: 'user',
  Verified: 'verified',
  Admin: 'admin'
} as const;

export type UserRoleType = typeof UserRole[keyof typeof UserRole];

// 用户信息
export interface User {
  id: string;
  sn?: number;
  username: string;
  email?: string;
  role: UserRoleType;
  bio?: string;
  isVerified: boolean;
  createdAt: string;
}

// 登录请求
export interface LoginRequest {
  username: string;
  password: string;
}

// 注册请求
export interface RegisterRequest {
  username: string;
  password: string;
  email?: string;
}

// Token 响应
export interface TokenResponse {
  accessToken: string;
  refreshToken: string;
  tokenType: string;
  expiresIn: number;
}

// 认证响应
// 注意：Token 现在存储在 HttpOnly Cookie 中，不再在响应体中返回
// API 直接返回 User 对象（不再包装在 {user: ...} 中）
export type AuthResponse = User;

// 刷新 Token 请求
export interface RefreshTokenRequest {
  refreshToken: string;
}

// API 统一响应格式
export interface ApiResponse<T> {
  code: number;
  message: string;
  data: T;
}

// 认证状态
export interface AuthState {
  user: User | null;
  accessToken: string | null;
  refreshToken: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
}
