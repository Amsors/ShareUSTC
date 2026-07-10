import request from '@/api/request';
import type { AxiosRequestConfig } from 'axios';
import type { LoginRequest, RegisterRequest, User } from '@/types/auth';

// 用户注册
export const register = (data: RegisterRequest): Promise<User> => {
  return request({ url: '/auth/register', method: 'post', data });
};

// 用户登录
export const login = (data: LoginRequest): Promise<User> => {
  return request({ url: '/auth/login', method: 'post', data });
};

// 刷新 Token
// 后端从 HttpOnly Cookie 中读取 refresh_token，不需要前端传递
// config 可传入 skipAuthError（静默刷新，失败由调用方按返回值处理）
export const refreshToken = (config?: AxiosRequestConfig): Promise<{ message: string }> => {
  return request({ url: '/auth/refresh', method: 'post', ...config });
};

// 用户登出
export const logout = (): Promise<{ message: string }> => {
  return request({ url: '/auth/logout', method: 'post' });
};
