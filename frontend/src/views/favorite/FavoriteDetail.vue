<template>
  <div class="favorite-detail-page">
    <!-- 面包屑导航 -->
    <el-breadcrumb class="breadcrumb">
      <el-breadcrumb-item :to="{ path: '/favorites' }">收藏夹</el-breadcrumb-item>
      <el-breadcrumb-item>{{ favoriteName }}</el-breadcrumb-item>
    </el-breadcrumb>

    <!-- 头部信息 -->
    <div class="detail-header">
      <div class="header-left">
        <div class="favorite-title">
          <el-icon :size="32" color="#409EFF"><Folder /></el-icon>
          <h1>{{ favoriteName }}</h1>
        </div>
        <p class="favorite-meta">
          共 {{ resourceCount }} 个资源 · 创建于 {{ createdAt }}
        </p>
      </div>
      <div class="header-actions">
        <el-button @click="showEditModal = true">
          <el-icon><Edit /></el-icon>
          重命名
        </el-button>
        <el-button type="primary" @click="handleDownload" :loading="downloading">
          <el-icon><Download /></el-icon>
          打包下载
        </el-button>
        <el-button type="danger" text @click="handleDelete">
          <el-icon><Delete /></el-icon>
          删除收藏夹
        </el-button>
      </div>
    </div>

    <!-- 资源列表 -->
    <div class="resource-list">
      <!-- 加载状态 -->
      <el-skeleton v-if="loading" :rows="5" animated />

      <!-- 空状态 -->
      <el-empty
        v-else-if="resources.length === 0"
        description="收藏夹是空的"
      >
        <p>快去浏览资源并添加到收藏夹吧！</p>
        <el-button type="primary" @click="$router.push('/resources')">
          浏览资源
        </el-button>
      </el-empty>

      <!-- 资源卡片列表 -->
      <div v-else class="resource-grid">
        <el-card
          v-for="resource in resources"
          :key="resource.id"
          class="resource-card"
          shadow="hover"
        >
          <div class="resource-content" @click="goToResource(resource.id)">
            <!-- 资源类型图标 -->
            <div
              class="resource-type-icon"
              :style="{ backgroundColor: getResourceTypeColor(resource.resourceType) }"
            >
              {{ resource.resourceType.toUpperCase() }}
            </div>

            <div class="resource-info">
              <h4 class="resource-title">{{ resource.title }}</h4>
              <p v-if="resource.courseName" class="resource-course">
                {{ resource.courseName }}
              </p>
              <div class="resource-tags" v-if="resource.tags?.length">
                <el-tag
                  v-for="tag in resource.tags.slice(0, 3)"
                  :key="tag"
                  size="small"
                  effect="plain"
                >
                  {{ tag }}
                </el-tag>
              </div>
              <div class="resource-stats">
                <span>
                  <el-icon><View /></el-icon>
                  {{ resource.stats.views }}
                </span>
                <span>
                  <el-icon><Download /></el-icon>
                  {{ resource.stats.downloads }}
                </span>
                <span>
                  <el-icon><Star /></el-icon>
                  {{ resource.stats.likes }}
                </span>
              </div>
            </div>
          </div>

          <div class="resource-actions" @click.stop>
            <el-popconfirm
              title="确定从收藏夹移除此资源？"
              confirm-button-text="移除"
              cancel-button-text="取消"
              @confirm="removeResource(resource.id)"
            >
              <template #reference>
                <el-button type="danger" text size="small">
                  <el-icon><Remove /></el-icon>
                  移除
                </el-button>
              </template>
            </el-popconfirm>
          </div>
        </el-card>
      </div>
    </div>

    <!-- 编辑收藏夹弹窗 -->
    <CreateFavoriteModal
      v-if="currentFavorite"
      v-model="showEditModal"
      :favorite="{ id: currentFavorite.id, name: currentFavorite.name, resourceCount: currentFavorite.resourceCount, createdAt: currentFavorite.createdAt }"
      is-edit
      @success="handleEditSuccess"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { ElMessage, ElMessageBox } from 'element-plus';
import {
  Folder,
  Edit,
  Delete,
  Download,
  View,
  Star,
  Remove
} from '@element-plus/icons-vue';
import { useFavoriteStore } from '../../stores/favorite';
import { downloadFavorite } from '../../api/favorite';
import { ResourceTypeLabels } from '../../types/resource';
import CreateFavoriteModal from '../../components/favorite/CreateFavoriteModal.vue';

const route = useRoute();
const router = useRouter();
const favoriteStore = useFavoriteStore();

// 从 route 获取收藏夹ID
const favoriteId = computed(() => route.params.id as string);

// 状态
const loading = ref(false);
const downloading = ref(false);
const showEditModal = ref(false);

// 从 store 获取数据
const currentFavorite = computed(() => favoriteStore.currentFavorite);
const favoriteName = computed(() => currentFavorite.value?.name || '加载中...');
const resourceCount = computed(() => currentFavorite.value?.resourceCount || 0);
const resources = computed(() => currentFavorite.value?.resources || []);
const createdAt = computed(() => {
  if (!currentFavorite.value?.createdAt) return '';
  const date = new Date(currentFavorite.value.createdAt);
  return date.toLocaleDateString('zh-CN');
});

// 获取资源类型颜色
const getResourceTypeColor = (type: string) => {
  const colorMap: Record<string, string> = {
    'pdf': '#F56C6C',
    'ppt': '#E6A23C',
    'pptx': '#E6A23C',
    'doc': '#409EFF',
    'docx': '#409EFF',
    'web_markdown': '#67C23A',
    'txt': '#909399',
    'zip': '#909399'
  };
  return colorMap[type] || '#909399';
};

// 获取收藏夹详情
const fetchDetail = async () => {
  loading.value = true;
  try {
    await favoriteStore.fetchFavoriteDetail(favoriteId.value);
  } catch (error) {
    ElMessage.error('获取收藏夹详情失败');
    router.push('/favorites');
  } finally {
    loading.value = false;
  }
};

// 跳转到资源详情
const goToResource = (resourceId: string) => {
  router.push(`/resources/${resourceId}`);
};

// 移除资源
const removeResource = async (resourceId: string) => {
  try {
    await favoriteStore.removeResourceFromFavorite(favoriteId.value, resourceId);
    ElMessage.success('移除成功');
  } catch (error: any) {
    ElMessage.error(error.message || '移除失败');
  }
};

// 编辑成功回调
const handleEditSuccess = () => {
  showEditModal.value = false;
  fetchDetail();
  ElMessage.success('更新成功');
};

// 打包下载
const handleDownload = async () => {
  if (resourceCount.value === 0) {
    ElMessage.warning('收藏夹为空，无法下载');
    return;
  }

  downloading.value = true;
  try {
    await downloadFavorite(favoriteId.value, currentFavorite.value?.name);
    ElMessage.success('开始下载');
  } catch (error: any) {
    ElMessage.error(error.message || '下载失败');
  } finally {
    downloading.value = false;
  }
};

// 删除收藏夹
const handleDelete = async () => {
  try {
    await ElMessageBox.confirm(
      `确定要删除收藏夹 "${favoriteName.value}" 吗？此操作不可恢复。`,
      '确认删除',
      {
        confirmButtonText: '删除',
        cancelButtonText: '取消',
        type: 'warning'
      }
    );

    await favoriteStore.deleteFavorite(favoriteId.value);
    ElMessage.success('删除成功');
    router.push('/favorites');
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(error.message || '删除失败');
    }
  }
};

// 页面加载时获取数据
onMounted(() => {
  fetchDetail();
});
</script>

<style scoped lang="scss">
.favorite-detail-page {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

.breadcrumb {
  margin-bottom: 20px;
}

.detail-header {
  background: #fff;
  border-radius: 8px;
  padding: 24px;
  margin-bottom: 24px;
  box-shadow: 0 2px 12px 0 rgba(0, 0, 0, 0.05);
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  flex-wrap: wrap;
  gap: 16px;

  .header-left {
    flex: 1;

    .favorite-title {
      display: flex;
      align-items: center;
      gap: 12px;
      margin-bottom: 8px;

      h1 {
        margin: 0;
        font-size: 24px;
        color: #303133;
      }
    }

    .favorite-meta {
      margin: 0;
      color: #909399;
      font-size: 14px;
    }
  }

  .header-actions {
    display: flex;
    gap: 12px;
    flex-wrap: wrap;
  }
}

.resource-list {
  background: #fff;
  border-radius: 8px;
  padding: 24px;
  box-shadow: 0 2px 12px 0 rgba(0, 0, 0, 0.05);
}

.resource-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
}

.resource-card {
  transition: all 0.3s;

  &:hover {
    transform: translateY(-2px);
  }

  :deep(.el-card__body) {
    padding: 16px;
  }
}

.resource-content {
  display: flex;
  gap: 12px;
  cursor: pointer;
}

.resource-type-icon {
  width: 48px;
  height: 48px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  font-size: 12px;
  font-weight: bold;
  flex-shrink: 0;
}

.resource-info {
  flex: 1;
  min-width: 0;

  .resource-title {
    margin: 0 0 4px;
    font-size: 14px;
    font-weight: 600;
    color: #303133;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .resource-course {
    margin: 0 0 8px;
    font-size: 12px;
    color: #606266;
  }

  .resource-tags {
    display: flex;
    gap: 4px;
    margin-bottom: 8px;
    flex-wrap: wrap;
  }

  .resource-stats {
    display: flex;
    gap: 12px;
    font-size: 12px;
    color: #909399;

    span {
      display: flex;
      align-items: center;
      gap: 2px;
    }
  }
}

.resource-actions {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid #ebeef5;
  text-align: right;
}

@media (max-width: 768px) {
  .detail-header {
    .header-actions {
      width: 100%;

      .el-button {
        flex: 1;
      }
    }
  }

  .resource-grid {
    grid-template-columns: 1fr;
  }
}
</style>
