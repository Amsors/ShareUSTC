import axios, { type AxiosError, type AxiosInstance, type AxiosResponse } from 'axios';
import { useAuthStore } from '../stores/auth';
import { ElMessage } from 'element-plus';
import router from '../router';
import logger from '../utils/logger';

// 创建 axios 实例
const baseURL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080/api';
console.log('[API] Base URL:', baseURL);

const request: AxiosInstance = axios.create({
  baseURL,
  timeout: 10000,
  headers: {
    'Content-Type': 'application/json'
  },
  withCredentials: true, // 启用 Cookie 支持，使浏览器自动发送 HttpOnly Cookie
});

// 请求拦截器
request.interceptors.request.use(
  (config) => {
    // Cookie 会自动通过 withCredentials 发送，不需要手动设置 Authorization 头
    // 但为了兼容可能需要手动传递 Token 的场景（如文件下载），保留从 store 获取 token 的逻辑
    const authStore = useAuthStore();
    const token = authStore.accessToken;

    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }

    // 如果是 FormData，删除默认的 Content-Type，让浏览器自动设置 multipart/form-data 和 boundary
    if (config.data instanceof FormData) {
      delete config.headers['Content-Type'];
    }

    logger.debug('[API]', `Request ${config.method?.toUpperCase()} ${config.url}`, config.data);
    return config;
  },
  (error) => {
    logger.error('[API]', 'Request Error', error);
    return Promise.reject(error);
  }
);

// 自定义错误类型，用于标记错误是否已被处理
class ApiError extends Error {
  isHandled: boolean;
  constructor(message: string, isHandled: boolean = false) {
    super(message);
    this.isHandled = isHandled;
  }
}

// 响应拦截器
request.interceptors.response.use(
  (response: AxiosResponse) => {
    logger.debug('[API]', `Response ${response.config.url}`, response.data);

    // 直接返回响应数据（后端不再包装 {code, message, data}）
    return response.data;
  },
  async (error: AxiosError) => {
    logger.error('[API]', 'Response Error', error);

    const { response, config } = error;

    if (response) {
      const { status, data } = response as any;
      const message = data?.message || '请求失败';

      // 检查是否标记为跳过认证错误处理（如初始化时的登录状态检查）
      const skipAuthError = (config as any)?.skipAuthError;

      switch (status) {
        case 400:
          ElMessage.error(message);
          break;
        case 401:
          // 如果标记为跳过认证错误，静默处理
          if (skipAuthError) {
            return Promise.reject(error);
          }

          // Token 过期，尝试刷新
          const authStore = useAuthStore();
          const refreshed = await authStore.refreshAccessToken();

          if (refreshed) {
            // 刷新成功，重试原请求
            if (config) {
              // Cookie 会自动发送，不需要重新设置 Authorization
              return request(config);
            }
          } else {
            // 刷新失败，清除登录状态并提示
            authStore.clearAuth();
            ElMessage.error('登录已失效，请重新登录');
            // 如果不在登录页面，强制跳转到登录（使用硬跳转确保状态完全重置）
            if (router.currentRoute.value.path !== '/login') {
              // 使用 window.location.href 强制刷新，确保所有组件状态重置
              window.location.href = '/login';
            }
          }
          break;
        case 403:
          if (!skipAuthError) {
            ElMessage.error('没有权限访问');
          }
          break;
        case 404:
          if (!skipAuthError) {
            ElMessage.error('请求的资源不存在');
          }
          break;
        case 409:
          ElMessage.error(message); // 如"用户名已存在"
          break;
        case 422:
          ElMessage.error(message);
          break;
        case 500:
          ElMessage.error('服务器错误');
          break;
        default:
          ElMessage.error(message);
      }

      return Promise.reject(error);
    } else {
      // 网络错误（CORS、超时等）
      // 如果标记为跳过认证错误，静默处理
      const skipAuthError = (config as any)?.skipAuthError;
      if (!skipAuthError) {
        ElMessage.error('网络错误，请检查网络连接');
      }
      return Promise.reject(new ApiError('网络错误', true));
    }
  }
);

export default request;
