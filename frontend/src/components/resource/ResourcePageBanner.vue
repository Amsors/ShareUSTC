<template>
  <!-- 资源页品牌介绍横幅：横向布局，风格平实、克制 -->
  <div class="resource-banner">
    <img class="banner-logo" :src="brandConfig.faviconPath" :alt="brandConfig.siteName" />
    <div class="banner-text">
      <div class="banner-heading">
        <h1 class="banner-title">{{ homeConfig.heroTitle }}</h1>
        <span class="banner-subtitle">{{ homeConfig.heroSubtitle }}</span>
      </div>
      <p class="banner-desc">{{ platformConfig.description }}</p>
    </div>
    <div class="banner-count">
      <span class="count-value">{{ total }}</span>
      <span class="count-label">资源总计</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { brandConfig, homeConfig, platformConfig } from '@/config/site.config';

defineProps<{
  // 资源总数
  total: number;
}>();
</script>

<style scoped>
.resource-banner {
  display: flex;
  align-items: center;
  gap: 28px;
  padding: 64px 54px;
  border-radius: 16px;
}

.banner-logo {
  flex-shrink: 0;
  width: 76px;
  height: 76px;
  border-radius: 16px;
  object-fit: cover;
}

.banner-text {
  flex: 1;
  min-width: 0;
}

.banner-heading {
  display: flex;
  align-items: baseline;
  flex-wrap: wrap;
  gap: 14px;
}

.banner-title {
  margin: 0;
  font-size: 34px;
  font-weight: 700;
  letter-spacing: -0.5px;
  color: var(--el-text-color-primary);
}

.banner-subtitle {
  font-size: 16px;
  color: var(--el-text-color-secondary);
}

.banner-desc {
  margin: 10px 0 0;
  font-size: 15px;
  line-height: 1.8;
  color: var(--el-text-color-secondary);
}

/* 资源总计（右侧） */
.banner-count {
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding-left: 28px;
  border-left: 1px solid var(--el-border-color-lighter);
}

.count-value {
  font-size: 32px;
  font-weight: 700;
  line-height: 1.1;
  color: var(--el-color-primary);
}

.count-label {
  font-size: 14px;
  color: var(--el-text-color-secondary);
}

@media (max-width: 768px) {
  .resource-banner {
    gap: 14px;
    padding: 16px;
  }

  .banner-logo {
    width: 48px;
    height: 48px;
  }

  .banner-title {
    font-size: 20px;
  }

  .banner-count {
    padding-left: 14px;
  }

  .count-value {
    font-size: 22px;
  }
}
</style>
