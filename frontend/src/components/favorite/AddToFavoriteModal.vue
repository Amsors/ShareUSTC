<template>
  <el-dialog
    v-model="visible"
    title="添加到收藏夹"
    width="450px"
    :close-on-click-modal="false"
    @close="handleClose"
  >
    <!-- 快速创建新收藏夹 -->
    <div class="create-section">
      <el-input
        v-model="newFavoriteName"
        placeholder="新建收藏夹..."
        maxlength="100"
        clearable
        @keyup.enter="handleCreateNew"
      >
        <template #append>
          <el-button :loading="creating" @click="handleCreateNew">
            <el-icon><Plus /></el-icon>
          </el-button>
        </template>
      </el-input>
    </div>

    <!-- 收藏夹列表 -->
    <div class="favorites-section">
      <h4>我的收藏夹</h4>

      <el-scrollbar max-height="300px">
        <!-- 加载状态 -->
        <div v-if="loading" class="loading-wrapper">
          <span class="loading-text">加载中...</span>
        </div>

        <!-- 空状态 -->
        <div v-else-if="favorites.length === 0" class="empty-wrapper">
          <p class="empty-text">暂无收藏夹，请创建一个新收藏夹</p>
        </div>

        <!-- 收藏夹列表 -->
        <div v-else class="favorite-list">
          <div
            v-for="favorite in favorites"
            :key="favorite.id"
            class="favorite-item"
            :class="{ 'is-selected': isInFavorite(favorite.id) }"
            @click="toggleFavorite(favorite.id)"
          >
            <div class="favorite-item-content">
              <el-icon :size="20" color="#409EFF"><Folder /></el-icon>
              <span class="favorite-name">{{ favorite.name }}</span>
              <span class="favorite-count">{{ favorite.resourceCount }} 个资源</span>
            </div>
            <div class="favorite-check">
              <el-icon v-if="isInFavorite(favorite.id)" :size="20" color="#67C23A"
                ><Check
              /></el-icon>
            </div>
          </div>
        </div>
      </el-scrollbar>
    </div>

    <template #footer>
      <el-button @click="visible = false">关闭</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, computed } from 'vue';
import { ElMessage } from 'element-plus';
import { Plus, Folder, Check } from '@element-plus/icons-vue';
import { useFavoriteStore } from '../../stores/favorite';
import { storeToRefs } from 'pinia';
import * as favoriteApi from '../../api/favorite';
import { getErrorMessage, isHandledError } from '@/api/request';
import logger from '@/utils/logger';

const props = defineProps<{
  modelValue: boolean;
  resourceId: string;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'success'): void;
}>();

const favoriteStore = useFavoriteStore();

// 使用 storeToRefs 保持响应式
const { favorites: storeFavorites } = storeToRefs(favoriteStore);

// 使用计算属性确保响应式更新
const favorites = computed(() => storeFavorites.value);

// 弹窗可见性
const visible = ref(props.modelValue);

// 监听 modelValue 变化
watch(
  () => props.modelValue,
  (newVal) => {
    visible.value = newVal;
    if (newVal) {
      fetchData();
    }
  }
);

// 监听 visible 变化，同步到父组件
watch(
  () => visible.value,
  (newVal) => {
    emit('update:modelValue', newVal);
  }
);

// 状态
const loading = ref(false);
const creating = ref(false);
const newFavoriteName = ref('');
const selectedFavorites = ref<Set<string>>(new Set());

// 获取数据
const fetchData = async () => {
  loading.value = true;
  try {
    // 并行获取收藏夹列表和资源收藏状态
    const [, statusRes] = await Promise.all([
      favoriteStore.fetchFavorites(),
      favoriteApi.checkResourceInFavorite(props.resourceId),
    ]);

    // 设置已选中的收藏夹
    selectedFavorites.value = new Set(statusRes.inFavorites);
  } catch (error) {
    logger.error('[AddToFavoriteModal]', '获取收藏夹数据失败', error);
    if (!isHandledError(error)) {
      ElMessage.error('获取数据失败');
    }
  } finally {
    loading.value = false;
  }
};

// 检查资源是否在收藏夹中
const isInFavorite = (favoriteId: string) => {
  return selectedFavorites.value.has(favoriteId);
};

// 切换收藏夹选择状态
const toggleFavorite = async (favoriteId: string) => {
  try {
    if (isInFavorite(favoriteId)) {
      // 从收藏夹移除（store 方法内部已调用 API 并更新计数）
      await favoriteStore.removeResourceFromFavorite(favoriteId, props.resourceId);
      selectedFavorites.value.delete(favoriteId);
      ElMessage.success('已从收藏夹移除');
    } else {
      // 添加到收藏夹
      const added = await favoriteStore.addResourceToFavorite(favoriteId, props.resourceId);
      if (added) {
        selectedFavorites.value.add(favoriteId);
        ElMessage.success('已添加到收藏夹');
      } else {
        // 资源已存在
        selectedFavorites.value.add(favoriteId);
        ElMessage.warning('该资源已在收藏夹中');
      }
    }
    emit('success');
  } catch (error) {
    logger.error('[AddToFavoriteModal]', '切换收藏夹失败', error);
    if (!isHandledError(error)) {
      const errorMessage = getErrorMessage(error, '操作失败');
      ElMessage.error(errorMessage);
    }
  }
};

// 创建新收藏夹
const handleCreateNew = async () => {
  const name = newFavoriteName.value.trim();
  if (!name) {
    ElMessage.warning('请输入收藏夹名称');
    return;
  }

  creating.value = true;
  try {
    // 1. 创建收藏夹
    const response = await favoriteStore.createFavorite(name);
    newFavoriteName.value = '';

    // 2. 自动添加到新创建的收藏夹
    if (response?.id) {
      const added = await favoriteStore.addResourceToFavorite(response.id, props.resourceId);
      if (added) {
        selectedFavorites.value.add(response.id);
        emit('success');
        ElMessage.success('创建成功并已添加到收藏夹');
      } else {
        selectedFavorites.value.add(response.id);
        emit('success');
        ElMessage.warning('创建成功，但该资源已在收藏夹中');
      }
    } else {
      ElMessage.success('创建成功');
    }
  } catch (error) {
    logger.error('[AddToFavoriteModal]', '创建收藏夹失败', error);
    if (!isHandledError(error)) {
      ElMessage.error(getErrorMessage(error, '创建失败'));
    }
  } finally {
    creating.value = false;
  }
};

// 关闭弹窗
const handleClose = () => {
  newFavoriteName.value = '';
};

// 页面加载时获取数据
onMounted(() => {
  if (visible.value) {
    fetchData();
  }
});
</script>

<style scoped lang="scss">
.create-section {
  margin-bottom: 20px;
}

.favorites-section {
  h4 {
    margin: 0 0 12px;
    font-size: 14px;
    color: var(--el-text-color-regular);
  }
}

.loading-wrapper,
.empty-wrapper {
  padding: 20px 0;
  text-align: center;
}

.empty-text {
  color: var(--el-text-color-secondary);
  font-size: 14px;
}

.loading-text {
  color: var(--el-text-color-secondary);
  font-size: 14px;
}

.favorite-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.favorite-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  border: 1px solid var(--el-border-color-lighter);

  &:hover {
    background-color: var(--el-fill-color-light);
  }

  &.is-selected {
    background-color: #f0f9ff;
    border-color: var(--el-color-primary);
  }

  .favorite-item-content {
    display: flex;
    align-items: center;
    gap: 12px;
    flex: 1;
    min-width: 0;

    .favorite-name {
      font-size: 14px;
      color: var(--el-text-color-primary);
      flex: 1;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }

    .favorite-count {
      font-size: 12px;
      color: var(--el-text-color-secondary);
      flex-shrink: 0;
    }
  }

  .favorite-check {
    width: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
  }
}
</style>
