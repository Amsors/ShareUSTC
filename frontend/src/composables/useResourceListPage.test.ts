import { nextTick } from 'vue';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
  getCourses: vi.fn(() => Promise.resolve([])),
  getTeachers: vi.fn(() => Promise.resolve([])),
  getResourceCount: vi.fn(() => Promise.resolve({ total: 0 })),
  getResourceList: vi.fn(() => Promise.resolve({ resources: [], total: 0 })),
  searchResources: vi.fn(() => Promise.resolve({ resources: [], total: 0 })),
  warning: vi.fn(),
  routerPush: vi.fn(),
  authStore: { isAuthenticated: false },
  favoriteStore: {
    loading: false,
    favorites: [],
    fetchFavorites: vi.fn(() => Promise.resolve()),
    addResourceToFavorite: vi.fn(() => Promise.resolve(true)),
  },
}));

vi.mock('@/api/course', () => ({ getCourses: mocks.getCourses }));
vi.mock('@/api/teacher', () => ({ getTeachers: mocks.getTeachers }));
vi.mock('@/api/resource', () => ({
  getResourceCount: mocks.getResourceCount,
  getResourceList: mocks.getResourceList,
  searchResources: mocks.searchResources,
}));
vi.mock('@/api/request', () => ({
  getErrorMessage: (_error: unknown, fallback: string) => fallback,
  isHandledError: () => false,
}));
vi.mock('@/stores/auth', () => ({ useAuthStore: () => mocks.authStore }));
vi.mock('@/stores/favorite', () => ({ useFavoriteStore: () => mocks.favoriteStore }));
vi.mock('vue-router', () => ({
  useRoute: () => ({ query: {} }),
  useRouter: () => ({ push: mocks.routerPush }),
}));
vi.mock('element-plus', () => ({
  ElMessage: {
    error: vi.fn(),
    info: vi.fn(),
    success: vi.fn(),
    warning: mocks.warning,
  },
}));

import {
  countCurrentPageResourcesByCategory,
  getMultiSelectQueryValue,
  useResourceListPage,
} from '@/composables/useResourceListPage';
import type { ResourceListItem } from '@/types/resource';

beforeEach(() => {
  mocks.getResourceList.mockClear();
  mocks.searchResources.mockClear();
  mocks.warning.mockClear();
});

describe('getMultiSelectQueryValue', () => {
  const all = ['pdf', 'ppt', 'doc'];

  it('全选时省略查询参数', () => {
    expect(getMultiSelectQueryValue([...all], all)).toBeUndefined();
  });

  it('部分选择时传入所选值', () => {
    expect(getMultiSelectQueryValue(['pdf', 'doc'], all)).toEqual(['pdf', 'doc']);
  });

  it('全部取消时传入不会命中的占位值', () => {
    expect(getMultiSelectQueryValue([], all)).toEqual(['__none__']);
  });
});

describe('countCurrentPageResourcesByCategory', () => {
  it('旧版响应缺少计数时按当前页资源分类统计', () => {
    const resources = [
      { category: 'other' },
      { category: 'learning_note' },
      { category: 'other' },
      { category: 'other' },
    ] as ResourceListItem[];

    expect(countCurrentPageResourcesByCategory(resources)).toEqual({
      learning_note: 1,
      other: 3,
    });
  });
});

describe('useResourceListPage', () => {
  it('选择课程和教师时不请求资源，点击搜索后才应用条件', async () => {
    const page = useResourceListPage();

    page.filterCourseSns.value = [101];
    page.filterTeacherSns.value = [202];
    await nextTick();

    expect(page.hasPendingRelationChanges.value).toBe(true);
    expect(mocks.getResourceList).not.toHaveBeenCalled();
    expect(mocks.searchResources).not.toHaveBeenCalled();

    page.handleSearch();

    expect(page.hasPendingRelationChanges.value).toBe(false);
    expect(mocks.getResourceList).toHaveBeenCalledWith(
      expect.objectContaining({ courseSns: [101], teacherSns: [202] })
    );
  });

  it('清空已应用的课程和教师后提示存在待应用修改', async () => {
    const page = useResourceListPage();
    page.filterCourseSns.value = [101];
    page.filterTeacherSns.value = [202];
    page.handleSearch();

    page.filterCourseSns.value = [];
    page.filterTeacherSns.value = [];
    await nextTick();

    expect(page.hasPendingRelationChanges.value).toBe(true);
  });

  it('未点击搜索时，翻页仍使用上一次已应用的课程条件', async () => {
    const page = useResourceListPage();
    page.filterCourseSns.value = [101];
    page.handleSearch();
    await nextTick();
    mocks.getResourceList.mockClear();

    page.filterCourseSns.value = [303];
    await nextTick();
    page.handlePageChange(2);

    expect(mocks.getResourceList).toHaveBeenCalledWith(
      expect.objectContaining({ courseSns: [101], page: 2 })
    );
  });
});
