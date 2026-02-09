import request from './request';
import type {
  LoginRequest,
  RegisterRequest,
  RefreshTokenRequest,
  AuthResponse,
  TokenResponse
} from '../types/auth';

// 用户注册
export const register = (data: RegisterRequest): Promise<AuthResponse> => {
  return request.post('/auth/register', data);
};

// 用户登录
export const login = (data: LoginRequest): Promise<AuthResponse> => {
  return request.post('/auth/login', data);
};

// 刷新 Token
export const refreshToken = (data: RefreshTokenRequest): Promise<TokenResponse> => {
  return request.post('/auth/refresh', data);
};

// 用户登出
export const logout = (): Promise<void> => {
  return request.post('/auth/logout');
};
