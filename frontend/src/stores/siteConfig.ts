import { ref, computed } from 'vue';
import { defineStore } from 'pinia';
import { getSiteConfig } from '@/api/user';
import type { SiteConfig } from '@/types/user';
import logger from '@/utils/logger';

export const useSiteConfigStore = defineStore('siteConfig', () => {
  // State
  const config = ref<SiteConfig>({
    requireEmailOnRegister: false,
    allowUsernameChange: true,
    allowEmailChange: true,
  });
  const isLoading = ref(false);
  const isLoaded = ref(false);

  // Getters
  const requireEmailOnRegister = computed(() => config.value.requireEmailOnRegister);
  const allowUsernameChange = computed(() => config.value.allowUsernameChange);
  const allowEmailChange = computed(() => config.value.allowEmailChange);

  // Actions
  const loadConfig = async () => {
    // 如果已经加载过，直接返回
    if (isLoaded.value) {
      return config.value;
    }

    isLoading.value = true;
    try {
      const data = await getSiteConfig();
      config.value = data;
      isLoaded.value = true;
      logger.debug('[SiteConfig]', '站点配置加载成功', data);
      return data;
    } catch (error) {
      logger.error('[SiteConfig]', '加载站点配置失败', error);
      // 使用默认值
      return config.value;
    } finally {
      isLoading.value = false;
    }
  };

  return {
    config,
    isLoading,
    isLoaded,
    requireEmailOnRegister,
    allowUsernameChange,
    allowEmailChange,
    loadConfig,
  };
});
