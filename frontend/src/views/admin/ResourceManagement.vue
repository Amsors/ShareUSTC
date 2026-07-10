<template>
  <div class="resource-management-page">
    <div class="page-header">
      <h1>资料管理</h1>
      <p class="subtitle">管理服务器中的所有资源</p>
    </div>

    <!-- 搜索栏 -->
    <div class="search-bar">
      <el-input
        v-model="searchKeyword"
        placeholder="搜索资源标题或课程名称"
        clearable
        @keyup.enter="handleSearch"
        @clear="handleSearch"
      >
        <template #prefix>
          <el-icon><Search /></el-icon>
        </template>
        <template #append>
          <el-button type="primary" @click="handleSearch">搜索</el-button>
        </template>
      </el-input>
    </div>

    <!-- 收藏夹管理区域 -->
    <el-card v-if="isAdmin" class="favorites-card">
      <template #header>
        <div class="card-header">
          <span>收藏夹管理</span>
          <el-button type="primary" link @click="fetchFavorites">
            <el-icon><Refresh /></el-icon>
            刷新
          </el-button>
        </div>
      </template>

      <el-empty v-if="favorites.length === 0" description="暂无收藏夹" />

      <div v-else class="favorites-list">
        <el-alert type="info" :closable="false" show-icon class="favorites-tip">
          <template #title>
            您可以一键删除自己收藏夹内的所有资源。此操作会永久删除资源文件，不可恢复。
          </template>
        </el-alert>

        <div class="table-wrapper">
          <el-table v-loading="favoritesLoading" :data="favorites" border>
            <el-table-column prop="name" label="收藏夹名称" min-width="180">
              <template #default="{ row }">
                <div class="favorite-name">
                  <el-icon><Folder /></el-icon>
                  <span>{{ row.name }}</span>
                </div>
              </template>
            </el-table-column>
            <el-table-column prop="resourceCount" label="资源数量" width="90" align="center" />
            <el-table-column prop="createdAt" label="创建时间" width="150">
              <template #default="{ row }">
                {{ formatDate(row.createdAt) }}
              </template>
            </el-table-column>
            <el-table-column label="操作" width="130" fixed="right">
              <template #default="{ row }">
                <el-button
                  type="danger"
                  size="small"
                  :disabled="row.resourceCount === 0"
                  @click="handleDeleteFavoriteResources(row)"
                >
                  <el-icon><Delete /></el-icon>
                  清空
                </el-button>
              </template>
            </el-table-column>
          </el-table>
        </div>
      </div>
    </el-card>

    <!-- 资源列表 -->
    <el-card class="resources-card">
      <template #header>
        <div class="card-header">
          <span>资源列表</span>
          <div class="header-actions">
            <el-tag type="info">共 {{ total }} 个资源</el-tag>
            <el-button type="primary" link @click="fetchResources">
              <el-icon><Refresh /></el-icon>
              刷新
            </el-button>
          </div>
        </div>
      </template>

      <div class="table-wrapper">
        <el-table v-loading="loading" :data="resources" border stripe style="width: 100%">
          <el-table-column prop="title" label="资源标题" min-width="200" show-overflow-tooltip>
            <template #default="{ row }">
              <div class="resource-title">
                <el-link type="primary" @click="viewResource(row.id)">
                  {{ row.title }}
                </el-link>
              </div>
            </template>
          </el-table-column>

          <el-table-column
            prop="courseName"
            label="课程名称"
            min-width="140"
            show-overflow-tooltip
          />

          <el-table-column label="类型/分类" width="90" align="center">
            <template #default="{ row }">
              <div class="type-category-cell">
                <el-tag size="small" :type="getResourceTypeType(row.resourceType)">
                  {{ formatResourceType(row.resourceType) }}
                </el-tag>
                <el-tag size="small" effect="plain" class="category-tag">
                  {{ formatCategory(row.category) }}
                </el-tag>
              </div>
            </template>
          </el-table-column>

          <el-table-column prop="auditStatus" label="状态" width="70" align="center">
            <template #default="{ row }">
              <el-tag size="small" :type="getAuditStatusType(row.auditStatus)">
                {{ formatAuditStatus(row.auditStatus) }}
              </el-tag>
            </template>
          </el-table-column>

          <el-table-column prop="uploaderName" label="上传者" width="100" />

          <el-table-column prop="fileSize" label="大小" width="80" align="right">
            <template #default="{ row }">
              {{ formatFileSize(row.fileSize) }}
            </template>
          </el-table-column>

          <el-table-column label="统计" width="130">
            <template #default="{ row }">
              <div class="stats-row compact">
                <span class="stat-item" title="浏览">
                  <el-icon><View /></el-icon> {{ row.views || 0 }}
                </span>
                <span class="stat-item" title="下载">
                  <el-icon><Download /></el-icon> {{ row.downloads || 0 }}
                </span>
                <span class="stat-item" title="点赞">
                  <el-icon><Pointer /></el-icon> {{ row.likes || 0 }}
                </span>
              </div>
            </template>
          </el-table-column>

          <el-table-column prop="createdAt" label="上传时间" width="140">
            <template #default="{ row }">
              {{ formatDate(row.createdAt) }}
            </template>
          </el-table-column>

          <el-table-column label="操作" width="150" fixed="right">
            <template #default="{ row }">
              <el-button
                type="primary"
                size="small"
                :loading="recalculatingId === row.id"
                @click="handleRecalculateHash(row)"
              >
                <el-icon><Refresh /></el-icon>
                Hash
              </el-button>
              <el-button type="danger" size="small" @click="handleDeleteResource(row)">
                <el-icon><Delete /></el-icon>
              </el-button>
            </template>
          </el-table-column>
        </el-table>
      </div>

      <!-- 分页 -->
      <div class="pagination-wrapper">
        <el-pagination
          v-model:current-page="page"
          v-model:page-size="perPage"
          :page-sizes="[10, 20, 50, 100]"
          :total="total"
          layout="total, sizes, prev, pager, next, jumper"
          @size-change="handleSizeChange"
          @current-change="handlePageChange"
        />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { Search, Refresh, Folder, Delete, View, Download, Pointer } from '@element-plus/icons-vue';
import { useAuthStore } from '../../stores/auth';
import {
  getAllResources,
  adminDeleteResource,
  recalculateResourceHash,
  getAdminFavorites,
  deleteAllFavoriteResources,
} from '../../api/admin';
import type { AdminResource, AdminFavorite } from '../../types/admin';
import { getErrorMessage, isHandledError } from '@/api/request';
import logger from '@/utils/logger';

const authStore = useAuthStore();

// Element Plus el-tag 的 type 取值
type ElTagType = 'primary' | 'success' | 'warning' | 'info' | 'danger' | '';

// 检查是否为管理员
const isAdmin = authStore.isAdmin;

// 资源列表数据
const resources = ref<AdminResource[]>([]);
const loading = ref(false);
const page = ref(1);
const perPage = ref(20);
const total = ref(0);
const searchKeyword = ref('');
const recalculatingId = ref<string | null>(null); // 正在重新计算hash的资源ID

// 收藏夹数据
const favorites = ref<AdminFavorite[]>([]);
const favoritesLoading = ref(false);

// 获取资源列表
const fetchResources = async () => {
  loading.value = true;
  try {
    const response = await getAllResources({
      page: page.value,
      perPage: perPage.value,
      keyword: searchKeyword.value || undefined,
    });
    resources.value = response.resources;
    total.value = response.total;
  } catch (error) {
    logger.error('[ResourceManagement]', '获取资源列表失败', error);
    if (!isHandledError(error)) {
      ElMessage.error(getErrorMessage(error, '获取资源列表失败'));
    }
  } finally {
    loading.value = false;
  }
};

// 获取收藏夹列表
const fetchFavorites = async () => {
  favoritesLoading.value = true;
  try {
    const response = await getAdminFavorites();
    favorites.value = response.favorites;
  } catch (error) {
    logger.error('[ResourceManagement]', '获取收藏夹列表失败', error);
    if (!isHandledError(error)) {
      ElMessage.error(getErrorMessage(error, '获取收藏夹列表失败'));
    }
  } finally {
    favoritesLoading.value = false;
  }
};

// 搜索
const handleSearch = () => {
  page.value = 1;
  fetchResources();
};

// 分页
const handlePageChange = (newPage: number) => {
  page.value = newPage;
  fetchResources();
};

const handleSizeChange = (newSize: number) => {
  perPage.value = newSize;
  page.value = 1;
  fetchResources();
};

// 删除资源
const handleDeleteResource = async (resource: AdminResource) => {
  try {
    await ElMessageBox.confirm(
      `确定要删除资源 "${resource.title}" 吗？\n\n此操作将永久删除该资源文件及所有关联数据（包括统计信息、评分、评论等），不可恢复。`,
      '确认删除资源',
      {
        confirmButtonText: '确认删除',
        cancelButtonText: '取消',
        type: 'warning',
        dangerouslyUseHTMLString: false,
      }
    );

    loading.value = true;
    await adminDeleteResource(resource.id);
    ElMessage.success('资源删除成功');
    fetchResources();
  } catch (error) {
    if (error !== 'cancel') {
      logger.error('[ResourceManagement]', '删除资源失败', error);
      if (!isHandledError(error)) {
        ElMessage.error(getErrorMessage(error, '删除失败'));
      }
    }
  } finally {
    loading.value = false;
  }
};

// 删除收藏夹内所有资源
const handleDeleteFavoriteResources = async (favorite: AdminFavorite) => {
  try {
    await ElMessageBox.confirm(
      `确定要删除收藏夹 "${favorite.name}" 中的所有资源吗？\n\n共有 ${favorite.resourceCount} 个资源将被永久删除，此操作不可恢复。`,
      '确认删除收藏夹内所有资源',
      {
        confirmButtonText: '确认删除',
        cancelButtonText: '取消',
        type: 'warning',
        dangerouslyUseHTMLString: false,
      }
    );

    favoritesLoading.value = true;
    const result = await deleteAllFavoriteResources(favorite.id);
    ElMessage.success(`成功删除 ${result.deletedCount} 个资源`);
    fetchFavorites();
    fetchResources();
  } catch (error) {
    if (error !== 'cancel') {
      logger.error('[ResourceManagement]', '删除收藏夹资源失败', error);
      if (!isHandledError(error)) {
        ElMessage.error(getErrorMessage(error, '删除失败'));
      }
    }
  } finally {
    favoritesLoading.value = false;
  }
};

// 查看资源详情
const viewResource = (resourceId: string) => {
  window.open(`/resources/${resourceId}`, '_blank');
};

// 重新计算资源hash
const handleRecalculateHash = async (resource: AdminResource) => {
  try {
    await ElMessageBox.confirm(
      `确定要重新计算资源 "${resource.title}" 的哈希值吗？\n\n此操作会重新读取文件内容并计算SHA-256哈希值。`,
      '确认重新计算Hash',
      {
        confirmButtonText: '确认',
        cancelButtonText: '取消',
        type: 'info',
      }
    );

    recalculatingId.value = resource.id;
    const result = await recalculateResourceHash(resource.id);

    // 显示详细结果
    const oldHashDisplay = result.oldHash ? `${result.oldHash.substring(0, 16)}...` : '无';
    const newHashDisplay = `${result.newHash.substring(0, 16)}...`;

    ElMessage.success({
      message: `Hash重新计算成功！\n原Hash: ${oldHashDisplay}\n新Hash: ${newHashDisplay}`,
      duration: 5000,
      showClose: true,
    });

    // 刷新列表以更新显示
    fetchResources();
  } catch (error) {
    if (error !== 'cancel') {
      logger.error('[ResourceManagement]', '重新计算 Hash 失败', error);
      if (!isHandledError(error)) {
        ElMessage.error(getErrorMessage(error, '重新计算Hash失败'));
      }
    }
  } finally {
    recalculatingId.value = null;
  }
};

// 格式化日期
const formatDate = (dateStr: string) => {
  if (!dateStr) return '-';
  const date = new Date(dateStr.endsWith('Z') ? dateStr : `${dateStr}Z`);
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  });
};

// 格式化文件大小
const formatFileSize = (bytes?: number) => {
  if (bytes === undefined || bytes === null) return '-';
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

// 格式化资源类型
const formatResourceType = (type: string) => {
  const typeMap: Record<string, string> = {
    web_markdown: 'Markdown',
    pdf: 'PDF',
    ppt: 'PPT',
    pptx: 'PPTX',
    doc: 'DOC',
    docx: 'DOCX',
    txt: 'TXT',
    zip: 'ZIP',
    image: '图片',
    jpeg: 'JPEG',
    jpg: 'JPG',
    png: 'PNG',
  };
  return typeMap[type] || type.toUpperCase();
};

// 获取资源类型标签样式
const getResourceTypeType = (type: string) => {
  const typeMap: Record<string, ElTagType> = {
    web_markdown: 'primary',
    pdf: 'danger',
    ppt: 'warning',
    pptx: 'warning',
    doc: 'info',
    docx: 'info',
    txt: '',
    zip: 'success',
  };
  return typeMap[type] || '';
};

// 格式化分类
const formatCategory = (category: string) => {
  const categoryMap: Record<string, string> = {
    exam: '试题',
    note: '笔记',
    slides: '课件',
    other: '其他',
  };
  return categoryMap[category] || category;
};

// 格式化审核状态
const formatAuditStatus = (status: string) => {
  const statusMap: Record<string, string> = {
    pending: '待审核',
    approved: '已通过',
    rejected: '已拒绝',
  };
  return statusMap[status] || status;
};

// 获取审核状态标签样式
const getAuditStatusType = (status: string) => {
  const typeMap: Record<string, ElTagType> = {
    pending: 'warning',
    approved: 'success',
    rejected: 'danger',
  };
  return typeMap[status] || 'info';
};

// 监听分页变化
watch([page, perPage], () => {
  fetchResources();
});

// 页面加载时获取数据
onMounted(() => {
  fetchResources();
  if (isAdmin) {
    fetchFavorites();
  }
});
</script>

<style scoped lang="scss">
.resource-management-page {
  padding: 20px;
}

.page-header {
  margin-bottom: 24px;

  h1 {
    margin: 0 0 8px;
    font-size: 24px;
    color: #303133;
  }

  .subtitle {
    margin: 0;
    color: #909399;
    font-size: 14px;
  }
}

.search-bar {
  margin-bottom: 20px;
  max-width: 500px;
}

.favorites-card {
  margin-bottom: 20px;

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
}

.favorites-tip {
  margin-bottom: 16px;
}

.favorite-name {
  display: flex;
  align-items: center;
  gap: 8px;

  .el-icon {
    color: #409eff;
  }
}

.resources-card {
  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;

    .header-actions {
      display: flex;
      align-items: center;
      gap: 12px;
    }
  }
}

.resource-title {
  .el-link {
    font-weight: 500;
  }
}

// 操作列按钮样式
:deep(.el-table__row) {
  .el-button {
    margin-left: 0;
    margin-right: 8px;

    &:last-child {
      margin-right: 0;
    }
  }
}

.stats-row {
  display: flex;
  gap: 16px;

  .stat-item {
    display: flex;
    align-items: center;
    gap: 4px;
    color: #606266;
    font-size: 13px;

    .el-icon {
      font-size: 14px;
    }
  }
}

.pagination-wrapper {
  margin-top: 20px;
  display: flex;
  justify-content: flex-end;
}

.favorites-list {
  .el-table {
    margin-top: 16px;
  }
}

// 表格横向滚动支持
.table-wrapper {
  width: 100%;
  overflow-x: auto;

  :deep(.el-table) {
    min-width: 900px;
  }
}

// 类型/分类合并单元格样式
.type-category-cell {
  display: flex;
  flex-direction: column;
  gap: 4px;
  align-items: center;

  .category-tag {
    font-size: 11px;
  }
}

// 紧凑统计行样式
.stats-row.compact {
  gap: 8px;

  .stat-item {
    font-size: 12px;
  }
}

// 响应式优化
@media (max-width: 1200px) {
  .resource-management-page {
    padding: 16px;
  }
}

@media (max-width: 768px) {
  .resource-management-page {
    padding: 12px;
  }

  .search-bar {
    max-width: 100%;
  }

  .table-wrapper {
    :deep(.el-table) {
      min-width: 800px;
    }
  }
}
</style>
