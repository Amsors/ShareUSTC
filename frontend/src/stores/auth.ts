import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { login, register, refreshToken, logout } from '../api/auth';
import { getCurrentUser } from '../api/user';
import { getErrorMessage, isHandledError } from '@/api/request';
import axios from 'axios';
import type { User, LoginRequest, RegisterRequest, AuthResponse } from '../types/auth';
import { UserRole } from '../types/auth';
import { ElMessage } from 'element-plus';
import logger from '../utils/logger';
import { clearDefaultFavoriteStorage } from '../composables/useDefaultFavorite';

export const useAuthStore = defineStore('auth', () => {
  // State
  const user = ref<User | null>(null);
  // accessToken 仅用于内存中的临时存储（如文件下载等场景）
  // 实际认证通过 HttpOnly Cookie 完成
  const accessToken = ref<string | null>(null);
  const isLoading = ref(false);
  const isAuthChecked = ref(false); // 标记是否已完成认证状态检查

  // Getters
  const isAuthenticated = computed(() => !!user.value);
  const isAdmin = computed(() => user.value?.role === UserRole.Admin);
  const isVerified = computed(() => user.value?.isVerified || false);

  // Actions

  // 初始化（验证会话状态并获取用户信息）
  const initialize = async (): Promise<boolean> => {
    // 统一走 src/api/request.ts，并用 skipAuthError 跳过拦截器的自动刷新/跳转，
    // 由本函数手动处理 401 → 刷新 → 重取，避免匿名访问时弹窗和跳转登录页
    try {
      const userData = await getCurrentUser({ skipAuthError: true, timeout: 5000 });
      if (userData) {
        user.value = userData;
        logger.info('[Auth]', `会话验证成功 | username=${userData.username}`);
        isAuthChecked.value = true;
        return true;
      }
    } catch (error) {
      const status = axios.isAxiosError(error) ? error.response?.status : undefined;
      if (status === 401) {
        // Access Token 过期或缺失，尝试静默刷新后重新获取用户信息
        logger.warn('[Auth]', 'Access Token 已过期，尝试刷新...');
        try {
          await refreshToken({ skipAuthError: true, timeout: 5000 });
          const userData = await getCurrentUser({ skipAuthError: true, timeout: 5000 });
          user.value = userData;
          logger.info('[Auth]', `Token 刷新成功 | username=${userData.username}`);
          isAuthChecked.value = true;
          return true;
        } catch {
          logger.warn('[Auth]', 'Token 刷新失败或用户未登录');
        }
      } else {
        logger.warn(
          '[Auth]',
          `会话验证失败 | error=${error instanceof Error ? error.message : error}`
        );
      }
    }

    // 认证失败或未登录，清除状态
    clearAuth();
    isAuthChecked.value = true;
    return false;
  };

  // 登录
  const loginUser = async (credentials: LoginRequest): Promise<boolean> => {
    isLoading.value = true;
    try {
      const response = await login(credentials);
      setAuthData(response);
      ElMessage.success('登录成功');
      return true;
    } catch (error) {
      logger.error('[Auth]', '登录失败', error);
      // 如果错误已经被拦截器处理（显示过消息），则不再显示
      if (!isHandledError(error)) {
        ElMessage.error(getErrorMessage(error, '登录失败'));
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
    } catch (error) {
      logger.error('[Auth]', '注册失败', error);
      // 如果错误已经被拦截器处理（显示过消息），则不再显示
      if (!isHandledError(error)) {
        ElMessage.error(getErrorMessage(error, '注册失败'));
      }
      return false;
    } finally {
      isLoading.value = false;
    }
  };

  // 刷新 Access Token
  const refreshAccessToken = async (): Promise<boolean> => {
    try {
      // 后端从 HttpOnly Cookie 中读取 refresh_token
      // skipAuthError：刷新失败保持静默，由调用方按返回值决定跳转/提示，避免重复弹窗
      await refreshToken({ skipAuthError: true });
      logger.info('[Auth]', 'Token 刷新成功');
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
  // 注意：AuthResponse 类型已定义为 User 的别名，API 直接返回用户信息
  const setAuthData = (response: AuthResponse) => {
    // 设置用户信息
    user.value = response;

    // 注意：Token 存储在 HttpOnly Cookie 中，由浏览器自动管理
    // 前端不直接访问 Token

    logger.info('[Auth]', `用户登录成功 | username=${response.username}, role=${response.role}`);
  };

  // 清除认证数据
  const clearAuth = () => {
    user.value = null;
    accessToken.value = null;
    // 清除默认收藏夹设置
    clearDefaultFavoriteStorage();
  };

  // 更新用户信息（用于资料修改后同步）
  const updateUserInfo = (userData: Partial<User>) => {
    if (user.value) {
      user.value = { ...user.value, ...userData };
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
    updateUserInfo,
  };
});
