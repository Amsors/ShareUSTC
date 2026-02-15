import request from './request';
import type {
  LoginRequest,
  RegisterRequest,
  AuthResponse
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
// 后端从 HttpOnly Cookie 中读取 refresh_token，不需要前端传递
export const refreshToken = (): Promise<{ message: string }> => {
  return request.post('/auth/refresh');
};

// 用户登出
export const logout = (): Promise<{ message: string }> => {
  return request.post('/auth/logout');
};
