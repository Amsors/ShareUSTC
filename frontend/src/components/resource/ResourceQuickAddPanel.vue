<template>
  <section class="quick-add-panel">
    <div class="quick-add-row">
      <span class="switch-label" :class="{ active: !enabled }">点击查看资源</span>
      <el-switch v-model="enabled" @change="emit('toggle', $event as boolean)" />
      <span class="switch-label" :class="{ active: enabled }">点击加入收藏夹</span>

      <div v-if="enabled" class="favorite-selector">
        <el-select
          v-model="selectedFavoriteId"
          placeholder="选择收藏夹"
          class="favorite-select"
          :disabled="favoriteLocked"
          :loading="favoriteLoading"
        >
          <el-option
            v-for="favorite in favorites"
            :key="favorite.id"
            :label="`${favorite.name} (${favorite.resourceCount})`"
            :value="favorite.id"
          />
        </el-select>

        <el-button
          v-if="!favoriteLocked && selectedFavoriteId"
          type="primary"
          @click="emit('selectFavorite')"
        >
          选择收藏夹
        </el-button>
        <el-button v-if="favoriteLocked" @click="emit('changeFavorite')">重新选择</el-button>
        <el-button
          v-if="favoriteLocked"
          type="warning"
          :loading="batchAddingAll"
          @click="emit('addAll')"
        >
          将本页 {{ resourceCount }} 份资源全部加入收藏夹
        </el-button>
      </div>
    </div>

    <el-alert
      v-if="enabled"
      class="quick-add-hint"
      :title="
        favoriteLocked
          ? '点击任意资源行即可加入收藏夹'
          : '请先选择收藏夹并点击「选择收藏夹」按钮锁定'
      "
      :type="favoriteLocked ? 'success' : 'info'"
      :closable="false"
      show-icon
    />
  </section>
</template>

<script setup lang="ts">
import type { Favorite } from '@/types/favorite';

defineProps<{
  favorites: Favorite[];
  favoriteLoading: boolean;
  favoriteLocked: boolean;
  batchAddingAll: boolean;
  resourceCount: number;
}>();

const emit = defineEmits<{
  toggle: [enabled: boolean];
  selectFavorite: [];
  changeFavorite: [];
  addAll: [];
}>();

const enabled = defineModel<boolean>('enabled', { required: true });
const selectedFavoriteId = defineModel<string>('selectedFavoriteId', { required: true });
</script>

<style scoped>
.quick-add-panel {
  margin-bottom: var(--su-space-4);
  padding: var(--su-space-4);
  background-color: var(--el-bg-color);
  border: 1px solid var(--el-border-color-light);
  border-radius: var(--su-radius-md);
}

.quick-add-row,
.favorite-selector {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--su-space-3);
}

.switch-label {
  color: var(--el-text-color-secondary);
  font-size: var(--el-font-size-small);
  transition: color var(--su-transition-base);
}

.switch-label.active {
  color: var(--el-text-color-primary);
  font-weight: 500;
}

.favorite-select {
  width: 220px;
}

.quick-add-hint {
  margin-top: var(--su-space-3);
}
</style>
