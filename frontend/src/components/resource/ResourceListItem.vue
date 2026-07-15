<template>
  <a
    :href="`/resources/${resource.id}`"
    class="resource-row"
    :class="{ 'quick-add-mode': quickAddMode, adding }"
    @click.prevent="emit('select', resource)"
  >
    <div v-if="adding" class="adding-overlay" aria-label="正在加入收藏夹">
      <el-icon class="spinning"><Loading /></el-icon>
    </div>

    <div class="file-icon" aria-hidden="true">
      <el-icon><Document /></el-icon>
    </div>

    <div class="resource-main">
      <h3 class="resource-title">{{ resource.title }}</h3>
      <div class="resource-description">
        <span v-if="resource.courseName" class="course-name">
          <el-icon><Reading /></el-icon>
          {{ resource.courseName }}
        </span>
        <span v-else class="course-name muted">未关联课程</span>
        <span v-if="resource.tags?.length" class="tag-summary">
          {{ resource.tags.slice(0, 3).join(' · ') }}
          <template v-if="resource.tags.length > 3">
            等 {{ resource.tags.length }} 个标签
          </template>
        </span>
      </div>
    </div>

    <div class="resource-classification">
      <el-tag size="small" :type="resourceTypeTagType">{{ resourceTypeLabel }}</el-tag>
      <el-tag size="small" type="info">{{ categoryLabel }}</el-tag>
    </div>

    <div class="resource-stats" aria-label="资源统计">
      <span title="浏览量"
        ><el-icon><View /></el-icon>{{ resource.stats.views }}</span
      >
      <span title="下载量"
        ><el-icon><Download /></el-icon>{{ resource.stats.downloads }}</span
      >
      <span title="点赞量"
        ><el-icon><Star /></el-icon>{{ resource.stats.likes }}</span
      >
    </div>

    <div class="resource-owner">
      <span>{{ resource.uploaderName || '未知用户' }}</span>
      <time :datetime="resource.createdAt">{{ formatTime(resource.createdAt) }}</time>
    </div>
  </a>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { TagProps } from 'element-plus';
import { Document, Download, Loading, Reading, Star, View } from '@element-plus/icons-vue';
import {
  ResourceCategoryLabels,
  ResourceTypeLabels,
  type ResourceCategoryType,
  type ResourceListItem,
  type ResourceTypeType,
} from '@/types/resource';

const props = defineProps<{
  resource: ResourceListItem;
  quickAddMode: boolean;
  adding: boolean;
}>();

const emit = defineEmits<{
  select: [resource: ResourceListItem];
}>();

const resourceTypeLabel = computed(
  () =>
    ResourceTypeLabels[props.resource.resourceType as ResourceTypeType] ||
    props.resource.resourceType
);
const categoryLabel = computed(
  () =>
    ResourceCategoryLabels[props.resource.category as ResourceCategoryType] ||
    props.resource.category
);

const resourceTypeTagType = computed<TagProps['type']>(() => {
  const typeMap: Record<string, TagProps['type']> = {
    pdf: 'danger',
    ppt: 'warning',
    pptx: 'warning',
    doc: 'primary',
    docx: 'primary',
    web_markdown: 'success',
    zip: 'info',
  };
  return typeMap[props.resource.resourceType] || 'info';
});

// 将后端返回的 UTC 时间转为本地相对时间
const formatTime = (time: string): string => {
  const date = new Date(time.endsWith('Z') ? time : `${time}Z`);
  const diff = Date.now() - date.getTime();

  if (diff < 60 * 60 * 1000) {
    const minutes = Math.floor(diff / (60 * 1000));
    return minutes < 1 ? '刚刚' : `${minutes}分钟前`;
  }
  if (diff < 24 * 60 * 60 * 1000) return `${Math.floor(diff / (60 * 60 * 1000))}小时前`;
  if (diff < 7 * 24 * 60 * 60 * 1000) return `${Math.floor(diff / (24 * 60 * 60 * 1000))}天前`;

  return date.toLocaleDateString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit' });
};
</script>

<style scoped>
.resource-row {
  position: relative;
  display: grid;
  grid-template-columns: auto minmax(240px, 1fr) minmax(140px, auto) auto minmax(96px, auto);
  gap: var(--su-space-4);
  align-items: center;
  min-height: 82px;
  padding: var(--su-space-3) var(--su-space-4);
  color: inherit;
  text-decoration: none;
  cursor: pointer;
  transition: background-color var(--su-transition-base);
}

.resource-row + .resource-row {
  border-top: 1px solid var(--el-border-color-lighter);
}

.resource-row:hover {
  background-color: var(--el-fill-color-light);
}

.resource-row.quick-add-mode:hover {
  background-color: transparent;
  outline: 2px solid var(--el-color-success);
  outline-offset: -2px;
}

.resource-row.adding {
  pointer-events: none;
}

.adding-overlay {
  position: absolute;
  z-index: 2;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--el-color-primary);
  background-color: var(--el-mask-color);
}

.spinning {
  font-size: var(--el-font-size-extra-large);
  animation: spin 1s linear infinite;
}

.file-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: var(--su-space-10);
  height: var(--su-space-10);
  color: var(--el-color-primary);
  font-size: var(--el-font-size-extra-large);
  background-color: var(--el-color-primary-light-9);
  border-radius: var(--su-radius-md);
}

.resource-main {
  min-width: 0;
}

.resource-title {
  overflow: hidden;
  margin: 0;
  color: var(--el-text-color-primary);
  font-size: var(--el-font-size-base);
  font-weight: 600;
  line-height: 1.5;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.resource-description,
.course-name,
.resource-stats,
.resource-owner {
  display: flex;
  align-items: center;
}

.resource-description {
  gap: var(--su-space-3);
  min-width: 0;
  margin-top: var(--su-space-1);
  color: var(--el-text-color-secondary);
  font-size: var(--el-font-size-extra-small);
}

.course-name {
  flex: 0 1 auto;
  gap: var(--su-space-1);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.muted,
.tag-summary {
  color: var(--el-text-color-placeholder);
}

.tag-summary {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.resource-classification {
  display: flex;
  flex-wrap: wrap;
  gap: var(--su-space-1);
}

.resource-stats {
  gap: var(--su-space-3);
  color: var(--el-text-color-secondary);
  font-size: var(--el-font-size-extra-small);
}

.resource-stats span {
  display: inline-flex;
  align-items: center;
  gap: var(--su-space-1);
}

.resource-owner {
  flex-direction: column;
  align-items: flex-end;
  gap: var(--su-space-1);
  color: var(--el-text-color-secondary);
  font-size: var(--el-font-size-extra-small);
  white-space: nowrap;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

@media (max-width: 1100px) {
  .resource-row {
    grid-template-columns: auto minmax(180px, 1fr) auto auto;
  }

  .resource-classification {
    display: none;
  }
}

@media (max-width: 768px) {
  .resource-row {
    grid-template-columns: auto minmax(0, 1fr) auto;
  }

  .resource-stats {
    display: none;
  }

  .tag-summary,
  .resource-owner span {
    display: none;
  }
}
</style>
