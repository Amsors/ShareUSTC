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
        <p class="favorite-meta">共 {{ resourceCount }} 个资源 · 创建于 {{ createdAt }}</p>
        <p v-if="currentFavorite?.id" class="favorite-uuid">收藏夹ID: {{ currentFavorite.id }}</p>
      </div>
      <div class="header-actions">
        <el-button @click="showEditModal = true">
          <el-icon><Edit /></el-icon>
          重命名
        </el-button>
        <el-button
          :type="isDefaultFavorite(currentFavorite?.id || '') ? 'success' : 'default'"
          @click="handleSetDefault"
        >
          <el-icon><Star /></el-icon>
          {{ isDefaultFavorite(currentFavorite?.id || '') ? '取消默认' : '设为默认' }}
        </el-button>

        <!-- 下载按钮组 -->
        <el-dropdown
          v-if="resourceCount > 0"
          split-button
          type="primary"
          @click="handleBrowserDownloadClick"
          @command="handleDownloadCommand"
        >
          <el-icon><ChromeFilled /></el-icon>
          浏览器打包下载
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="browser">
                <el-icon><ChromeFilled /></el-icon>
                浏览器打包下载
                <el-tag size="small" type="success" effect="plain" style="margin-left: 8px">
                  推荐
                </el-tag>
              </el-dropdown-item>
              <el-dropdown-item command="folder">
                <el-icon><FolderOpened /></el-icon>
                下载到文件夹
              </el-dropdown-item>
              <el-dropdown-item command="server">
                <el-icon><Download /></el-icon>
                服务器打包下载
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
        <el-button v-else type="primary" disabled>
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
      <div v-if="loading" class="loading-container">
        <el-icon class="loading-icon" :size="48"><Loading /></el-icon>
      </div>

      <!-- 空状态 -->
      <el-empty v-else-if="resources.length === 0" description="收藏夹是空的">
        <p>快去浏览资源并添加到收藏夹吧！</p>
        <el-button type="primary" @click="$router.push('/resources')"> 浏览资源 </el-button>
      </el-empty>

      <!-- 资源卡片列表 -->
      <div v-else class="resource-grid">
        <a
          v-for="resource in resources"
          :key="resource.id"
          :href="`/resources/${resource.id}`"
          class="resource-card-link"
          @click.prevent="goToResource(resource.id)"
        >
          <el-card class="resource-card" shadow="hover">
            <div class="resource-content">
              <!-- 资源类型图标 -->
              <div
                class="resource-type-icon"
                :style="{ backgroundColor: getResourceTypeColor(resource.resourceType) }"
              >
                {{ resource.resourceType.toUpperCase() }}
              </div>

              <!-- 存储类型标识 -->
              <div
                class="storage-type-badge"
                :class="resource.storageType"
                :title="resource.storageType === 'oss' ? '云端存储' : '本地存储'"
              >
                {{ resource.storageType === 'oss' ? '云' : '本' }}
              </div>

              <div class="resource-info">
                <h4 class="resource-title">{{ resource.title }}</h4>
                <p v-if="resource.courseName" class="resource-course">
                  {{ resource.courseName }}
                </p>
                <div v-if="resource.tags?.length" class="resource-tags">
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
                  <span v-if="resource.fileSize" class="file-size">
                    <el-icon><Document /></el-icon>
                    {{ formatFileSize(resource.fileSize) }}
                  </span>
                </div>
              </div>
            </div>

            <div class="resource-actions" @click.stop.prevent>
              <el-popconfirm
                title="确定从收藏夹移除此资源？"
                confirm-button-text="移除"
                cancel-button-text="取消"
                @confirm="removeResource(resource.id)"
              >
                <template #reference>
                  <el-button type="danger" text size="small" @click.stop.prevent>
                    <el-icon><Remove /></el-icon>
                    移除
                  </el-button>
                </template>
              </el-popconfirm>
            </div>
          </el-card>
        </a>
      </div>
    </div>

    <!-- 编辑收藏夹弹窗 -->
    <CreateFavoriteModal
      v-if="currentFavorite"
      v-model="showEditModal"
      :favorite="{
        id: currentFavorite.id,
        name: currentFavorite.name,
        resourceCount: currentFavorite.resourceCount,
        createdAt: currentFavorite.createdAt,
      }"
      is-edit
      @success="handleEditSuccess"
    />

    <!-- 浏览器下载进度弹窗 -->
    <BrowserDownloadProgressModal
      v-model="showBrowserDownloadModal"
      :progress="browserDownloadProgress"
      :show-oss-tip="hasMixedStorage"
      @cancel="handleBrowserDownloadCancel"
      @retry="handleBrowserDownloadRetry"
      @close="handleBrowserDownloadClose"
    />

    <!-- 下载到文件夹进度弹窗 -->
    <FolderDownloadProgressModal
      v-model="showFolderDownloadModal"
      :progress="folderDownloadProgress"
      @cancel="handleFolderDownloadCancel"
      @retry="handleFolderDownloadRetry"
      @close="handleFolderDownloadClose"
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
  Remove,
  Loading,
  ChromeFilled,
  Document,
  FolderOpened,
} from '@element-plus/icons-vue';
import { useDefaultFavorite } from '../../composables/useDefaultFavorite';
import { useFavoriteStore } from '../../stores/favorite';
import { getErrorMessage, isHandledError } from '@/api/request';
import logger from '@/utils/logger';
import { downloadFavorite } from '../../api/favorite';
import {
  browserDownloadFavorite,
  checkBrowserSupport,
  downloadToFolder,
  checkFileSystemAccessSupport,
  type DownloadProgress,
  type FolderDownloadProgress,
} from '../../utils/browserZip';
import CreateFavoriteModal from '../../components/favorite/CreateFavoriteModal.vue';
import BrowserDownloadProgressModal from '../../components/favorite/BrowserDownloadProgressModal.vue';
import FolderDownloadProgressModal from '../../components/favorite/FolderDownloadProgressModal.vue';

const route = useRoute();
const router = useRouter();
const favoriteStore = useFavoriteStore();
const { isDefaultFavorite, setDefaultFavorite, clearDefaultFavorite } = useDefaultFavorite();

// 从 route 获取收藏夹ID
const favoriteId = computed(() => route.params.id as string);

// 状态
const loading = ref(false);
const downloading = ref(false);
const showEditModal = ref(false);

// 浏览器下载相关状态
const showBrowserDownloadModal = ref(false);
const isBrowserDownloading = ref(false);
const browserDownloadProgress = ref<DownloadProgress>({
  currentFile: '',
  currentIndex: 0,
  totalFiles: 0,
  percent: 0,
  status: 'downloading',
  cachedCount: 0,
  downloadedCount: 0,
  failedCount: 0,
});

// 下载到文件夹相关状态
const showFolderDownloadModal = ref(false);
const isFolderDownloading = ref(false);
const folderDownloadProgress = ref<FolderDownloadProgress>({
  currentFile: '',
  currentIndex: 0,
  totalFiles: 0,
  percent: 0,
  status: 'selecting',
  savedCount: 0,
  failedCount: 0,
});

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

// 检查是否包含混合存储
const hasMixedStorage = computed(() => {
  const resList = resources.value;
  if (resList.length === 0) return false;
  const hasOss = resList.some((r) => r.storageType === 'oss');
  const hasLocal = resList.some((r) => r.storageType === 'local');
  return hasOss && hasLocal;
});

// 获取资源类型颜色
const getResourceTypeColor = (type: string) => {
  const colorMap: Record<string, string> = {
    pdf: '#F56C6C',
    ppt: '#E6A23C',
    pptx: '#E6A23C',
    doc: '#409EFF',
    docx: '#409EFF',
    web_markdown: '#67C23A',
    txt: '#909399',
    zip: '#909399',
  };
  return colorMap[type] || '#909399';
};

// 格式化文件大小
const formatFileSize = (bytes?: number): string => {
  if (!bytes) return '-';
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
};

// 获取收藏夹详情
const fetchDetail = async () => {
  loading.value = true;
  try {
    await favoriteStore.fetchFavoriteDetail(favoriteId.value);
  } catch (error) {
    logger.error('[FavoriteDetail]', '获取收藏夹详情失败', error);
    if (!isHandledError(error)) {
      ElMessage.error('获取收藏夹详情失败');
    }
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
  } catch (error) {
    logger.error('[FavoriteDetail]', '移除资源失败', error);
    if (!isHandledError(error)) {
      ElMessage.error(getErrorMessage(error, '移除失败'));
    }
  }
};

// 编辑成功回调
const handleEditSuccess = () => {
  showEditModal.value = false;
  fetchDetail();
  ElMessage.success('更新成功');
};

// 处理下载命令
const handleDownloadCommand = (command: string) => {
  if (command === 'browser') {
    handleBrowserDownloadClick();
  } else if (command === 'folder') {
    handleFolderDownloadClick();
  } else if (command === 'server') {
    handleServerDownloadClick();
  }
};

// 计算收藏夹资源总大小
const calculateTotalSize = (): number => {
  return resources.value.reduce((total, resource) => total + (resource.fileSize || 0), 0);
};

// 浏览器打包下载点击（显示二次确认）
const handleBrowserDownloadClick = async () => {
  if (resourceCount.value === 0) {
    ElMessage.warning('收藏夹为空，无法下载');
    return;
  }

  // 检查浏览器支持
  const support = checkBrowserSupport();
  if (!support.supported) {
    ElMessage.error(support.reason || '您的浏览器不支持浏览器打包下载');
    return;
  }

  const totalSize = calculateTotalSize();
  const sizeText = totalSize > 0 ? formatFileSize(totalSize) : '未知大小';

  try {
    await ElMessageBox.confirm(
      `即将打包下载 ${resourceCount.value} 个资源，预计压缩包大小：${sizeText}。\n\n下载过程中请勿关闭页面。`,
      '确认打包下载',
      {
        confirmButtonText: '开始下载',
        cancelButtonText: '取消',
        type: 'info',
        dangerouslyUseHTMLString: false,
      }
    );
    // 用户确认后开始下载
    handleBrowserDownload();
  } catch {
    // 用户取消，不做任何操作
  }
};

// 服务器打包下载点击（显示二次确认）
const handleServerDownloadClick = async () => {
  if (resourceCount.value === 0) {
    ElMessage.warning('收藏夹为空，无法下载');
    return;
  }

  const totalSize = calculateTotalSize();
  const sizeText = totalSize > 0 ? formatFileSize(totalSize) : '未知大小';

  try {
    await ElMessageBox.confirm(
      `即将使用服务器打包下载 ${resourceCount.value} 个资源，预计压缩包大小：${sizeText}。`,
      '确认打包下载',
      {
        confirmButtonText: '开始下载',
        cancelButtonText: '取消',
        type: 'info',
      }
    );
    // 用户确认后开始下载
    handleServerDownload();
  } catch {
    // 用户取消，不做任何操作
  }
};

// 服务器打包下载（实际执行）
const handleServerDownload = async () => {
  downloading.value = true;
  try {
    await downloadFavorite(favoriteId.value, currentFavorite.value?.name);
    ElMessage.success('开始下载');
  } catch (error) {
    logger.error('[FavoriteDetail]', '服务器打包下载失败', error);
    if (!isHandledError(error)) {
      ElMessage.error(getErrorMessage(error, '下载失败'));
    }
  } finally {
    downloading.value = false;
  }
};

// 浏览器打包下载（实际执行）
const handleBrowserDownload = async () => {
  // 检查是否已在下载中
  if (isBrowserDownloading.value) {
    ElMessage.warning('下载正在进行中，请稍候');
    return;
  }

  // 重置状态
  isBrowserDownloading.value = true;
  browserDownloadProgress.value = {
    currentFile: '',
    currentIndex: 0,
    totalFiles: resources.value.length,
    percent: 0,
    status: 'downloading',
    cachedCount: 0,
    downloadedCount: 0,
    failedCount: 0,
  };
  showBrowserDownloadModal.value = true;

  try {
    await browserDownloadFavorite(
      resources.value,
      currentFavorite.value?.name || '收藏夹',
      (progress) => {
        browserDownloadProgress.value = progress;
      }
    );

    ElMessage.success('浏览器打包下载完成');
  } catch (error) {
    logger.error('[FavoriteDetail]', '浏览器打包下载失败', error);
    browserDownloadProgress.value = {
      ...browserDownloadProgress.value,
      status: 'error',
      error: getErrorMessage(error, '下载失败'),
    };
    if (!isHandledError(error)) {
      ElMessage.error(getErrorMessage(error, '浏览器打包下载失败'));
    }
  } finally {
    isBrowserDownloading.value = false;
  }
};

// 取消浏览器下载
const handleBrowserDownloadCancel = () => {
  // 注意：由于 fetch 无法真正取消，这里只是关闭弹窗
  // 实际的下载会在后台继续进行，但不会再更新 UI
  isBrowserDownloading.value = false;
  showBrowserDownloadModal.value = false;
  ElMessage.info('已取消下载');
};

// 重试浏览器下载
const handleBrowserDownloadRetry = () => {
  showBrowserDownloadModal.value = false;
  // 延迟一点再显示确认弹窗
  setTimeout(() => {
    handleBrowserDownloadClick();
  }, 300);
};

// 关闭浏览器下载弹窗
const handleBrowserDownloadClose = () => {
  showBrowserDownloadModal.value = false;
};

// 下载到文件夹点击（显示二次确认）
const handleFolderDownloadClick = async () => {
  if (resourceCount.value === 0) {
    ElMessage.warning('收藏夹为空，无法下载');
    return;
  }

  // 检查浏览器支持
  const support = checkFileSystemAccessSupport();
  if (!support.supported) {
    ElMessageBox.alert(support.reason || '您的浏览器不支持文件夹选择功能', '浏览器不支持', {
      confirmButtonText: '知道了',
      type: 'warning',
    });
    return;
  }

  const totalSize = calculateTotalSize();
  const sizeText = totalSize > 0 ? formatFileSize(totalSize) : '未知大小';

  try {
    await ElMessageBox.confirm(
      `即将将 ${resourceCount.value} 个资源直接下载到您选择的文件夹中，预计总大小：${sizeText}。\n\n下载过程中请勿关闭页面，首次下载可能需要您授权访问文件夹。`,
      '确认下载到文件夹',
      {
        confirmButtonText: '选择文件夹并下载',
        cancelButtonText: '取消',
        type: 'info',
      }
    );
    // 用户确认后开始下载
    handleFolderDownload();
  } catch {
    // 用户取消，不做任何操作
  }
};

// 下载到文件夹（实际执行）
const handleFolderDownload = async () => {
  // 检查是否已在下载中
  if (isFolderDownloading.value) {
    ElMessage.warning('下载正在进行中，请稍候');
    return;
  }

  // 重置状态
  isFolderDownloading.value = true;
  folderDownloadProgress.value = {
    currentFile: '',
    currentIndex: 0,
    totalFiles: resources.value.length,
    percent: 0,
    status: 'selecting',
    savedCount: 0,
    failedCount: 0,
  };
  showFolderDownloadModal.value = true;

  try {
    await downloadToFolder(resources.value, (progress) => {
      folderDownloadProgress.value = progress;
    });

    if (folderDownloadProgress.value.status !== 'cancelled') {
      ElMessage.success('文件已成功保存到文件夹');
    }
  } catch (error) {
    const message = getErrorMessage(error, '');
    folderDownloadProgress.value = {
      ...folderDownloadProgress.value,
      status: 'error',
      error: message || '下载失败',
    };
    if (message && !message.includes('取消')) {
      logger.error('[FavoriteDetail]', '下载到文件夹失败', error);
      if (!isHandledError(error)) {
        ElMessage.error(message || '下载到文件夹失败');
      }
    }
  } finally {
    isFolderDownloading.value = false;
  }
};

// 取消下载到文件夹
const handleFolderDownloadCancel = () => {
  isFolderDownloading.value = false;
  showFolderDownloadModal.value = false;
  ElMessage.info('已取消下载');
};

// 重试下载到文件夹
const handleFolderDownloadRetry = () => {
  showFolderDownloadModal.value = false;
  // 延迟一点再显示确认弹窗
  setTimeout(() => {
    handleFolderDownloadClick();
  }, 300);
};

// 关闭下载到文件夹弹窗
const handleFolderDownloadClose = () => {
  showFolderDownloadModal.value = false;
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
        type: 'warning',
      }
    );

    await favoriteStore.deleteFavorite(favoriteId.value);

    // 如果删除的是默认收藏夹，清除默认收藏夹设置
    if (isDefaultFavorite(favoriteId.value)) {
      clearDefaultFavorite();
    }

    ElMessage.success('删除成功');
    router.push('/favorites');
  } catch (error) {
    if (error !== 'cancel') {
      logger.error('[FavoriteDetail]', '删除收藏夹失败', error);
      if (!isHandledError(error)) {
        ElMessage.error(getErrorMessage(error, '删除失败'));
      }
    }
  }
};

// 设置/取消默认收藏夹
const handleSetDefault = async () => {
  if (!currentFavorite.value) return;

  if (isDefaultFavorite(currentFavorite.value.id)) {
    // 取消默认
    setDefaultFavorite('', '');
    ElMessage.success('已取消默认收藏夹');
  } else {
    // 设为默认
    setDefaultFavorite(currentFavorite.value.id, currentFavorite.value.name);
    ElMessage.success(`已将 "${currentFavorite.value.name}" 设为默认收藏夹`);
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

    .favorite-uuid {
      margin: 4px 0 0;
      color: #c0c4cc;
      font-size: 12px;
      font-family: monospace;
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

.loading-container {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 300px;

  .loading-icon {
    animation: spin 1s linear infinite;
  }
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.resource-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 16px;
}

.resource-card-link {
  text-decoration: none;
  color: inherit;
  display: block;
}

.resource-card {
  transition: all 0.3s;
  position: relative;
  height: 150px; // 固定卡片高度

  .resource-card-link:hover & {
    transform: translateY(-2px);
    box-shadow: 0 4px 16px 0 rgba(0, 0, 0, 0.1);
  }

  :deep(.el-card__body) {
    padding: 16px;
    height: 100%;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
  }
}

.resource-content {
  display: flex;
  gap: 12px;
  cursor: pointer;
  position: relative;
  flex: 1;
  min-height: 0; // 允许 flex 子项收缩
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

.storage-type-badge {
  position: absolute;
  top: -8px;
  left: -8px;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  font-weight: bold;
  color: #fff;
  z-index: 1;

  &.oss {
    background-color: #67c23a;
  }

  &.local {
    background-color: #909399;
  }
}

.resource-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;

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
    margin: 0 0 4px;
    font-size: 12px;
    color: #606266;
    min-height: 18px; // 固定高度，无内容时占位
  }

  .resource-tags {
    display: flex;
    gap: 4px;
    margin-bottom: 4px;
    flex-wrap: wrap;
    min-height: 24px; // 固定高度，无标签时占位
  }

  .resource-stats {
    display: flex;
    gap: 12px;
    font-size: 12px;
    color: #909399;
    margin-top: auto; // 将统计信息推到底部

    span {
      display: flex;
      align-items: center;
      gap: 2px;
    }

    .file-size {
      color: #409eff;
    }
  }
}

.resource-actions {
  margin-top: auto;
  padding-top: 8px;
  border-top: 1px solid #ebeef5;
  text-align: right;
  flex-shrink: 0;
}

.loading-placeholder {
  min-height: 400px;
}

@media (max-width: 768px) {
  .detail-header {
    .header-actions {
      width: 100%;

      .el-button,
      .el-dropdown {
        flex: 1;
      }
    }
  }

  .resource-grid {
    grid-template-columns: 1fr;
  }
}
</style>
