<template>
  <div class="resource-detail-page">
    <div v-if="loading" class="loading-container">
      <el-skeleton :rows="10" animated />
    </div>

    <div v-else-if="!resource" class="error-container">
      <el-empty description="资源不存在或已被删除" />
      <el-button type="primary" @click="goBack">返回列表</el-button>
    </div>

    <template v-else>
      <!-- 头部信息 -->
      <el-card class="resource-header-card" shadow="never">
        <div class="resource-header">
          <div class="header-left">
            <div class="resource-tags">
              <el-tag :type="getResourceTypeTagType(resource.resourceType)" size="large">
                {{ ResourceTypeLabels[resource.resourceType as keyof typeof ResourceTypeLabels] || resource.resourceType }}
              </el-tag>
              <el-tag type="info" size="large">
                {{ ResourceCategoryLabels[resource.category as ResourceCategoryType] || resource.category }}
              </el-tag>
            </div>

            <h1 class="resource-title">{{ resource.title }}</h1>

            <div v-if="resource.courseName" class="resource-course">
              <el-icon><Reading /></el-icon>
              {{ resource.courseName }}
            </div>

            <div class="resource-meta">
              <span class="meta-item">
                <el-avatar :size="24" :icon="UserFilled" />
                {{ resource.uploaderName || '未知用户' }}
              </span>
              <span class="meta-item">
                <el-icon><Clock /></el-icon>
                {{ formatTime(resource.createdAt) }}
              </span>
            </div>
          </div>

          <div class="header-right">
            <div class="action-buttons">
              <el-button type="primary" size="large" :loading="downloading" @click="handleDownload">
                <el-icon><Download /></el-icon>
                下载资源
              </el-button>

              <LikeButton v-if="authStore.isAuthenticated" :resource-id="resourceId" @update="onLikeUpdate" />

              <el-button v-if="authStore.isAuthenticated" size="large" @click="showAddToFavorite = true">
                <el-icon><Folder /></el-icon>
                收藏
              </el-button>

              <el-button size="large" v-if="canDelete" type="danger" @click="handleDelete">
                <el-icon><Delete /></el-icon>
                删除
              </el-button>
            </div>

            <div class="resource-stats">
              <div class="stat-item">
                <el-icon><View /></el-icon>
                <span class="stat-value">{{ resource.stats.views }}</span>
                <span class="stat-label">浏览</span>
              </div>
              <div class="stat-item">
                <el-icon><Download /></el-icon>
                <span class="stat-value">{{ resource.stats.downloads }}</span>
                <span class="stat-label">下载</span>
              </div>
              <div class="stat-item">
                <el-icon><Star /></el-icon>
                <span class="stat-value">{{ resource.stats.likes }}</span>
                <span class="stat-label">收藏</span>
              </div>
            </div>
          </div>
        </div>
      </el-card>

      <!-- 主体内容 -->
      <div class="resource-content">
        <el-row :gutter="24">
          <!-- 左侧：预览和描述 -->
          <el-col :xs="24" :lg="16">
            <el-card class="preview-card" shadow="never">
              <template #header>
                <span>资源预览</span>
              </template>

              <div class="preview-content">
                <!-- 使用 PreviewSwitch 组件显示预览 -->
                <PreviewSwitch
                  :resource-id="resourceId"
                  :resource-type="resource.resourceType"
                  :resource-title="resource.title"
                />
              </div>
            </el-card>

            <!-- 资源描述 -->
            <el-card v-if="resource.description" class="description-card" shadow="never">
              <template #header>
                <span>资源描述</span>
              </template>
              <div class="description-content">{{ resource.description }}</div>
            </el-card>

            <!-- 评论区域 -->
            <el-card class="comments-card" shadow="never">
              <CommentSection :resource-id="resourceId" />
            </el-card>
          </el-col>

          <!-- 右侧：标签和推荐 -->
          <el-col :xs="24" :lg="8">
            <!-- 标签 -->
            <el-card v-if="resource.tags && resource.tags.length > 0" class="tags-card" shadow="never">
              <template #header>
                <span>标签</span>
              </template>
              <div class="tags-list">
                <el-tag
                  v-for="tag in resource.tags"
                  :key="tag"
                  class="tag-item"
                  effect="plain"
                >
                  {{ tag }}
                </el-tag>
              </div>
            </el-card>

            <!-- 资源信息 -->
            <el-card class="info-card" shadow="never">
              <template #header>
                <span>资源信息</span>
              </template>
              <div class="info-list">
                <div class="info-item">
                  <span class="info-label">文件大小</span>
                  <span class="info-value">{{ formatFileSize(resource.fileSize) }}</span>
                </div>
                <div class="info-item">
                  <span class="info-label">上传时间</span>
                  <span class="info-value">{{ formatDate(resource.createdAt) }}</span>
                </div>
                <div class="info-item">
                  <span class="info-label">更新时间</span>
                  <span class="info-value">{{ formatDate(resource.updatedAt) }}</span>
                </div>
                <div class="info-item">
                  <span class="info-label">文件类型</span>
                  <span class="info-value">{{ resource.resourceType }}</span>
                </div>
              </div>
            </el-card>

            <!-- 评分信息 -->
            <el-card v-if="resource.stats.ratingCount > 0" class="rating-card" shadow="never">
              <template #header>
                <span>评分</span>
              </template>
              <div class="rating-content">
                <div v-if="resource.stats.avgQuality" class="rating-item">
                  <span class="rating-label">质量</span>
                  <el-rate
                    :model-value="resource.stats.avgQuality / 2"
                    disabled
                    show-score
                    :score-template="`${(resource.stats.avgQuality / 2).toFixed(1)}`"
                  />
                </div>
              </div>
            </el-card>
          </el-col>
        </el-row>
      </div>

      <!-- 添加到收藏夹弹窗 -->
      <AddToFavoriteModal
        v-if="authStore.isAuthenticated"
        v-model="showAddToFavorite"
        :resource-id="resourceId"
        @success="onAddToFavoriteSuccess"
      />
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { ElMessage, ElMessageBox } from 'element-plus';
import {
  Reading,
  UserFilled,
  Clock,
  Download,
  Star,
  Delete,
  View,
  Folder
} from '@element-plus/icons-vue';
import { getResourceDetail, downloadResource, deleteResource } from '../../api/resource';
import { useAuthStore } from '../../stores/auth';
import PreviewSwitch from '../../components/preview/PreviewSwitch.vue';
import LikeButton from '../../components/interaction/LikeButton.vue';
import CommentSection from '../../components/interaction/CommentSection.vue';
import AddToFavoriteModal from '../../components/favorite/AddToFavoriteModal.vue';
import {
  ResourceTypeLabels,
  ResourceCategoryLabels,
  formatFileSize,
  type ResourceDetail,
  type ResourceCategoryType
} from '../../types/resource';

const route = useRoute();
const router = useRouter();
const authStore = useAuthStore();

// 状态
const loading = ref(true);
const downloading = ref(false);
const resource = ref<ResourceDetail | null>(null);
const showAddToFavorite = ref(false);

// 计算属性
const resourceId = computed(() => route.params.id as string);

const canDelete = computed(() => {
  if (!resource.value || !authStore.user) return false;
  return resource.value.uploaderId === authStore.user.id || authStore.user.role === 'admin';
});

// 获取资源类型标签类型
const getResourceTypeTagType = (type: string) => {
  const typeMap: Record<string, string> = {
    pdf: 'danger',
    ppt: 'warning',
    pptx: 'warning',
    doc: 'primary',
    docx: 'primary',
    web_markdown: 'success',
    zip: 'info'
  };
  return typeMap[type] || 'info';
};

// 格式化时间（服务器返回的是 UTC 时间，需要转换为本地时间显示）
const formatTime = (time: string) => {
  // 将无时区的时间字符串视为 UTC 时间
  const utcTimeString = time.endsWith('Z') ? time : `${time}Z`;
  const date = new Date(utcTimeString);
  const now = new Date();
  const diff = now.getTime() - date.getTime();

  if (diff < 60 * 60 * 1000) {
    const minutes = Math.floor(diff / (60 * 1000));
    return minutes < 1 ? '刚刚' : `${minutes}分钟前`;
  }
  if (diff < 24 * 60 * 60 * 1000) {
    return `${Math.floor(diff / (60 * 60 * 1000))}小时前`;
  }
  if (diff < 7 * 24 * 60 * 60 * 1000) {
    return `${Math.floor(diff / (24 * 60 * 60 * 1000))}天前`;
  }
  return date.toLocaleDateString('zh-CN');
};

// 收藏状态更新处理
const onLikeUpdate = (_isLiked: boolean, count: number) => {
  if (resource.value) {
    resource.value.stats.likes = count;
  }
};

// 添加到收藏夹成功回调
const onAddToFavoriteSuccess = () => {
  // 可以在这里添加一些UI反馈，比如显示资源已被收藏的提示
};

// 格式化日期（服务器返回的是 UTC 时间，需要转换为本地时间显示）
const formatDate = (time: string) => {
  // 将无时区的时间字符串视为 UTC 时间
  const utcTimeString = time.endsWith('Z') ? time : `${time}Z`;
  const date = new Date(utcTimeString);
  return date.toLocaleString('zh-CN');
};

// 加载资源详情
const loadResourceDetail = async () => {
  loading.value = true;
  try {
    const response = await getResourceDetail(resourceId.value);
    resource.value = response;
  } catch (error: any) {
    if (!error.isHandled) {
      ElMessage.error(error.message || '加载资源详情失败');
    }
    resource.value = null;
  } finally {
    loading.value = false;
  }
};

// 下载资源
const handleDownload = async () => {
  if (!resource.value) return;

  downloading.value = true;
  try {
    await downloadResource(resourceId.value, resource.value.title);
    ElMessage.success('开始下载');
    // 更新下载次数
    resource.value.stats.downloads++;
  } catch (error: any) {
    if (!error.isHandled) {
      ElMessage.error(error.message || '下载失败');
    }
  } finally {
    downloading.value = false;
  }
};

// 删除资源
const handleDelete = async () => {
  if (!resource.value) return;

  try {
    await ElMessageBox.confirm(
      '确定要删除这个资源吗？此操作不可恢复。',
      '删除确认',
      {
        confirmButtonText: '删除',
        cancelButtonText: '取消',
        type: 'warning'
      }
    );

    await deleteResource(resourceId.value);
    ElMessage.success('删除成功');
    router.push('/resources');
  } catch (error: any) {
    if (error !== 'cancel' && !error.isHandled) {
      ElMessage.error(error.message || '删除失败');
    }
  }
};

// 返回列表
const goBack = () => {
  router.push('/resources');
};

// 页面加载时获取资源详情
onMounted(() => {
  loadResourceDetail();
});
</script>

<style scoped>
.resource-detail-page {
  max-width: 1200px;
  margin: 0 auto;
  padding: 24px;
}

.loading-container,
.error-container {
  padding: 60px 0;
  text-align: center;
}

.resource-header-card {
  margin-bottom: 24px;
}

.resource-header {
  display: flex;
  justify-content: space-between;
  gap: 24px;
}

.header-left {
  flex: 1;
}

.resource-tags {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
}

.resource-title {
  font-size: 24px;
  font-weight: 600;
  margin: 0 0 12px 0;
  color: var(--el-text-color-primary);
  line-height: 1.4;
}

.resource-course {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  color: var(--el-text-color-secondary);
  margin-bottom: 16px;
}

.resource-meta {
  display: flex;
  gap: 24px;
}

.meta-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  color: var(--el-text-color-secondary);
}

.header-right {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 24px;
}

.action-buttons {
  display: flex;
  gap: 12px;
}

.resource-stats {
  display: flex;
  gap: 24px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.stat-value {
  font-size: 20px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.stat-label {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.resource-content {
  margin-top: 24px;
}

.preview-card,
.description-card,
.tags-card,
.info-card,
.rating-card,
.comments-card {
  margin-bottom: 24px;
}

.preview-content {
  min-height: 400px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.preview-placeholder,
.no-preview {
  text-align: center;
}

.preview-icon {
  font-size: 64px;
  color: var(--el-text-color-secondary);
  margin-bottom: 16px;
}

.preview-placeholder p,
.no-preview p {
  color: var(--el-text-color-secondary);
  margin-bottom: 16px;
}

.description-content {
  line-height: 1.8;
  color: var(--el-text-color-regular);
  white-space: pre-wrap;
}

.tags-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.tag-item {
  margin: 0;
}

.info-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.info-item {
  display: flex;
  justify-content: space-between;
  font-size: 14px;
}

.info-label {
  color: var(--el-text-color-secondary);
}

.info-value {
  color: var(--el-text-color-primary);
}

.rating-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.rating-item {
  display: flex;
  align-items: center;
  gap: 12px;
}

.rating-label {
  font-size: 14px;
  color: var(--el-text-color-secondary);
  min-width: 60px;
}

@media (max-width: 768px) {
  .resource-header {
    flex-direction: column;
  }

  .header-right {
    align-items: flex-start;
  }

  .resource-meta {
    flex-wrap: wrap;
  }

  .action-buttons {
    flex-wrap: wrap;
  }
}
</style>
