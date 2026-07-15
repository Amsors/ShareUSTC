<template>
  <aside class="filter-sidebar" aria-label="资源筛选">
    <div class="sidebar-heading">
      <h2>筛选与排序</h2>
      <el-button v-if="hasActiveFilter" link type="primary" size="small" @click="resetFilters">
        重置
      </el-button>
    </div>

    <section class="filter-group">
      <div class="group-heading">
        <h3>资源分类</h3>
        <el-button
          v-if="!allCategoriesSelected"
          link
          type="primary"
          size="small"
          @click="selectAllCategories"
        >
          全选
        </el-button>
      </div>
      <button
        v-for="option in categoryOptions"
        :key="option.value"
        type="button"
        class="filter-option"
        :class="{ selected: categories.includes(option.value) }"
        :aria-checked="categories.includes(option.value)"
        role="checkbox"
        @click="toggleCategory(option.value)"
      >
        <span class="check-box" aria-hidden="true">
          <el-icon v-if="categories.includes(option.value)"><Check /></el-icon>
        </span>
        <span>{{ option.label }}({{ categoryCounts[option.value] ?? 0 }})</span>
      </button>
    </section>

    <section class="filter-group">
      <h3>排序</h3>
      <button
        v-for="option in sortOptions"
        :key="option.value"
        type="button"
        class="filter-option"
        :class="{ selected: sortBy === option.value }"
        :aria-checked="sortBy === option.value"
        role="checkbox"
        @click="sortBy = sortBy === option.value ? '' : option.value"
      >
        <span class="check-box" aria-hidden="true">
          <el-icon v-if="sortBy === option.value"><Check /></el-icon>
        </span>
        <span>{{ option.label }}</span>
      </button>
    </section>

    <section class="filter-group">
      <div class="group-heading">
        <h3>资源类型</h3>
        <el-button
          v-if="!allResourceTypesSelected"
          link
          type="primary"
          size="small"
          @click="selectAllResourceTypes"
        >
          全选
        </el-button>
      </div>
      <button
        v-for="option in resourceTypeOptions"
        :key="option.value"
        type="button"
        class="filter-option"
        :class="{ selected: resourceTypes.includes(option.value) }"
        :aria-checked="resourceTypes.includes(option.value)"
        role="checkbox"
        @click="toggleResourceType(option.value)"
      >
        <span class="check-box" aria-hidden="true">
          <el-icon v-if="resourceTypes.includes(option.value)"><Check /></el-icon>
        </span>
        <span>{{ option.label }}</span>
      </button>
    </section>
  </aside>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { Check } from '@element-plus/icons-vue';
import {
  ResourceCategoryLabels,
  ResourceTypeFilterLabels,
  type ResourceCategoryType,
  type ResourceSortField,
  type ResourceTypeFilterType,
} from '@/types/resource';

withDefaults(
  defineProps<{
    categoryCounts?: Partial<Record<ResourceCategoryType, number>>;
  }>(),
  {
    categoryCounts: () => ({}),
  }
);

const resourceTypes = defineModel<ResourceTypeFilterType[]>('resourceTypes', { required: true });
const categories = defineModel<ResourceCategoryType[]>('categories', { required: true });
const sortBy = defineModel<ResourceSortField | ''>('sortBy', { required: true });

const categoryOptions = Object.entries(ResourceCategoryLabels).map(([value, label]) => ({
  value: value as ResourceCategoryType,
  label,
}));
const resourceTypeOptions = Object.entries(ResourceTypeFilterLabels).map(([value, label]) => ({
  value: value as ResourceTypeFilterType,
  label,
}));
const sortOptions: Array<{ label: string; value: ResourceSortField }> = [
  { label: '最新上传', value: 'created_at' },
  { label: '最多下载', value: 'downloads' },
  { label: '最多点赞', value: 'likes' },
  { label: '最高评分', value: 'rating' },
  { label: '标题降序', value: 'title' },
];

const allCategoriesSelected = computed(() => categories.value.length === categoryOptions.length);
const allResourceTypesSelected = computed(
  () => resourceTypes.value.length === resourceTypeOptions.length
);
const hasActiveFilter = computed(
  () =>
    !allCategoriesSelected.value || !allResourceTypesSelected.value || sortBy.value !== 'created_at'
);

const toggleCategory = (value: ResourceCategoryType) => {
  categories.value = categories.value.includes(value)
    ? categories.value.filter((item) => item !== value)
    : [...categories.value, value];
};

const toggleResourceType = (value: ResourceTypeFilterType) => {
  resourceTypes.value = resourceTypes.value.includes(value)
    ? resourceTypes.value.filter((item) => item !== value)
    : [...resourceTypes.value, value];
};

const selectAllCategories = () => {
  categories.value = categoryOptions.map((option) => option.value);
};

const selectAllResourceTypes = () => {
  resourceTypes.value = resourceTypeOptions.map((option) => option.value);
};

const resetFilters = () => {
  selectAllCategories();
  selectAllResourceTypes();
  sortBy.value = 'created_at';
};
</script>

<style scoped>
.filter-sidebar {
  align-self: start;
  padding: var(--su-space-3);
  background-color: var(--el-bg-color);
  border: 1px solid var(--el-border-color-light);
  border-radius: var(--su-radius-md);
}

.sidebar-heading,
.group-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--su-space-1);
}

.sidebar-heading {
  padding-bottom: var(--su-space-2);
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.sidebar-heading h2,
.filter-group h3 {
  margin: 0;
}

.sidebar-heading h2 {
  color: var(--el-text-color-primary);
  font-size: var(--el-font-size-small);
}

.filter-group {
  display: flex;
  flex-direction: column;
  gap: var(--su-space-1);
  padding-top: var(--su-space-2);
}

.filter-group + .filter-group {
  margin-top: var(--su-space-1);
  border-top: 1px solid var(--el-border-color-lighter);
}

.filter-group h3 {
  color: var(--el-text-color-regular);
  font-size: var(--el-font-size-extra-small);
  font-weight: 600;
}

.filter-option {
  display: flex;
  align-items: center;
  gap: var(--su-space-2);
  width: 100%;
  padding: var(--su-space-1) var(--su-space-2);
  color: var(--el-text-color-regular);
  font: inherit;
  font-size: var(--el-font-size-extra-small);
  line-height: 1.2;
  text-align: left;
  background-color: transparent;
  border: 0;
  border-radius: var(--su-radius-sm);
  cursor: pointer;
  transition: background-color var(--su-transition-base);
}

.filter-option:hover,
.filter-option.selected {
  background-color: var(--el-fill-color-light);
}

.filter-option.selected {
  color: var(--el-color-primary);
  font-weight: 500;
}

.check-box {
  display: inline-flex;
  flex: 0 0 var(--su-space-3);
  align-items: center;
  justify-content: center;
  width: var(--su-space-3);
  height: var(--su-space-3);
  color: var(--el-color-white);
  border: 1px solid var(--el-border-color);
  border-radius: var(--su-radius-sm);
}

.selected .check-box {
  background-color: var(--el-color-primary);
  border-color: var(--el-color-primary);
}
</style>
