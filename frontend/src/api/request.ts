import axios, { type AxiosError, type AxiosInstance, type AxiosResponse } from 'axios';
import { useAuthStore } from '@/stores/auth';
import { ElMessage } from 'element-plus';
import router from '@/router';
import logger from '@/utils/logger';

// 扩展 axios 请求配置，声明本项目自定义字段（替代散落的 as any 访问）
declare module 'axios' {
  interface AxiosRequestConfig {
    /** 跳过响应拦截器的通用错误弹窗，由调用方自行处理（如需要业务化文案/特殊 UI） */
    skipErrorHandler?: boolean;
    /** 跳过 401/403/404 等认证类错误的自动处理（会话检查、静默刷新等场景） */
    skipAuthError?: boolean;
  }
}

// 后端统一错误响应体：{ error, message }
interface ErrorResponseData {
  error?: string;
  message?: string;
}

// 创建 axios 实例
const baseURL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080/api';
logger.info('[API]', 'Base URL:', baseURL);

const request: AxiosInstance = axios.create({
  baseURL,
  timeout: 10000,
  headers: {
    'Content-Type': 'application/json',
  },
  withCredentials: true, // 启用 Cookie 支持，使浏览器自动发送 HttpOnly Cookie
  paramsSerializer: {
    // 自定义数组参数序列化格式，使用逗号分隔值
    // 例如: teacherSns=1,2,3 而不是 teacherSns[]=1&teacherSns[]=2
    serialize: (params) => {
      const parts: string[] = [];
      for (const [key, value] of Object.entries(params)) {
        if (value === undefined || value === null) continue;
        if (Array.isArray(value)) {
          if (value.length > 0) {
            // 数组格式: key=val1,val2,val3（逗号分隔）
            parts.push(`${encodeURIComponent(key)}=${encodeURIComponent(value.join(','))}`);
          }
        } else {
          parts.push(`${encodeURIComponent(key)}=${encodeURIComponent(value)}`);
        }
      }
      return parts.join('&');
    },
  },
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

// 自定义错误类型，用于标记错误是否已被处理（拦截器已弹过窗）
export class ApiError extends Error {
  isHandled: boolean;
  status?: number;
  constructor(message: string, isHandled: boolean = false, status?: number) {
    super(message);
    this.name = 'ApiError';
    this.isHandled = isHandled;
    this.status = status;
  }
}

// 业务错误类型，用于区分需要特殊处理的错误
export class BusinessError extends Error {
  status: number;
  constructor(message: string, status: number) {
    super(message);
    this.status = status;
    this.name = 'BusinessError';
  }
}

/** 类型守卫：错误是否为拦截器抛出的 ApiError（带 isHandled 标记） */
export function isApiError(error: unknown): error is ApiError {
  return error instanceof ApiError;
}

/** 类型守卫：错误是否为需要调用方处理的 BusinessError（带 status） */
export function isBusinessError(error: unknown): error is BusinessError {
  return error instanceof BusinessError;
}

/** 错误是否已被响应拦截器处理（已弹过窗），调用方据此避免重复提示 */
export function isHandledError(error: unknown): boolean {
  return isApiError(error) && error.isHandled;
}

/** 从未知错误中安全提取可展示的消息文案 */
export function getErrorMessage(error: unknown, fallback = '操作失败'): string {
  if (error instanceof Error) {
    return error.message || fallback;
  }
  if (typeof error === 'string' && error) {
    return error;
  }
  return fallback;
}

// 响应拦截器
request.interceptors.response.use(
  (response: AxiosResponse) => {
    logger.debug('[API]', `Response ${response.config.url}`, response.data);

    // 直接返回响应数据（后端不再包装 {code, message, data}）
    return response.data;
  },
  async (error: AxiosError<ErrorResponseData>) => {
    logger.error('[API]', 'Response Error', error);

    const { response, config } = error;

    if (response) {
      const { status, data } = response;
      const message = data?.message || '请求失败';

      // 检查是否标记为跳过认证错误处理（如初始化时的登录状态检查）
      const skipAuthError = config?.skipAuthError;

      switch (status) {
        case 400:
          // 如果标记为跳过错误处理，静默处理
          if (config?.skipErrorHandler) {
            return Promise.reject(new BusinessError(message, 400));
          }
          ElMessage.error(message);
          // 标记错误已处理，避免调用方重复显示
          return Promise.reject(new ApiError(message, true));
        case 401: {
          // 如果标记为跳过认证错误，静默处理（会话检查、静默刷新等）
          if (skipAuthError) {
            return Promise.reject(error);
          }

          const errorCode = data?.error;
          const isAuthPath = (config?.url || '').includes('/auth/');

          // 仅当 access token 过期（error === "TokenExpired"）且请求不属于 /auth/* 时，
          // 才自动刷新并重放原请求；其余 401（凭证错误、缺少认证等）不触发刷新
          if (errorCode === 'TokenExpired' && !isAuthPath) {
            const authStore = useAuthStore();
            const refreshed = await authStore.refreshAccessToken();

            if (refreshed && config) {
              // 刷新成功，重试原请求（Cookie 自动发送，无需重设 Authorization）
              return request(config);
            }

            // 刷新失败（refresh token 也已过期）：清除状态并跳转登录页
            authStore.clearAuth();
            ElMessage.error('登录已失效，请重新登录');
            if (router.currentRoute.value.path !== '/login') {
              // 使用 window.location.href 硬跳转，确保所有组件状态重置
              window.location.href = '/login';
            }
            return Promise.reject(new ApiError('登录已失效', true));
          }

          // 其余 401：凭证错误 / 缺少认证 / 认证域失败，直接提示，不触发刷新
          ElMessage.error(message);
          return Promise.reject(new ApiError(message, true));
        }
        case 403:
          if (!skipAuthError) {
            ElMessage.error('没有权限访问');
          }
          return Promise.reject(new ApiError('没有权限访问', true));
        case 404:
          if (!skipAuthError) {
            ElMessage.error('请求的资源不存在');
          }
          return Promise.reject(new ApiError('请求的资源不存在', true));
        case 409:
          // 如果标记为跳过错误处理，静默处理
          if (config?.skipErrorHandler) {
            return Promise.reject(new BusinessError(message, 409));
          }
          ElMessage.error(message); // 如"用户名已存在"
          return Promise.reject(new ApiError(message, true));
        case 422:
          ElMessage.error(message);
          return Promise.reject(new ApiError(message, true));
        case 500:
          ElMessage.error('服务器错误');
          return Promise.reject(new ApiError('服务器错误', true));
        default:
          ElMessage.error(message);
          return Promise.reject(new ApiError(message, true));
      }
    } else {
      // 网络错误（CORS、超时等）
      // 如果标记为跳过认证错误，静默处理
      const skipAuthError = config?.skipAuthError;
      if (!skipAuthError) {
        ElMessage.error('网络错误，请检查网络连接');
      }
      return Promise.reject(new ApiError('网络错误', true));
    }
  }
);

export default request;
