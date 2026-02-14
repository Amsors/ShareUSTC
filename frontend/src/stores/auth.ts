import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { login, register, refreshToken, logout } from '../api/auth';
import axios from 'axios';
import type {
  User,
  LoginRequest,
  RegisterRequest,
  AuthResponse
} from '../types/auth';
import { UserRole } from '../types/auth';
import { ElMessage } from 'element-plus';
import logger from '../utils/logger';

const TOKEN_KEY = 'access_token';
const REFRESH_TOKEN_KEY = 'refresh_token';
const USER_KEY = 'user';

// 安全的 localStorage 操作函数
const safeStorage = {
  setItem(key: string, value: string): boolean {
    try {
      localStorage.setItem(key, value);
      return true;
    } catch (e) {
      logger.error('[Storage]', `设置 ${key} 失败`, e);
      return false;
    }
  },
  getItem(key: string): string | null {
    try {
      return localStorage.getItem(key);
    } catch (e) {
      logger.error('[Storage]', `读取 ${key} 失败`, e);
      return null;
    }
  },
  removeItem(key: string): boolean {
    try {
      localStorage.removeItem(key);
      return true;
    } catch (e) {
      logger.error('[Storage]', `删除 ${key} 失败`, e);
      return false;
    }
  }
};

// 创建一个独立的 axios 实例用于初始化验证（不经过响应拦截器的处理）
const verifyRequest = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080/api',
  timeout: 5000,
});

export const useAuthStore = defineStore('auth', () => {
  // State
  const user = ref<User | null>(null);
  const accessToken = ref<string | null>(safeStorage.getItem(TOKEN_KEY));
  const refreshTokenValue = ref<string | null>(safeStorage.getItem(REFRESH_TOKEN_KEY));
  const isLoading = ref(false);
  const isAuthChecked = ref(false); // 标记是否已完成认证状态检查

  // Getters
  const isAuthenticated = computed(() => !!accessToken.value && !!user.value);
  const isAdmin = computed(() => user.value?.role === UserRole.Admin);
  const isVerified = computed(() => user.value?.isVerified || false);

  // Actions

  // 初始化（从 localStorage 恢复并验证 Token）
  const initialize = async (): Promise<boolean> => {
    const storedToken = safeStorage.getItem(TOKEN_KEY);
    const storedUser = safeStorage.getItem(USER_KEY);
    const storedRefreshToken = safeStorage.getItem(REFRESH_TOKEN_KEY);

    if (!storedToken || !storedUser) {
      clearAuth();
      isAuthChecked.value = true;
      return false;
    }

    // 先设置 token（用于 API 调用），但不在 UI 显示用户信息
    accessToken.value = storedToken;
    refreshTokenValue.value = storedRefreshToken;

    // 验证 Token 有效性
    try {
      // 使用独立实例发送验证请求，避免触发响应拦截器的自动处理
      const response = await verifyRequest.get('/users/me', {
        headers: { Authorization: `Bearer ${storedToken}` }
      });

      if (response.data.code === 200 && response.data.data) {
        // Token 有效，更新用户信息
        user.value = response.data.data;
        safeStorage.setItem(USER_KEY, JSON.stringify(response.data.data));
        logger.info('[Auth]', `Token 验证成功 | username=${response.data.data.username}`);
        isAuthChecked.value = true;
        return true;
      } else {
        // 响应格式不正确，保留登录状态
        logger.warn('[Auth]', 'Token 验证响应格式不正确，保留登录状态');
        isAuthChecked.value = true;
        return true;
      }
    } catch (error: any) {
      if (error.response?.status === 401) {
        logger.warn('[Auth]', 'Access Token 已过期，尝试刷新...');
        // 立即清除用户信息，避免 UI 上继续显示已登录状态
        user.value = null;

        // 尝试用 Refresh Token 刷新
        if (storedRefreshToken) {
          try {
            const refreshResponse = await verifyRequest.post('/auth/refresh', {
              refreshToken: storedRefreshToken
            });

            if (refreshResponse.data.code === 200 && refreshResponse.data.data) {
              const { accessToken: newAccessToken, refreshToken: newRefreshToken } = refreshResponse.data.data;

              // 更新 Token
              accessToken.value = newAccessToken;
              refreshTokenValue.value = newRefreshToken;
              safeStorage.setItem(TOKEN_KEY, newAccessToken);
              safeStorage.setItem(REFRESH_TOKEN_KEY, newRefreshToken);

              // 用新 Token 获取用户信息
              const userResponse = await verifyRequest.get('/users/me', {
                headers: { Authorization: `Bearer ${newAccessToken}` }
              });

              if (userResponse.data.code === 200 && userResponse.data.data) {
                user.value = userResponse.data.data;
                safeStorage.setItem(USER_KEY, JSON.stringify(userResponse.data.data));
                logger.info('[Auth]', `Token 刷新成功 | username=${userResponse.data.data.username}`);
                isAuthChecked.value = true;
                return true;
              }
            }
          } catch (refreshError: any) {
            logger.warn('[Auth]', 'Refresh Token 也已过期');
            ElMessage.warning('登录已失效，请重新登录');
          }
        }

        // 只有确定是 401 且刷新失败时，才清除登录状态
        logger.warn('[Auth]', '认证已失效，清除登录状态');
        user.value = null;
        accessToken.value = null;
        refreshTokenValue.value = null;
        clearStorage();
        ElMessage.warning('登录已失效，请重新登录');
        isAuthChecked.value = true;
        return false;
      } else if (error.code === 'ECONNABORTED' || error.message?.includes('timeout')) {
        // 请求超时，清除登录状态（因为无法确定token是否有效）
        logger.warn('[Auth]', 'Token 验证请求超时，清除登录状态');
        user.value = null;
        accessToken.value = null;
        refreshTokenValue.value = null;
        clearStorage();
        ElMessage.warning('登录验证超时，请重新登录');
        isAuthChecked.value = true;
        return false;
      } else {
        // 其他错误（网络错误、服务器错误等），清除登录状态
        logger.warn('[Auth]', `Token 验证请求失败 | error=${error.message || error}`);
        user.value = null;
        accessToken.value = null;
        refreshTokenValue.value = null;
        clearStorage();
        ElMessage.warning('登录验证失败，请重新登录');
        isAuthChecked.value = true;
        return false;
      }
    }
  };

  // 登录
  const loginUser = async (credentials: LoginRequest): Promise<boolean> => {
    isLoading.value = true;
    try {
      const response = await login(credentials);
      setAuthData(response);
      ElMessage.success('登录成功');
      return true;
    } catch (error: any) {
      logger.error('[Auth]', '登录失败', error);
      // 如果错误已经被拦截器处理（显示过消息），则不再显示
      if (!error.isHandled) {
        ElMessage.error(error.message || '登录失败');
      }
      return false;
    } finally {
      isLoading.value = false;
    }
  };

  // 注册
  const registerUser = async (data: RegisterRequest): Promise<boolean> => {
    isLoading.value = true;
    try {
      const response = await register(data);
      setAuthData(response);
      ElMessage.success('注册成功');
      return true;
    } catch (error: any) {
      logger.error('[Auth]', '注册失败', error);
      // 如果错误已经被拦截器处理（显示过消息），则不再显示
      if (!error.isHandled) {
        ElMessage.error(error.message || '注册失败');
      }
      return false;
    } finally {
      isLoading.value = false;
    }
  };

  // 刷新 Access Token
  const refreshAccessToken = async (): Promise<boolean> => {
    const currentRefreshToken = refreshTokenValue.value;
    if (!currentRefreshToken) {
      return false;
    }

    try {
      const response = await refreshToken({ refreshToken: currentRefreshToken });
      accessToken.value = response.accessToken;
      refreshTokenValue.value = response.refreshToken;
      safeStorage.setItem(TOKEN_KEY, response.accessToken);
      safeStorage.setItem(REFRESH_TOKEN_KEY, response.refreshToken);
      return true;
    } catch (error) {
      logger.error('[Auth]', '刷新 Token 失败', error);
      return false;
    }
  };

  // 登出
  const logoutUser = async () => {
    try {
      await logout();
    } catch (error) {
      logger.error('[Auth]', '登出失败', error);
    } finally {
      clearAuth();
      ElMessage.success('已退出登录');
    }
  };

  // 设置认证数据
  const setAuthData = (response: AuthResponse) => {
    // 先设置 token，再设置用户信息，确保 isAuthenticated 计算正确
    accessToken.value = response.tokens.accessToken;
    refreshTokenValue.value = response.tokens.refreshToken;
    user.value = response.user;

    // 保存到 localStorage
    safeStorage.setItem(TOKEN_KEY, response.tokens.accessToken);
    safeStorage.setItem(REFRESH_TOKEN_KEY, response.tokens.refreshToken);
    safeStorage.setItem(USER_KEY, JSON.stringify(response.user));

    logger.info('[Auth]', `用户登录成功 | username=${response.user.username}, role=${response.user.role}`);
  };

  // 清除认证数据
  const clearAuth = () => {
    user.value = null;
    accessToken.value = null;
    refreshTokenValue.value = null;
    clearStorage();
  };

  // 清除 localStorage
  const clearStorage = () => {
    safeStorage.removeItem(TOKEN_KEY);
    safeStorage.removeItem(REFRESH_TOKEN_KEY);
    safeStorage.removeItem(USER_KEY);
  };

  // 更新用户信息（用于资料修改后同步）
  const updateUserInfo = (userData: Partial<User>) => {
    if (user.value) {
      user.value = { ...user.value, ...userData };
      safeStorage.setItem(USER_KEY, JSON.stringify(user.value));
    }
  };

  return {
    user,
    accessToken,
    isLoading,
    isAuthChecked,
    isAuthenticated,
    isAdmin,
    isVerified,
    initialize,
    loginUser,
    registerUser,
    refreshAccessToken,
    logoutUser,
    clearAuth,
    setAuthData,
    updateUserInfo
  };
});
