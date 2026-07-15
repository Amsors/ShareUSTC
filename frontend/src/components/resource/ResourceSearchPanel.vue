<template>
  <el-card class="search-panel" shadow="never">
    <div class="search-controls">
      <el-input
        v-model="searchQuery"
        placeholder="搜索资源标题或课程名称"
        class="search-input"
        size="large"
        clearable
        @keyup.enter="emit('search')"
        @clear="emit('clear')"
      />

      <el-select
        v-model="courseSns"
        aria-label="课程筛选"
        placeholder="选择课程（可搜索、多选）"
        clearable
        multiple
        filterable
        class="relation-filter"
        size="large"
        :disabled="loading || loadingCourses"
        :loading="loadingCourses"
      >
        <template #prefix>
          <el-icon><Reading /></el-icon>
        </template>
        <template #tag>
          <span v-if="courseSns.length > 0" class="selection-summary">
            已选 {{ courseSns.length }} 门课程
          </span>
        </template>
        <el-option
          v-for="course in courses"
          :key="course.sn"
          :label="course.name + (course.semester ? ` (${course.semester})` : '')"
          :value="course.sn"
        >
          <div class="option-content">
            <span class="option-name">{{ course.name }}</span>
            <span v-if="course.semester" class="option-meta">{{ course.semester }}</span>
          </div>
        </el-option>
      </el-select>

      <el-select
        v-model="teacherSns"
        aria-label="教师筛选"
        placeholder="选择教师（可搜索、多选）"
        clearable
        multiple
        filterable
        class="relation-filter"
        size="large"
        :disabled="loading || loadingTeachers"
        :loading="loadingTeachers"
      >
        <template #prefix>
          <el-icon><User /></el-icon>
        </template>
        <template #tag>
          <span v-if="teacherSns.length > 0" class="selection-summary">
            已选 {{ teacherSns.length }} 位教师
          </span>
        </template>
        <el-option
          v-for="teacher in teachers"
          :key="teacher.sn"
          :label="teacher.name + (teacher.department ? ` (${teacher.department})` : '')"
          :value="teacher.sn"
        >
          <div class="option-content">
            <span class="option-name">{{ teacher.name }}</span>
            <span v-if="teacher.department" class="option-meta">{{ teacher.department }}</span>
          </div>
        </el-option>
      </el-select>

      <el-button
        type="primary"
        size="large"
        class="search-button"
        :icon="Search"
        @click="emit('search')"
      >
        搜索
      </el-button>
    </div>

    <section
      v-if="selectedFilterCount > 0"
      class="selected-filters"
      aria-label="已选课程与教师"
      aria-live="polite"
    >
      <div class="selected-filter-header">
        <div class="selected-filter-heading">
          <div class="selected-filter-title">
            <strong>已选筛选</strong>
            <span v-if="selectedFilterCount > 0" class="selected-filter-count">
              {{ selectedFilterCount }} 项
            </span>
          </div>
          <span v-if="hasPendingRelationChanges" class="pending-hint">
            <el-icon><WarningFilled /></el-icon>
            筛选条件已修改，点击搜索后生效
          </span>
        </div>
        <el-button
          v-if="selectedFilterCount > 0"
          link
          type="primary"
          class="clear-all-button"
          @click="clearAllRelations"
        >
          清空全部
        </el-button>
      </div>

      <div v-if="selectedCourses.length > 0" class="selected-filter-group">
        <span class="selected-filter-label">
          <el-icon><Reading /></el-icon>
          课程
        </span>
        <div class="selected-filter-tags">
          <el-tag
            v-for="course in visibleSelectedCourses"
            :key="course.sn"
            :title="course.name + (course.semester ? ` · ${course.semester}` : '')"
            closable
            effect="plain"
            class="filter-tag"
            @close="removeCourse(course.sn)"
          >
            {{ course.name
            }}<span v-if="course.semester" class="tag-meta"> · {{ course.semester }}</span>
          </el-tag>
        </div>
      </div>

      <div v-if="selectedTeachers.length > 0" class="selected-filter-group">
        <span class="selected-filter-label">
          <el-icon><User /></el-icon>
          教师
        </span>
        <div class="selected-filter-tags">
          <el-tag
            v-for="teacher in visibleSelectedTeachers"
            :key="teacher.sn"
            :title="teacher.name + (teacher.department ? ` · ${teacher.department}` : '')"
            closable
            effect="plain"
            class="filter-tag"
            @close="removeTeacher(teacher.sn)"
          >
            {{ teacher.name
            }}<span v-if="teacher.department" class="tag-meta"> · {{ teacher.department }}</span>
          </el-tag>
        </div>
      </div>

      <el-button
        v-if="hiddenFilterCount > 0"
        link
        type="primary"
        class="expand-button"
        @click="filtersExpanded = true"
      >
        展开全部（还有 {{ hiddenFilterCount }} 项）
      </el-button>
      <el-button
        v-else-if="filtersExpanded && selectedFilterCount > MAX_VISIBLE_FILTERS_PER_GROUP"
        link
        type="primary"
        class="expand-button"
        @click="filtersExpanded = false"
      >
        收起
      </el-button>
    </section>
  </el-card>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { Reading, Search, User, WarningFilled } from '@element-plus/icons-vue';
import type { Course } from '@/types/course';
import type { Teacher } from '@/types/teacher';

const MAX_VISIBLE_FILTERS_PER_GROUP = 6;

const props = defineProps<{
  courses: Course[];
  teachers: Teacher[];
  loading: boolean;
  loadingCourses: boolean;
  loadingTeachers: boolean;
  hasPendingRelationChanges: boolean;
}>();

const emit = defineEmits<{
  search: [];
  clear: [];
}>();

const searchQuery = defineModel<string>('searchQuery', { required: true });
const courseSns = defineModel<number[]>('courseSns', { required: true });
const teacherSns = defineModel<number[]>('teacherSns', { required: true });
const filtersExpanded = ref(false);

const selectedCourses = computed(() => {
  const courseMap = new Map(props.courses.map((course) => [course.sn, course]));
  return courseSns.value
    .map((sn) => courseMap.get(sn))
    .filter((course): course is Course => course !== undefined);
});

const selectedTeachers = computed(() => {
  const teacherMap = new Map(props.teachers.map((teacher) => [teacher.sn, teacher]));
  return teacherSns.value
    .map((sn) => teacherMap.get(sn))
    .filter((teacher): teacher is Teacher => teacher !== undefined);
});

const selectedFilterCount = computed(() => courseSns.value.length + teacherSns.value.length);
const visibleSelectedCourses = computed(() =>
  filtersExpanded.value
    ? selectedCourses.value
    : selectedCourses.value.slice(0, MAX_VISIBLE_FILTERS_PER_GROUP)
);
const visibleSelectedTeachers = computed(() =>
  filtersExpanded.value
    ? selectedTeachers.value
    : selectedTeachers.value.slice(0, MAX_VISIBLE_FILTERS_PER_GROUP)
);
const hiddenFilterCount = computed(
  () =>
    selectedCourses.value.length -
    visibleSelectedCourses.value.length +
    selectedTeachers.value.length -
    visibleSelectedTeachers.value.length
);

const removeCourse = (sn: number) => {
  courseSns.value = courseSns.value.filter((courseSn) => courseSn !== sn);
};

const removeTeacher = (sn: number) => {
  teacherSns.value = teacherSns.value.filter((teacherSn) => teacherSn !== sn);
};

const clearAllRelations = () => {
  courseSns.value = [];
  teacherSns.value = [];
  filtersExpanded.value = false;
};
</script>

<style scoped>
.search-panel {
  margin-bottom: var(--su-space-5);
  border-color: var(--el-border-color-light);
}

.search-panel :deep(.el-card__body) {
  padding: var(--su-space-5);
}

.search-controls {
  display: grid;
  grid-template-columns: minmax(0, 2fr) repeat(2, minmax(0, 1fr)) auto;
  gap: var(--su-space-4);
  align-items: stretch;
}

.search-input,
.relation-filter {
  width: 100%;
}

.search-input :deep(.el-input__wrapper) {
  font-size: var(--el-font-size-medium);
}

.selection-summary {
  min-width: 0;
  overflow: hidden;
  color: var(--el-text-color-regular);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.option-content {
  display: flex;
  min-width: 0;
  align-items: center;
  justify-content: space-between;
  gap: var(--su-space-3);
}

.option-name {
  overflow: hidden;
  color: var(--el-text-color-primary);
  font-weight: 500;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.option-meta {
  flex: none;
  color: var(--el-text-color-secondary);
  font-size: var(--el-font-size-small);
}

.search-button {
  height: 100%;
  padding-inline: var(--su-space-8);
}

.selected-filters {
  display: grid;
  margin-top: var(--su-space-4);
  padding: var(--su-space-4);
  background-color: var(--el-fill-color-light);
  border-radius: var(--su-radius-md);
  gap: var(--su-space-3);
}

.selected-filter-header,
.selected-filter-title,
.pending-hint,
.selected-filter-label,
.selected-filter-tags {
  display: flex;
  align-items: center;
}

.selected-filter-header {
  justify-content: space-between;
  gap: var(--su-space-3);
}

.selected-filter-heading {
  display: flex;
  min-width: 0;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--su-space-2) var(--su-space-4);
}

.selected-filter-title {
  flex: none;
  gap: var(--su-space-2);
  color: var(--el-text-color-primary);
  font-size: var(--el-font-size-base);
}

.selected-filter-count,
.pending-hint,
.selected-filter-label {
  font-size: var(--el-font-size-small);
}

.selected-filter-count,
.selected-filter-label {
  color: var(--el-text-color-secondary);
}

.pending-hint {
  gap: var(--su-space-1);
  color: var(--el-color-warning-dark-2);
}

.clear-all-button {
  flex: none;
}

.selected-filter-group {
  display: grid;
  grid-template-columns: max-content minmax(0, 1fr);
  align-items: start;
  gap: var(--su-space-3);
}

.selected-filter-label {
  flex: 0 0 auto;
  gap: var(--su-space-1);
  min-height: var(--el-component-size-small);
}

.selected-filter-tags {
  min-width: 0;
  flex-wrap: wrap;
  gap: var(--su-space-2);
}

.filter-tag {
  max-width: 100%;
}

.filter-tag :deep(.el-tag__content) {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tag-meta {
  color: var(--el-text-color-secondary);
}

.expand-button {
  margin-top: var(--su-space-2);
  margin-left: 0;
}

@media (max-width: 900px) {
  .search-controls {
    grid-template-columns: repeat(2, minmax(0, 1fr)) auto;
  }

  .search-input {
    grid-column: 1 / -1;
  }
}

@media (max-width: 768px) {
  .search-controls {
    grid-template-columns: 1fr;
  }

  .search-input {
    grid-column: auto;
  }

  .search-button {
    width: 100%;
  }

  .selected-filter-header {
    align-items: flex-start;
  }
}
</style>
