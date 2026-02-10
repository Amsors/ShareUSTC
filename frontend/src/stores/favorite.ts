import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { Favorite, FavoriteDetail, FavoriteListResponse } from '../types/favorite';
import * as favoriteApi from '../api/favorite';

export const useFavoriteStore = defineStore('favorite', () => {
  // State
  const favorites = ref<Favorite[]>([]);
  const currentFavorite = ref<FavoriteDetail | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

  // Getters
  const favoriteCount = computed(() => favorites.value.length);
  const hasFavorites = computed(() => favorites.value.length > 0);

  // Actions
  /**
   * 获取收藏夹列表
   */
  const fetchFavorites = async () => {
    loading.value = true;
    error.value = null;
    try {
      const response = await favoriteApi.getFavorites();
      favorites.value = response.favorites;
      return response;
    } catch (err) {
      error.value = '获取收藏夹列表失败';
      throw err;
    } finally {
      loading.value = false;
    }
  };

  /**
   * 获取收藏夹详情
   */
  const fetchFavoriteDetail = async (favoriteId: string) => {
    loading.value = true;
    error.value = null;
    try {
      const detail = await favoriteApi.getFavoriteDetail(favoriteId);
      currentFavorite.value = detail;
      return detail;
    } catch (err) {
      error.value = '获取收藏夹详情失败';
      throw err;
    } finally {
      loading.value = false;
    }
  };

  /**
   * 创建收藏夹
   */
  const createFavorite = async (name: string) => {
    loading.value = true;
    try {
      const response = await favoriteApi.createFavorite({ name });
      // 重新获取列表以更新状态
      await fetchFavorites();
      return response;
    } catch (err) {
      error.value = '创建收藏夹失败';
      throw err;
    } finally {
      loading.value = false;
    }
  };

  /**
   * 更新收藏夹
   */
  const updateFavorite = async (favoriteId: string, name: string) => {
    loading.value = true;
    try {
      await favoriteApi.updateFavorite(favoriteId, { name });
      // 更新本地状态
      const index = favorites.value.findIndex(f => f.id === favoriteId);
      if (index !== -1) {
        favorites.value[index].name = name;
      }
      // 如果当前查看的收藏夹被更新，也更新它
      if (currentFavorite.value?.id === favoriteId) {
        currentFavorite.value.name = name;
      }
    } catch (err) {
      error.value = '更新收藏夹失败';
      throw err;
    } finally {
      loading.value = false;
    }
  };

  /**
   * 删除收藏夹
   */
  const deleteFavorite = async (favoriteId: string) => {
    loading.value = true;
    try {
      await favoriteApi.deleteFavorite(favoriteId);
      // 从本地列表中移除
      favorites.value = favorites.value.filter(f => f.id !== favoriteId);
      // 如果当前查看的收藏夹被删除，清空它
      if (currentFavorite.value?.id === favoriteId) {
        currentFavorite.value = null;
      }
    } catch (err) {
      error.value = '删除收藏夹失败';
      throw err;
    } finally {
      loading.value = false;
    }
  };

  /**
   * 添加资源到收藏夹
   */
  const addResourceToFavorite = async (favoriteId: string, resourceId: string) => {
    try {
      await favoriteApi.addToFavorite(favoriteId, { resourceId });
      // 更新本地收藏夹计数
      const favorite = favorites.value.find(f => f.id === favoriteId);
      if (favorite) {
        favorite.resourceCount++;
      }
      // 如果当前查看的是这个收藏夹，刷新详情
      if (currentFavorite.value?.id === favoriteId) {
        await fetchFavoriteDetail(favoriteId);
      }
    } catch (err) {
      throw err;
    }
  };

  /**
   * 从收藏夹移除资源
   */
  const removeResourceFromFavorite = async (favoriteId: string, resourceId: string) => {
    try {
      await favoriteApi.removeFromFavorite(favoriteId, resourceId);
      // 更新本地收藏夹计数
      const favorite = favorites.value.find(f => f.id === favoriteId);
      if (favorite) {
        favorite.resourceCount = Math.max(0, favorite.resourceCount - 1);
      }
      // 如果当前查看的是这个收藏夹，从列表中移除
      if (currentFavorite.value?.id === favoriteId) {
        currentFavorite.value.resources = currentFavorite.value.resources.filter(
          r => r.id !== resourceId
        );
        currentFavorite.value.resourceCount = currentFavorite.value.resources.length;
      }
    } catch (err) {
      throw err;
    }
  };

  /**
   * 检查资源是否在收藏夹中
   */
  const checkResourceFavoriteStatus = async (resourceId: string) => {
    try {
      const response = await favoriteApi.checkResourceInFavorite(resourceId);
      return response;
    } catch (err) {
      return { inFavorites: [], isFavorited: false };
    }
  };

  /**
   * 清空当前收藏夹
   */
  const clearCurrentFavorite = () => {
    currentFavorite.value = null;
  };

  /**
   * 清空错误信息
   */
  const clearError = () => {
    error.value = null;
  };

  return {
    // State
    favorites,
    currentFavorite,
    loading,
    error,
    // Getters
    favoriteCount,
    hasFavorites,
    // Actions
    fetchFavorites,
    fetchFavoriteDetail,
    createFavorite,
    updateFavorite,
    deleteFavorite,
    addResourceToFavorite,
    removeResourceFromFavorite,
    checkResourceFavoriteStatus,
    clearCurrentFavorite,
    clearError,
  };
});
