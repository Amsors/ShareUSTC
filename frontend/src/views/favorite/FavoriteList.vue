<template>
  <div class="favorite-list-page">
    <div class="page-header">
      <h1>我的收藏夹</h1>
      <el-button type="primary" @click="showCreateModal = true">
        <el-icon><Plus /></el-icon>
        新建收藏夹
      </el-button>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="loading-container">
      <el-icon class="loading-icon" :size="48"><Loading /></el-icon>
    </div>

    <!-- 空状态 -->
    <el-empty v-else-if="!hasFavorites" description="暂无收藏夹">
      <el-button type="primary" @click="showCreateModal = true"> 创建第一个收藏夹 </el-button>
    </el-empty>

    <!-- 收藏夹列表 -->
    <div v-else class="favorite-grid">
      <el-card
        v-for="favorite in favorites"
        :key="favorite.id"
        class="favorite-card"
        shadow="hover"
        @click="goToDetail(favorite.id)"
      >
        <div class="favorite-content">
          <div class="favorite-icon">
            <el-icon :size="48" color="#409EFF"><Folder /></el-icon>
          </div>
          <div class="favorite-info">
            <h3 class="favorite-name">{{ favorite.name }}</h3>
            <p class="favorite-meta">
              <el-icon><Document /></el-icon>
              {{ favorite.resourceCount }} 个资源
            </p>
            <p class="favorite-date">创建于 {{ formatDate(favorite.createdAt) }}</p>
          </div>
        </div>
        <div class="favorite-actions" @click.stop>
          <!-- 默认收藏夹标记 -->
          <el-tag
            v-if="isDefaultFavorite(favorite.id)"
            type="success"
            size="small"
            class="default-tag"
          >
            <el-icon><StarFilled /></el-icon>
            默认
          </el-tag>
          <el-dropdown trigger="click">
            <el-button type="primary" text>
              <el-icon><More /></el-icon>
            </el-button>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item @click="handleSetDefault(favorite)">
                  <el-icon><Star /></el-icon>
                  {{ isDefaultFavorite(favorite.id) ? '取消默认' : '设为默认' }}
                </el-dropdown-item>
                <el-dropdown-item @click="handleEdit(favorite)">
                  <el-icon><Edit /></el-icon>
                  重命名
                </el-dropdown-item>
                <el-dropdown-item divided @click="handleDelete(favorite)">
                  <el-icon><Delete /></el-icon>
                  <span style="color: #f56c6c">删除</span>
                </el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>
      </el-card>
    </div>

    <!-- 创建收藏夹弹窗 -->
    <CreateFavoriteModal v-model="showCreateModal" @success="handleCreateSuccess" />

    <!-- 编辑收藏夹弹窗 -->
    <CreateFavoriteModal
      v-model="showEditModal"
      :favorite="editingFavorite"
      is-edit
      @success="handleEditSuccess"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { ElMessage, ElMessageBox } from 'element-plus';
import {
  Plus,
  Folder,
  Document,
  More,
  Edit,
  Delete,
  Loading,
  Star,
  StarFilled,
} from '@element-plus/icons-vue';
import { storeToRefs } from 'pinia';
import { useFavoriteStore } from '../../stores/favorite';
import type { Favorite } from '../../types/favorite';
import CreateFavoriteModal from '../../components/favorite/CreateFavoriteModal.vue';
import { useDefaultFavorite } from '../../composables/useDefaultFavorite';
import { getErrorMessage } from '@/api/request';

const router = useRouter();
const favoriteStore = useFavoriteStore();
const { setDefaultFavorite, isDefaultFavorite, clearDefaultFavorite } = useDefaultFavorite();

// 状态
const loading = ref(false);
const showCreateModal = ref(false);
const showEditModal = ref(false);
const editingFavorite = ref<Favorite | null>(null);

// 从 store 获取数据（使用 storeToRefs 保持响应性）
const { favorites, hasFavorites } = storeToRefs(favoriteStore);

// 格式化日期
const formatDate = (dateStr: string) => {
  // 将无时区的时间字符串视为 UTC 时间
  const utcTimeString = dateStr.endsWith('Z') ? dateStr : `${dateStr}Z`;
  const date = new Date(utcTimeString);
  return date.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
};

// 获取收藏夹列表
const fetchFavorites = async () => {
  loading.value = true;
  try {
    await favoriteStore.fetchFavorites();
  } catch {
    ElMessage.error('获取收藏夹列表失败');
  } finally {
    loading.value = false;
  }
};

// 跳转到详情页
const goToDetail = (favoriteId: string) => {
  router.push(`/favorites/${favoriteId}`);
};

// 编辑收藏夹
const handleEdit = (favorite: Favorite) => {
  editingFavorite.value = favorite;
  showEditModal.value = true;
};

// 删除收藏夹
const handleDelete = async (favorite: Favorite) => {
  try {
    await ElMessageBox.confirm(
      `确定要删除收藏夹 "${favorite.name}" 吗？此操作不可恢复。`,
      '确认删除',
      {
        confirmButtonText: '删除',
        cancelButtonText: '取消',
        type: 'warning',
      }
    );

    await favoriteStore.deleteFavorite(favorite.id);

    // 如果删除的是默认收藏夹，清除默认收藏夹设置
    if (isDefaultFavorite(favorite.id)) {
      clearDefaultFavorite();
    }

    ElMessage.success('删除成功');
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error(getErrorMessage(error, '删除失败'));
    }
  }
};

// 创建成功回调
const handleCreateSuccess = () => {
  showCreateModal.value = false;
  ElMessage.success('创建成功');
};

// 编辑成功回调
const handleEditSuccess = () => {
  showEditModal.value = false;
  editingFavorite.value = null;
  ElMessage.success('更新成功');
};

// 设置/取消默认收藏夹
const handleSetDefault = async (favorite: Favorite) => {
  if (isDefaultFavorite(favorite.id)) {
    // 取消默认
    setDefaultFavorite('', '');
    ElMessage.success('已取消默认收藏夹');
  } else {
    // 设为默认
    setDefaultFavorite(favorite.id, favorite.name);
    ElMessage.success(`已将 "${favorite.name}" 设为默认收藏夹`);
  }
};

// 页面加载时获取数据
onMounted(() => {
  fetchFavorites();
});
</script>

<style scoped lang="scss">
.favorite-list-page {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 30px;

  h1 {
    margin: 0;
    font-size: 24px;
    color: #303133;
  }
}

.loading-container {
  padding: 60px 0;
  text-align: center;
}

.loading-icon {
  color: #409eff;
  animation: rotating 2s linear infinite;
}

@keyframes rotating {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.favorite-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 20px;
}

.favorite-card {
  cursor: pointer;
  transition: all 0.3s;
  position: relative;

  &:hover {
    transform: translateY(-2px);
  }

  :deep(.el-card__body) {
    padding: 20px;
  }
}

.favorite-content {
  display: flex;
  align-items: flex-start;
  gap: 16px;
}

.favorite-icon {
  flex-shrink: 0;
}

.favorite-info {
  flex: 1;
  min-width: 0;

  .favorite-name {
    margin: 0 0 8px;
    font-size: 16px;
    font-weight: 600;
    color: #303133;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .favorite-meta {
    margin: 0 0 4px;
    font-size: 14px;
    color: #606266;
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .favorite-date {
    margin: 0;
    font-size: 12px;
    color: #909399;
  }
}

.favorite-actions {
  position: absolute;
  top: 10px;
  right: 10px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.default-tag {
  display: inline-flex !important;
  align-items: center;
  flex-shrink: 0;
  padding: 0 6px;
  height: 24px;

  :deep(.el-tag__content) {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    white-space: nowrap;
  }
}

@media (max-width: 768px) {
  .favorite-grid {
    grid-template-columns: 1fr;
  }
}
</style>
