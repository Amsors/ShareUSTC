import { ref, computed, watch } from 'vue';
import { useAuthStore } from '../stores/auth';
import { storeToRefs } from 'pinia';

const DEFAULT_FAVORITE_KEY = 'default_favorite';

interface DefaultFavoriteData {
  userId: string;
  favoriteId: string;
  favoriteName: string;
}

/**
 * 默认收藏夹管理 Composable
 * 用于管理用户设置的默认收藏夹，数据存储在 localStorage 中
 */
export function useDefaultFavorite() {
  const authStore = useAuthStore();
  const { user } = storeToRefs(authStore);

  // 当前用户的默认收藏夹信息
  const defaultFavorite = ref<DefaultFavoriteData | null>(null);

  // 是否有默认收藏夹
  const hasDefaultFavorite = computed(() => {
    if (!user.value) return false;
    return defaultFavorite.value?.userId === user.value.id;
  });

  // 获取默认收藏夹ID
  const defaultFavoriteId = computed(() => {
    if (!user.value || !defaultFavorite.value) return null;
    if (defaultFavorite.value.userId !== user.value.id) return null;
    return defaultFavorite.value.favoriteId;
  });

  // 获取默认收藏夹名称
  const defaultFavoriteName = computed(() => {
    if (!user.value || !defaultFavorite.value) return null;
    if (defaultFavorite.value.userId !== user.value.id) return null;
    return defaultFavorite.value.favoriteName;
  });

  /**
   * 从 localStorage 加载默认收藏夹信息
   */
  const loadDefaultFavorite = () => {
    if (!user.value) {
      defaultFavorite.value = null;
      return;
    }

    try {
      const stored = localStorage.getItem(DEFAULT_FAVORITE_KEY);
      if (stored) {
        const data: DefaultFavoriteData = JSON.parse(stored);
        // 验证数据是否属于当前用户
        if (data.userId === user.value.id) {
          defaultFavorite.value = data;
        } else {
          // 数据属于其他用户，清理掉
          localStorage.removeItem(DEFAULT_FAVORITE_KEY);
          defaultFavorite.value = null;
        }
      } else {
        defaultFavorite.value = null;
      }
    } catch {
      defaultFavorite.value = null;
    }
  };

  /**
   * 设置默认收藏夹
   * @param favoriteId 收藏夹ID
   * @param favoriteName 收藏夹名称
   */
  const setDefaultFavorite = (favoriteId: string, favoriteName: string) => {
    if (!user.value) return;

    // 如果 favoriteId 为空，则清除默认收藏夹
    if (!favoriteId) {
      clearDefaultFavorite();
      return;
    }

    const data: DefaultFavoriteData = {
      userId: user.value.id,
      favoriteId,
      favoriteName,
    };

    localStorage.setItem(DEFAULT_FAVORITE_KEY, JSON.stringify(data));
    defaultFavorite.value = data;
  };

  /**
   * 清除默认收藏夹设置
   */
  const clearDefaultFavorite = () => {
    localStorage.removeItem(DEFAULT_FAVORITE_KEY);
    defaultFavorite.value = null;
  };

  /**
   * 检查指定收藏夹是否是默认收藏夹
   * @param favoriteId 收藏夹ID
   */
  const isDefaultFavorite = (favoriteId: string): boolean => {
    if (!user.value || !defaultFavorite.value) return false;
    return (
      defaultFavorite.value.userId === user.value.id &&
      defaultFavorite.value.favoriteId === favoriteId
    );
  };

  // 监听用户变化，自动加载或清理默认收藏夹信息
  watch(
    () => user.value?.id,
    (newUserId, oldUserId) => {
      if (newUserId !== oldUserId) {
        loadDefaultFavorite();
      }
    },
    { immediate: true }
  );

  return {
    defaultFavorite,
    hasDefaultFavorite,
    defaultFavoriteId,
    defaultFavoriteName,
    setDefaultFavorite,
    clearDefaultFavorite,
    isDefaultFavorite,
    loadDefaultFavorite,
  };
}

/**
 * 清除默认收藏夹（用于登出时调用）
 */
export const clearDefaultFavoriteStorage = () => {
  localStorage.removeItem(DEFAULT_FAVORITE_KEY);
};
