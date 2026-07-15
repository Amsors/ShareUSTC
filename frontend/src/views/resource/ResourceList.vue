<template>
  <div class="resource-list-page">
    <!-- 首访时依次展示全站指南和资源页指南 -->
    <UserGuideModal ref="userGuideModalRef" @closed="handleUserGuideClosed" />
    <ResourceGuideModal ref="resourceGuideModalRef" />

    <transition name="banner-collapse">
      <div v-if="!hasSearched" class="banner-wrapper">
        <div class="banner-content">
          <ResourcePageBanner :total="totalResourceCount" />
        </div>
      </div>
    </transition>

    <ResourceSearchPanel
      v-model:search-query="searchQuery"
      v-model:course-sns="filterCourseSns"
      v-model:teacher-sns="filterTeacherSns"
      :courses="courseList"
      :teachers="teacherList"
      :loading="loading"
      :loading-courses="loadingCourses"
      :loading-teachers="loadingTeachers"
      :has-pending-relation-changes="hasPendingRelationChanges"
      @search="handleSearch"
      @clear="handleClearSearch"
    />

    <div class="resource-workspace" :class="{ 'resource-workspace--filtered': hasSearched }">
      <transition name="filter-sidebar">
        <ResourceFilterSidebar
          v-if="hasSearched"
          v-model:resource-types="filterTypes"
          v-model:categories="filterCategories"
          v-model:sort-by="sortBy"
          :category-counts="categoryCounts"
        />
      </transition>

      <main class="resource-pane">
        <transition name="fade" mode="out-in">
          <div v-if="!hasSearched" key="landing" class="landing-actions">
            <button type="button" class="action-card" @click="goToUpload">
              <span class="action-icon"
                ><el-icon><Upload /></el-icon
              ></span>
              <span class="action-text">
                <strong>上传资源</strong>
                <small>分享课程笔记、往年试卷和复习资料</small>
              </span>
              <el-icon class="action-arrow"><ArrowRight /></el-icon>
            </button>

            <button type="button" class="action-card" @click="goToAccount">
              <span class="action-icon"
                ><el-icon><User /></el-icon
              ></span>
              <span class="action-text">
                <strong>{{ authStore.isAuthenticated ? '个人中心' : '注册 / 登录' }}</strong>
                <small>
                  {{
                    authStore.isAuthenticated
                      ? '查看个人资料、我的资源与账号信息'
                      : '登录后可收藏、上传与管理学习资源'
                  }}
                </small>
              </span>
              <el-icon class="action-arrow"><ArrowRight /></el-icon>
            </button>
          </div>

          <section v-else key="results" aria-label="资源检索结果">
            <ResourceQuickAddPanel
              v-if="authStore.isAuthenticated"
              v-model:enabled="enableQuickAdd"
              v-model:selected-favorite-id="selectedFavoriteId"
              :favorites="favoritesWithCount"
              :favorite-loading="favoriteStore.loading"
              :favorite-locked="favoriteLocked"
              :batch-adding-all="batchAddingAll"
              :resource-count="resources.length"
              @toggle="handleQuickAddToggle"
              @select-favorite="handleSelectFavorite"
              @change-favorite="handleChangeFavorite"
              @add-all="handleAddAllCurrentPage"
            />

            <div class="result-heading">
              <div>
                <h1>资源列表</h1>
                <p>共 {{ total }} 份匹配资源</p>
              </div>
              <span class="view-mode"
                ><el-icon><List /></el-icon>列表视图</span
              >
            </div>

            <div v-if="loading" class="resource-loading" aria-live="polite">
              <el-icon class="loading-spinner"><Loading /></el-icon>
              <p>加载中...</p>
            </div>

            <div v-else-if="resources.length === 0" class="resource-empty">
              <el-empty description="没有找到匹配的资源，试试调整关键词或筛选条件" />
            </div>

            <div v-else class="resource-list">
              <ResourceListItem
                v-for="resource in resources"
                :key="resource.id"
                :resource="resource"
                :quick-add-mode="enableQuickAdd"
                :adding="addingResourceId === resource.id"
                @select="handleResourceItemClick"
              />
            </div>

            <div v-if="total > 0" class="pagination-container">
              <el-pagination
                v-model:current-page="currentPage"
                v-model:page-size="pageSize"
                :page-sizes="RESOURCE_PAGE_SIZES"
                :total="total"
                layout="total, sizes, prev, pager, next"
                @size-change="handleSizeChange"
                @current-change="handlePageChange"
              />
            </div>
          </section>
        </transition>
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue';
import { ArrowRight, List, Loading, Upload, User } from '@element-plus/icons-vue';
import ResourceGuideModal from '@/components/common/ResourceGuideModal.vue';
import UserGuideModal from '@/components/common/UserGuideModal.vue';
import ResourceFilterSidebar from '@/components/resource/ResourceFilterSidebar.vue';
import ResourceListItem from '@/components/resource/ResourceListItem.vue';
import ResourcePageBanner from '@/components/resource/ResourcePageBanner.vue';
import ResourceQuickAddPanel from '@/components/resource/ResourceQuickAddPanel.vue';
import ResourceSearchPanel from '@/components/resource/ResourceSearchPanel.vue';
import { RESOURCE_PAGE_SIZES, useResourceListPage } from '@/composables/useResourceListPage';

const userGuideModalRef = ref<InstanceType<typeof UserGuideModal> | null>(null);
const resourceGuideModalRef = ref<InstanceType<typeof ResourceGuideModal> | null>(null);
let guideTimer: ReturnType<typeof setTimeout> | undefined;

const {
  authStore,
  favoriteStore,
  loading,
  resources,
  total,
  categoryCounts,
  currentPage,
  pageSize,
  hasSearched,
  totalResourceCount,
  searchQuery,
  filterTypes,
  filterCategories,
  sortBy,
  filterTeacherSns,
  filterCourseSns,
  teacherList,
  courseList,
  loadingTeachers,
  loadingCourses,
  hasPendingRelationChanges,
  enableQuickAdd,
  selectedFavoriteId,
  favoriteLocked,
  addingResourceId,
  batchAddingAll,
  favoritesWithCount,
  handleSearch,
  handleClearSearch,
  handleSizeChange,
  handlePageChange,
  handleSelectFavorite,
  handleChangeFavorite,
  handleQuickAddToggle,
  handleAddAllCurrentPage,
  handleResourceItemClick,
  goToUpload,
  goToAccount,
  initialize,
} = useResourceListPage();

const handleUserGuideClosed = () => resourceGuideModalRef.value?.show();

onMounted(() => {
  initialize();
  guideTimer = setTimeout(() => {
    const opened = userGuideModalRef.value?.show();
    if (opened !== true) resourceGuideModalRef.value?.show();
  }, 500);
});

onBeforeUnmount(() => {
  if (guideTimer) clearTimeout(guideTimer);
});
</script>

<style scoped>
.resource-list-page {
  max-width: 1400px;
  margin: 0 auto;
  padding: var(--su-space-6);
}

.banner-wrapper {
  display: grid;
  grid-template-rows: 1fr;
  margin-inline: -72px;
  margin-bottom: var(--su-space-3);
}

.banner-content {
  min-height: 0;
  overflow: hidden;
}

.banner-collapse-enter-active,
.banner-collapse-leave-active {
  transition:
    opacity var(--su-transition-slow),
    grid-template-rows var(--su-transition-slow),
    margin-bottom var(--su-transition-slow);
}

.banner-collapse-enter-from,
.banner-collapse-leave-to {
  grid-template-rows: 0fr;
  margin-bottom: 0;
  opacity: 0;
}

.resource-workspace {
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  gap: var(--su-space-5);
  align-items: start;
}

.resource-workspace--filtered {
  grid-template-columns: 230px minmax(0, 1fr);
}

.filter-sidebar-enter-active {
  transition:
    opacity var(--su-transition-slow),
    transform var(--su-transition-slow);
}

.filter-sidebar-enter-from {
  opacity: 0;
  transform: translateY(-2px);
}

.resource-pane {
  min-width: 0;
}

.landing-actions {
  display: flex;
  flex-direction: column;
  gap: var(--su-space-4);
}

.action-card {
  display: flex;
  align-items: center;
  gap: var(--su-space-4);
  width: 100%;
  padding: var(--su-space-5) var(--su-space-6);
  color: inherit;
  font: inherit;
  text-align: left;
  background-color: var(--el-bg-color);
  border: 1px solid var(--el-border-color-light);
  border-radius: var(--su-radius-md);
  cursor: pointer;
  transition: border-color var(--su-transition-base);
}

.action-card:hover {
  border-color: var(--el-color-primary);
}

.action-icon {
  display: flex;
  flex: 0 0 48px;
  align-items: center;
  justify-content: center;
  width: 48px;
  height: 48px;
  color: var(--el-color-primary);
  font-size: var(--el-font-size-extra-large);
  background-color: var(--el-color-primary-light-9);
  border-radius: var(--su-radius-lg);
}

.action-text {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: var(--su-space-1);
  min-width: 0;
}

.action-text strong {
  color: var(--el-text-color-primary);
}

.action-text small,
.action-arrow {
  color: var(--el-text-color-secondary);
}

.action-arrow {
  flex-shrink: 0;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity var(--su-transition-slow);
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.result-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--su-space-4);
  margin-bottom: var(--su-space-3);
}

.result-heading h1,
.result-heading p {
  margin: 0;
}

.result-heading h1 {
  color: var(--el-text-color-primary);
  font-size: var(--el-font-size-large);
}

.result-heading p,
.view-mode {
  color: var(--el-text-color-secondary);
  font-size: var(--el-font-size-extra-small);
}

.result-heading p {
  margin-top: var(--su-space-1);
}

.view-mode {
  display: inline-flex;
  align-items: center;
  gap: var(--su-space-1);
}

.resource-list {
  overflow: hidden;
  background-color: var(--el-bg-color);
  border: 1px solid var(--el-border-color-light);
  border-radius: var(--su-radius-md);
}

.resource-loading {
  display: flex;
  min-height: 50vh;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--su-space-4);
  color: var(--el-text-color-secondary);
  background-color: var(--el-fill-color-light);
  border-radius: var(--su-radius-md);
}

.loading-spinner {
  color: var(--el-color-primary);
  font-size: 48px;
  animation: spin 1s linear infinite;
}

.resource-empty {
  padding-block: var(--su-space-10);
}

.pagination-container {
  display: flex;
  justify-content: center;
  padding-block: var(--su-space-6);
  overflow-x: auto;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

@media (max-width: 1439px) {
  .banner-wrapper {
    margin-inline: -48px;
  }
}

@media (max-width: 1279px) {
  .banner-wrapper {
    margin-inline: -20px;
  }
}

@media (max-width: 900px) {
  .resource-workspace {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 768px) {
  .resource-list-page {
    padding: var(--su-space-4);
  }

  .banner-wrapper {
    margin-inline: 0;
  }

  .action-text small,
  .view-mode {
    display: none;
  }
}

@media (prefers-reduced-motion: reduce) {
  .banner-collapse-enter-active,
  .banner-collapse-leave-active {
    transition: none;
  }
}
</style>
