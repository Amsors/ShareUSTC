import { computed, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { ElMessage } from 'element-plus';
import { getCourses } from '@/api/course';
import { getResourceCount, getResourceList, searchResources } from '@/api/resource';
import { getTeachers } from '@/api/teacher';
import { getErrorMessage, isHandledError } from '@/api/request';
import { useAuthStore } from '@/stores/auth';
import { useFavoriteStore } from '@/stores/favorite';
import type { Course } from '@/types/course';
import type { Favorite } from '@/types/favorite';
import {
  ResourceCategoryLabels,
  ResourceTypeFilterLabels,
  type ResourceCategoryType,
  type ResourceListItem,
  type ResourceSortField,
  type ResourceTypeFilterType,
} from '@/types/resource';
import type { Teacher } from '@/types/teacher';
import logger from '@/utils/logger';

export const RESOURCE_PAGE_SIZES = [10, 100, 1000];

const FILTER_EMPTY_RESOURCES_KEY = 'filterEmptyResources';
const ALL_RESOURCE_TYPES = Object.keys(ResourceTypeFilterLabels) as ResourceTypeFilterType[];
const ALL_RESOURCE_CATEGORIES = Object.keys(ResourceCategoryLabels) as ResourceCategoryType[];

// 兼容尚未返回 categoryCounts 的旧版后端，以当前页资源生成回退计数
export function countCurrentPageResourcesByCategory(
  resources: ResourceListItem[]
): Partial<Record<ResourceCategoryType, number>> {
  return resources.reduce<Partial<Record<ResourceCategoryType, number>>>((counts, resource) => {
    const category = resource.category as ResourceCategoryType;
    counts[category] = (counts[category] ?? 0) + 1;
    return counts;
  }, {});
}

// 全选时省略查询参数，全部取消时传入不可能命中的占位值
export function getMultiSelectQueryValue<T extends string>(
  selected: T[],
  all: T[]
): string[] | undefined {
  if (selected.length === all.length && all.every((value) => selected.includes(value))) {
    return undefined;
  }
  return selected.length > 0 ? selected : ['__none__'];
}

// 维护资源列表页的检索、分页与批量收藏状态
export function useResourceListPage() {
  const router = useRouter();
  const route = useRoute();
  const favoriteStore = useFavoriteStore();
  const authStore = useAuthStore();

  const loading = ref(false);
  const resources = ref<ResourceListItem[]>([]);
  const total = ref(0);
  const categoryCounts = ref<Partial<Record<ResourceCategoryType, number>>>({});
  const currentPage = ref(1);
  const pageSize = ref(100);
  const hasSearched = ref(false);
  const totalResourceCount = ref(0);

  const searchQuery = ref('');
  const filterTypes = ref<ResourceTypeFilterType[]>([...ALL_RESOURCE_TYPES]);
  const filterCategories = ref<ResourceCategoryType[]>([...ALL_RESOURCE_CATEGORIES]);
  const sortBy = ref<ResourceSortField | ''>('created_at');
  const filterTeacherSns = ref<number[]>([]);
  const filterCourseSns = ref<number[]>([]);
  const appliedTeacherSns = ref<number[]>([]);
  const appliedCourseSns = ref<number[]>([]);

  const teacherList = ref<Teacher[]>([]);
  const courseList = ref<Course[]>([]);
  const loadingTeachers = ref(false);
  const loadingCourses = ref(false);

  const enableQuickAdd = ref(false);
  const selectedFavoriteId = ref('');
  const favoriteLocked = ref(false);
  const addingResourceId = ref<string | null>(null);
  const batchAddingAll = ref(false);
  const favoritesWithCount = ref<Favorite[]>([]);
  let latestResourceRequestId = 0;

  const hasNonRelationSearchCriteria = computed(
    () =>
      searchQuery.value.trim().length > 0 ||
      filterTypes.value.length !== ALL_RESOURCE_TYPES.length ||
      filterCategories.value.length !== ALL_RESOURCE_CATEGORIES.length ||
      (sortBy.value !== '' && sortBy.value !== 'created_at')
  );
  const hasSearchCriteria = computed(
    () =>
      hasNonRelationSearchCriteria.value ||
      filterTeacherSns.value.length > 0 ||
      filterCourseSns.value.length > 0
  );
  const hasAppliedSearchCriteria = computed(
    () =>
      hasNonRelationSearchCriteria.value ||
      appliedTeacherSns.value.length > 0 ||
      appliedCourseSns.value.length > 0
  );
  const hasPendingRelationChanges = computed(
    () =>
      filterTeacherSns.value.length !== appliedTeacherSns.value.length ||
      filterCourseSns.value.length !== appliedCourseSns.value.length ||
      filterTeacherSns.value.some((sn) => !appliedTeacherSns.value.includes(sn)) ||
      filterCourseSns.value.some((sn) => !appliedCourseSns.value.includes(sn))
  );
  const isSearchMode = computed(() => searchQuery.value.trim().length > 0);
  const selectedFavorite = computed(() =>
    favoritesWithCount.value.find((favorite) => favorite.id === selectedFavoriteId.value)
  );

  const getFilterSetting = (): boolean => {
    try {
      const stored = localStorage.getItem(FILTER_EMPTY_RESOURCES_KEY);
      if (stored) {
        const data: unknown = JSON.parse(stored);
        if (typeof data === 'object' && data !== null && 'enabled' in data) {
          return data.enabled !== false;
        }
      }
    } catch (error) {
      logger.warn('[ResourceList]', '解析资源过滤设置失败:', error);
    }
    return true;
  };

  const loadTeachers = async () => {
    loadingTeachers.value = true;
    try {
      teacherList.value = await getTeachers(getFilterSetting());
    } catch (error) {
      logger.error('[ResourceList]', '加载教师列表失败:', error);
    } finally {
      loadingTeachers.value = false;
    }
  };

  const loadCourses = async () => {
    loadingCourses.value = true;
    try {
      courseList.value = await getCourses(getFilterSetting());
    } catch (error) {
      logger.error('[ResourceList]', '加载课程列表失败:', error);
    } finally {
      loadingCourses.value = false;
    }
  };

  const loadFavorites = async () => {
    if (!authStore.isAuthenticated) return;
    try {
      await favoriteStore.fetchFavorites();
      favoritesWithCount.value = favoriteStore.favorites;
    } catch (error) {
      logger.error('[ResourceList]', '加载收藏夹失败:', error);
    }
  };

  const loadResourceCount = async () => {
    try {
      const result = await getResourceCount();
      totalResourceCount.value = result.total;
    } catch (error) {
      logger.error('[ResourceList]', '获取资源总数失败:', error);
    }
  };

  const loadResources = async () => {
    const requestId = ++latestResourceRequestId;
    loading.value = true;
    try {
      const teacherSns = appliedTeacherSns.value.length ? appliedTeacherSns.value : undefined;
      const courseSns = appliedCourseSns.value.length ? appliedCourseSns.value : undefined;
      const resourceTypes = getMultiSelectQueryValue(filterTypes.value, ALL_RESOURCE_TYPES);
      const categories = getMultiSelectQueryValue(filterCategories.value, ALL_RESOURCE_CATEGORIES);
      const response = isSearchMode.value
        ? await searchResources({
            q: searchQuery.value.trim(),
            page: currentPage.value,
            perPage: pageSize.value,
            resourceTypes,
            categories,
            teacherSns,
            courseSns,
          })
        : await getResourceList({
            page: currentPage.value,
            perPage: pageSize.value,
            resourceTypes,
            categories,
            sortBy: sortBy.value || undefined,
            sortOrder: 'desc',
            teacherSns,
            courseSns,
          });

      if (requestId !== latestResourceRequestId) return;

      resources.value = response.resources;
      total.value = response.total;
      categoryCounts.value =
        response.categoryCounts ?? countCurrentPageResourcesByCategory(response.resources);
    } catch (error) {
      logger.error('[ResourceList]', '加载资源列表失败:', error);
      if (requestId === latestResourceRequestId && !isHandledError(error)) {
        ElMessage.error(getErrorMessage(error, '加载资源列表失败'));
      }
    } finally {
      if (requestId === latestResourceRequestId) loading.value = false;
    }
  };

  const runSearch = () => {
    hasSearched.value = true;
    currentPage.value = 1;
    void loadResources();
  };

  const resetToBanner = () => {
    hasSearched.value = false;
    resources.value = [];
    total.value = 0;
    categoryCounts.value = {};
  };

  const handleSearch = () => {
    if (!hasSearchCriteria.value) {
      if (hasSearched.value) {
        appliedTeacherSns.value = [];
        appliedCourseSns.value = [];
        resetToBanner();
        return;
      }
      ElMessage.warning('请输入关键词或选择筛选条件');
      return;
    }
    appliedTeacherSns.value = [...filterTeacherSns.value];
    appliedCourseSns.value = [...filterCourseSns.value];
    runSearch();
  };

  const handleClearSearch = () => {
    if (hasAppliedSearchCriteria.value) runSearch();
    else if (hasSearched.value) resetToBanner();
  };

  const handleSizeChange = (size: number) => {
    pageSize.value = size;
    currentPage.value = 1;
    void loadResources();
  };

  const handlePageChange = (page: number) => {
    currentPage.value = page;
    void loadResources();
  };

  const handleSelectFavorite = () => {
    if (selectedFavoriteId.value) favoriteLocked.value = true;
  };

  const handleChangeFavorite = () => {
    favoriteLocked.value = false;
  };

  const handleQuickAddToggle = (enabled: boolean) => {
    if (!enabled) {
      favoriteLocked.value = false;
      selectedFavoriteId.value = '';
    }
  };

  const handleAddAllCurrentPage = async () => {
    if (!selectedFavoriteId.value || resources.value.length === 0) return;

    batchAddingAll.value = true;
    const favoriteName = selectedFavorite.value?.name || '收藏夹';
    let successCount = 0;
    let existCount = 0;
    let failCount = 0;

    try {
      for (const resource of resources.value) {
        try {
          const added = await favoriteStore.addResourceToFavorite(
            selectedFavoriteId.value,
            resource.id
          );
          if (added) successCount += 1;
          else existCount += 1;
        } catch (error) {
          failCount += 1;
          logger.error('[ResourceList]', `批量添加资源失败: ${resource.id}`, error);
        }
      }

      const parts: string[] = [];
      if (successCount > 0) parts.push(`成功添加 ${successCount} 份`);
      if (existCount > 0) parts.push(`已存在 ${existCount} 份`);
      if (failCount > 0) parts.push(`失败 ${failCount} 份`);

      if (successCount > 0 && existCount === 0 && failCount === 0) {
        ElMessage.success(`成功将 ${successCount} 份资源加入收藏夹: ${favoriteName}`);
      } else if (successCount > 0 || existCount > 0) {
        ElMessage.info(`${parts.join('，')} 到收藏夹: ${favoriteName}`);
      } else if (failCount > 0) {
        ElMessage.error(`添加失败，${failCount} 份资源未能加入收藏夹`);
      }
    } catch (error) {
      logger.error('[ResourceList]', '批量添加所有资源失败:', error);
      if (!isHandledError(error)) ElMessage.error('批量添加失败，请稍后重试');
    } finally {
      batchAddingAll.value = false;
    }
  };

  const handleResourceItemClick = async (resource: ResourceListItem) => {
    if (!enableQuickAdd.value) {
      await router.push(`/resources/${resource.id}`);
      return;
    }
    if (!selectedFavoriteId.value) {
      ElMessage.warning('请先选择收藏夹');
      return;
    }
    if (addingResourceId.value) return;

    addingResourceId.value = resource.id;
    try {
      const added = await favoriteStore.addResourceToFavorite(
        selectedFavoriteId.value,
        resource.id
      );
      if (added) ElMessage.success(`已加入收藏夹: ${selectedFavorite.value?.name}`);
      else ElMessage.warning('该资源已在收藏夹中');
    } catch (error) {
      logger.error('[ResourceList]', '添加资源到收藏夹失败:', error);
      if (!isHandledError(error)) ElMessage.error(getErrorMessage(error, '添加失败'));
    } finally {
      addingResourceId.value = null;
    }
  };

  const goToUpload = () => router.push('/upload');
  const goToAccount = () => router.push(authStore.isAuthenticated ? '/profile' : '/login');

  const initialize = () => {
    const queryKeyword = typeof route.query.q === 'string' ? route.query.q : '';
    if (queryKeyword) {
      searchQuery.value = queryKeyword;
      runSearch();
    }
    void loadTeachers();
    void loadCourses();
    void loadFavorites();
    void loadResourceCount();
  };

  watch(
    [filterTypes, filterCategories, sortBy],
    () => {
      if (hasAppliedSearchCriteria.value) runSearch();
      else if (hasSearched.value) resetToBanner();
    },
    { deep: true }
  );

  return {
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
  };
}
