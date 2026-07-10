<template>
  <div class="resource-detail-page">
    <div v-if="loading" class="loading-container">
      <el-icon class="loading-icon" :size="48"><Loading /></el-icon>
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
                {{
                  ResourceTypeLabels[resource.resourceType as keyof typeof ResourceTypeLabels] ||
                  resource.resourceType
                }}
              </el-tag>
              <el-tag type="info" size="large">
                {{
                  ResourceCategoryLabels[resource.category as ResourceCategoryType] ||
                  resource.category
                }}
              </el-tag>
            </div>

            <h1 class="resource-title">{{ resource.title }}</h1>

            <div v-if="resource.courseName" class="resource-course">
              <el-icon><Reading /></el-icon>
              {{ resource.courseName }}
            </div>

            <div class="resource-meta">
              <span class="meta-item uploader-link" @click="goToUploaderHomepage">
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

              <LikeButton
                v-if="authStore.isAuthenticated"
                :resource-id="resourceId"
                @update="onLikeUpdate"
              />

              <el-button
                v-if="authStore.isAuthenticated"
                size="large"
                @click="showAddToFavorite = true"
              >
                <el-icon><Folder /></el-icon>
                收藏
              </el-button>

              <!-- 收藏至默认收藏夹 -->
              <el-tooltip
                v-if="authStore.isAuthenticated"
                :content="tooltipContent"
                placement="bottom"
              >
                <el-button
                  v-if="authStore.isAuthenticated"
                  size="large"
                  :type="defaultFavoriteButtonType"
                  :disabled="!hasDefaultFavorite || isInDefaultFavorite"
                  :loading="addingToDefault || checkingDefaultStatus"
                  @click="addToDefaultFavorite"
                >
                  <el-icon><Star /></el-icon>
                  {{ defaultFavoriteButtonText }}
                </el-button>
              </el-tooltip>

              <el-button v-if="canEditRelations" size="large" @click="showEditRelations = true">
                <el-icon><Link /></el-icon>
                修改信息
              </el-button>

              <el-button v-if="canEdit" size="large" type="success" @click="handleEdit">
                <el-icon><Edit /></el-icon>
                编辑
              </el-button>

              <el-button v-if="canDelete" size="large" type="danger" @click="handleDelete">
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
          <!-- 左侧：描述、预览和评论 -->
          <el-col :xs="24" :lg="16">
            <!-- 资源描述 -->
            <el-card
              v-if="resource.description || canEditDescription"
              class="description-card"
              shadow="never"
            >
              <template #header>
                <div class="description-header">
                  <span>资源描述</span>
                  <el-button
                    v-if="canEditDescription"
                    type="primary"
                    link
                    @click="showEditDescription = true"
                  >
                    <el-icon><Edit /></el-icon>
                    {{ resource.description ? '编辑' : '添加描述' }}
                  </el-button>
                </div>
              </template>
              <div v-if="resource.description" class="description-content">
                {{ resource.description }}
              </div>
            </el-card>

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
                  :file-size="resource.fileSize"
                />
              </div>
            </el-card>

            <!-- 评论区域 -->
            <el-card class="comments-card" shadow="never">
              <CommentSection :resource-id="resourceId" />
            </el-card>
          </el-col>

          <!-- 右侧：标签和推荐 -->
          <el-col :xs="24" :lg="8">
            <!-- 标签 -->
            <el-card
              v-if="resource.tags && resource.tags.length > 0"
              class="tags-card"
              shadow="never"
            >
              <template #header>
                <span>标签</span>
              </template>
              <div class="tags-list">
                <el-tag v-for="tag in resource.tags" :key="tag" class="tag-item" effect="plain">
                  {{ tag }}
                </el-tag>
              </div>
            </el-card>

            <!-- 授课教师 -->
            <el-card
              v-if="resource.teachers && resource.teachers.length > 0"
              class="teachers-card"
              shadow="never"
            >
              <template #header>
                <span>授课教师</span>
              </template>
              <div class="teachers-list">
                <div v-for="teacher in resource.teachers" :key="teacher.sn" class="teacher-item">
                  <el-icon><User /></el-icon>
                  <span class="teacher-name">{{ teacher.name }}</span>
                  <span v-if="teacher.department" class="teacher-dept"
                    >({{ teacher.department }})</span
                  >
                </div>
              </div>
            </el-card>

            <!-- 关联课程 -->
            <el-card
              v-if="resource.courses && resource.courses.length > 0"
              class="courses-card"
              shadow="never"
            >
              <template #header>
                <span>关联课程</span>
              </template>
              <div class="courses-list">
                <div v-for="course in resource.courses" :key="course.sn" class="course-item">
                  <el-icon><Reading /></el-icon>
                  <span class="course-name">{{ course.name }}</span>
                  <span v-if="course.semester" class="course-semester"
                    >({{ course.semester }})</span
                  >
                  <span v-if="course.credits" class="course-credits">{{ course.credits }}学分</span>
                </div>
              </div>
            </el-card>

            <!-- 关联资源 -->
            <el-card
              v-if="resource.relatedResources && resource.relatedResources.length > 0"
              class="related-resources-card"
              shadow="never"
            >
              <template #header>
                <span>关联资源</span>
              </template>
              <div class="related-resources-list">
                <div
                  v-for="relatedResource in resource.relatedResources"
                  :key="relatedResource.id"
                  class="related-resource-item"
                  @click="goToResourceDetail(relatedResource.id)"
                >
                  <div class="related-resource-icon">
                    <el-icon><Document /></el-icon>
                  </div>
                  <div class="related-resource-info">
                    <div class="related-resource-title">{{ relatedResource.title }}</div>
                    <div class="related-resource-meta">
                      <el-tag
                        size="small"
                        :type="getResourceTypeTagType(relatedResource.resourceType)"
                      >
                        {{
                          ResourceTypeLabels[relatedResource.resourceType as ResourceTypeType] ||
                          relatedResource.resourceType
                        }}
                      </el-tag>
                      <span class="related-resource-category">
                        {{
                          ResourceCategoryLabels[
                            relatedResource.category as ResourceCategoryType
                          ] || relatedResource.category
                        }}
                      </span>
                    </div>
                  </div>
                  <el-icon class="related-resource-arrow"><ArrowRight /></el-icon>
                </div>
              </div>
            </el-card>

            <!-- 资源信息 -->
            <el-card class="info-card" shadow="never">
              <template #header>
                <span>资源信息</span>
              </template>
              <div class="info-list">
                <div class="info-item">
                  <span class="info-label">资源ID</span>
                  <span
                    class="info-value uuid-container copyable"
                    :title="resource.id + ' (点击复制)'"
                    @click="copyResourceId(resource.id)"
                  >
                    <span class="uuid-text">{{ resource.id }}</span>
                    <el-icon class="copy-icon"><CopyDocument /></el-icon>
                  </span>
                </div>
                <div class="info-item">
                  <span class="info-label">文件大小</span>
                  <span class="info-value">{{ formatFileSize(resource.fileSize) }}</span>
                </div>
                <div class="info-item">
                  <span class="info-label">存储位置</span>
                  <el-tag :type="resource.storageType === 'oss' ? 'success' : 'info'" size="small">
                    {{ StorageTypeLabels[resource.storageType] || '本地存储' }}
                  </el-tag>
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
            <RatingWidget :resource-id="resourceId" @update="onRatingUpdate" />
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

      <!-- 修改关联信息弹窗 -->
      <EditResourceRelationsModal
        v-if="resource"
        v-model="showEditRelations"
        :resource-id="resourceId"
        :initial-teachers="resource.teachers?.map((t) => t.sn) || []"
        :initial-courses="resource.courses?.map((c) => c.sn) || []"
        :initial-related-resources="resource.relatedResources?.map((r) => r.id) || []"
        @success="handleRelationsUpdated"
      />

      <!-- 编辑资源描述弹窗 -->
      <el-dialog
        v-model="showEditDescription"
        title="编辑资源描述"
        width="600px"
        :close-on-click-modal="false"
      >
        <el-input
          v-model="editingDescription"
          type="textarea"
          :rows="8"
          placeholder="请输入资源描述，描述资源的内容、用途等信息（纯文本，不支持 Markdown）"
          maxlength="10000"
          show-word-limit
        />
        <template #footer>
          <span class="dialog-footer">
            <el-button @click="showEditDescription = false">取消</el-button>
            <el-button type="primary" :loading="savingDescription" @click="saveDescription">
              保存
            </el-button>
          </span>
        </template>
      </el-dialog>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
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
  Folder,
  Loading,
  Edit,
  User,
  CopyDocument,
  Document,
  ArrowRight,
  Link,
} from '@element-plus/icons-vue';
import {
  getResourceDetail,
  downloadResource,
  deleteResource,
  updateResourceDescription,
} from '@/api/resource';
import { checkResourceInFavorite } from '@/api/favorite';
import { useAuthStore } from '@/stores/auth';
import { useFavoriteStore } from '@/stores/favorite';
import { useDefaultFavorite } from '@/composables/useDefaultFavorite';
import PreviewSwitch from '@/components/preview/PreviewSwitch.vue';
import LikeButton from '@/components/interaction/LikeButton.vue';
import CommentSection from '@/components/interaction/CommentSection.vue';
import AddToFavoriteModal from '@/components/favorite/AddToFavoriteModal.vue';
import RatingWidget from '@/components/interaction/RatingWidget.vue';
import EditResourceRelationsModal from '@/components/resource/EditResourceRelationsModal.vue';
import {
  ResourceTypeLabels,
  ResourceCategoryLabels,
  StorageTypeLabels,
  formatFileSize,
  type ResourceDetail,
  type ResourceCategoryType,
  type ResourceTypeType,
} from '@/types/resource';
import type { ResourceRatingInfo } from '@/types/rating';
import { getErrorMessage, isHandledError } from '@/api/request';
import logger from '@/utils/logger';

const route = useRoute();
const router = useRouter();
const authStore = useAuthStore();
const favoriteStore = useFavoriteStore();
const { hasDefaultFavorite, defaultFavoriteId, defaultFavoriteName } = useDefaultFavorite();

// 状态
const loading = ref(true);
const downloading = ref(false);
const resource = ref<ResourceDetail | null>(null);
const showAddToFavorite = ref(false);
const showEditRelations = ref(false);
const addingToDefault = ref(false);
const isInDefaultFavorite = ref(false); // 资源是否已在默认收藏夹中
const checkingDefaultStatus = ref(false); // 正在检查默认收藏夹状态
const showEditDescription = ref(false);
const editingDescription = ref('');
const savingDescription = ref(false);

// 计算属性
const resourceId = computed(() => route.params.id as string);

// 默认收藏夹按钮类型
const defaultFavoriteButtonType = computed(() => {
  if (!hasDefaultFavorite.value) return 'info';
  if (isInDefaultFavorite.value) return 'primary';
  return ''; // 默认样式，和"收藏"按钮一样
});

// 默认收藏夹按钮文本
const defaultFavoriteButtonText = computed(() => {
  if (isInDefaultFavorite.value) return '已加入默认收藏夹';
  return '收藏至默认收藏夹';
});

// 默认收藏夹按钮 tooltip 内容
const tooltipContent = computed(() => {
  if (!hasDefaultFavorite.value) return '请在收藏夹页面设置默认收藏夹';
  if (isInDefaultFavorite.value) return `已加入默认收藏夹: ${defaultFavoriteName.value}`;
  return `收藏至默认收藏夹: ${defaultFavoriteName.value}`;
});

const canDelete = computed(() => {
  if (!resource.value || !authStore.user) return false;
  return resource.value.uploaderId === authStore.user.id || authStore.user.role === 'admin';
});

// 是否可以修改关联信息（上传者或管理员可以修改）
const canEditRelations = computed(() => {
  if (!resource.value || !authStore.user) return false;
  return resource.value.uploaderId === authStore.user.id || authStore.user.role === 'admin';
});

// 是否可以编辑（Markdown资源且是上传者或管理员）
const canEdit = computed(() => {
  if (!resource.value || !authStore.user) return false;
  if (resource.value.resourceType !== 'web_markdown') return false;
  return resource.value.uploaderId === authStore.user.id || authStore.user.role === 'admin';
});

// 是否可以编辑资源描述（上传者或管理员）
const canEditDescription = computed(() => {
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
    zip: 'info',
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
  // 重新检查资源是否在默认收藏夹中，同步"收藏至默认收藏夹"按钮状态
  checkDefaultFavoriteStatus();
};

// 检查资源是否在默认收藏夹中
const checkDefaultFavoriteStatus = async () => {
  if (!hasDefaultFavorite.value || !defaultFavoriteId.value) {
    isInDefaultFavorite.value = false;
    return;
  }

  checkingDefaultStatus.value = true;
  try {
    const result = await checkResourceInFavorite(resourceId.value);
    isInDefaultFavorite.value = result.inFavorites.includes(defaultFavoriteId.value);
  } catch {
    isInDefaultFavorite.value = false;
  } finally {
    checkingDefaultStatus.value = false;
  }
};

// 评分更新回调
const onRatingUpdate = (_info: ResourceRatingInfo) => {
  // 评分信息已在组件内部更新，这里可以添加额外的处理
};

// 跳转到指定资源详情页（在新标签页打开并聚焦）
const goToResourceDetail = (id: string) => {
  const url = `${window.location.origin}/resources/${id}`;
  const newWindow = window.open(url, '_blank');
  if (newWindow) {
    newWindow.focus();
  }
};

// 关联信息更新成功回调
const handleRelationsUpdated = () => {
  // 重新加载资源详情
  loadResourceDetail();
};

// 复制资源ID到剪贴板
const copyResourceId = async (id: string) => {
  try {
    await navigator.clipboard.writeText(id);
    ElMessage.success('资源ID已复制到剪贴板');
  } catch {
    // 降级方案：如果 clipboard API 不可用，使用传统方法
    const textArea = document.createElement('textarea');
    textArea.value = id;
    textArea.style.position = 'fixed';
    textArea.style.left = '-9999px';
    document.body.appendChild(textArea);
    textArea.select();
    try {
      document.execCommand('copy');
      ElMessage.success('资源ID已复制到剪贴板');
    } catch (error) {
      logger.error('[ResourceDetail]', '复制资源ID失败', error);
      if (!isHandledError(error)) {
        ElMessage.error('复制失败，请手动复制');
      }
    }
    document.body.removeChild(textArea);
  }
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
    // 加载完成后检查资源是否在默认收藏夹中
    if (authStore.isAuthenticated) {
      await checkDefaultFavoriteStatus();
    }
  } catch (error) {
    logger.error('[ResourceDetail]', '加载资源详情失败', error);
    if (!isHandledError(error)) {
      ElMessage.error(getErrorMessage(error, '加载资源详情失败'));
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
    await downloadResource(resourceId.value, undefined, {
      useCache: true,
      resourceDetail: {
        title: resource.value.title,
        resourceType: resource.value.resourceType,
      },
    });
    ElMessage.success('开始下载');
    // 更新下载次数
    resource.value.stats.downloads++;
  } catch (error) {
    logger.error('[ResourceDetail]', '下载资源失败', error);
    if (!isHandledError(error)) {
      ElMessage.error(getErrorMessage(error, '下载失败'));
    }
  } finally {
    downloading.value = false;
  }
};

// 删除资源
const handleDelete = async () => {
  if (!resource.value) return;

  try {
    await ElMessageBox.confirm('确定要删除这个资源吗？此操作不可恢复。', '删除确认', {
      confirmButtonText: '删除',
      cancelButtonText: '取消',
      type: 'warning',
    });

    await deleteResource(resourceId.value);
    ElMessage.success('删除成功');
    router.push('/resources');
  } catch (error) {
    if (error !== 'cancel') {
      logger.error('[ResourceDetail]', '删除资源失败', error);
      if (!isHandledError(error)) {
        ElMessage.error(getErrorMessage(error, '删除失败'));
      }
    }
  }
};

// 返回列表
const goBack = () => {
  router.push('/resources');
};

// 跳转到上传者主页
const goToUploaderHomepage = () => {
  if (resource.value?.uploaderId) {
    router.push(`/user/${resource.value.uploaderId}`);
  }
};

// 编辑资源
const handleEdit = () => {
  router.push(`/resources/${resourceId.value}/edit`);
};

// 添加到默认收藏夹
const addToDefaultFavorite = async () => {
  if (!hasDefaultFavorite.value || !defaultFavoriteId.value) {
    ElMessage.warning('请先设置默认收藏夹');
    return;
  }

  // 如果已经在默认收藏夹中，不需要重复添加
  if (isInDefaultFavorite.value) {
    return;
  }

  addingToDefault.value = true;
  try {
    const added = await favoriteStore.addResourceToFavorite(
      defaultFavoriteId.value,
      resourceId.value
    );

    if (added) {
      isInDefaultFavorite.value = true;
      ElMessage.success(`已添加到默认收藏夹: ${defaultFavoriteName.value}`);
    } else {
      // 资源已存在
      isInDefaultFavorite.value = true;
      ElMessage.warning('该资源已在默认收藏夹中');
    }
  } catch (error) {
    logger.error('[ResourceDetail]', '添加到默认收藏夹失败', error);
    // 只有非业务错误才显示错误弹窗
    if (!isHandledError(error)) {
      const errorMessage = getErrorMessage(error, '添加到默认收藏夹失败');
      ElMessage.error(errorMessage);
    }
  } finally {
    addingToDefault.value = false;
  }
};

// 监听编辑描述对话框的显示，打开时初始化编辑内容
watch(showEditDescription, (newValue) => {
  if (newValue && resource.value) {
    editingDescription.value = resource.value.description || '';
  }
});

// 保存资源描述
const saveDescription = async () => {
  if (!resource.value) return;

  savingDescription.value = true;
  try {
    const description = editingDescription.value.trim() || null;
    await updateResourceDescription(resourceId.value, description);
    ElMessage.success('资源描述更新成功');
    resource.value.description = description ?? undefined;
    showEditDescription.value = false;
  } catch (error) {
    logger.error('[ResourceDetail]', '更新资源描述失败', error);
    if (!isHandledError(error)) {
      const errorMessage = getErrorMessage(error, '更新失败');
      ElMessage.error(errorMessage);
    }
  } finally {
    savingDescription.value = false;
  }
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

.loading-icon {
  color: var(--el-color-primary);
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

.meta-item.uploader-link {
  cursor: pointer;
  transition: color 0.2s;
}

.meta-item.uploader-link:hover {
  color: var(--el-color-primary);
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
.comments-card,
.teachers-card,
.courses-card,
.related-resources-card {
  margin-bottom: 24px;
}

/* 关联资源列表样式 */
.related-resources-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.related-resource-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  border: 1px solid var(--el-border-color-lighter);
}

.related-resource-item:hover {
  background-color: var(--el-fill-color-light);
  border-color: var(--el-color-primary-light-5);
}

.related-resource-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border-radius: 8px;
  background-color: var(--el-color-primary-light-9);
  color: var(--el-color-primary);
  font-size: 20px;
  flex-shrink: 0;
}

.related-resource-info {
  flex: 1;
  min-width: 0;
}

.related-resource-title {
  font-weight: 500;
  color: var(--el-text-color-primary);
  margin-bottom: 4px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.related-resource-meta {
  display: flex;
  align-items: center;
  gap: 8px;
}

.related-resource-category {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.related-resource-arrow {
  color: var(--el-text-color-secondary);
  flex-shrink: 0;
}

.related-resource-item:hover .related-resource-arrow {
  color: var(--el-color-primary);
}

/* 授课教师列表样式 */
.teachers-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.teacher-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 0;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.teacher-item:last-child {
  border-bottom: none;
}

.teacher-name {
  font-weight: 500;
  color: var(--el-text-color-primary);
}

.teacher-dept {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

/* 关联课程列表样式 */
.courses-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.course-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 0;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.course-item:last-child {
  border-bottom: none;
}

.course-name {
  font-weight: 500;
  color: var(--el-text-color-primary);
  flex: 1;
}

.course-semester {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.course-credits {
  font-size: 12px;
  color: var(--el-color-primary);
  background: var(--el-color-primary-light-9);
  padding: 2px 8px;
  border-radius: 4px;
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

.description-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.description-empty {
  text-align: center;
  padding: 24px 0;
  color: var(--el-text-color-secondary);
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

.uuid-container {
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 6px;
  border-radius: 4px;
  transition: background-color 0.2s;
  font-size: 12px;
  font-family: monospace;
  max-width: 180px;
}

.uuid-container:hover {
  background-color: var(--el-color-primary-light-9);
  color: var(--el-color-primary);
}

.uuid-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  min-width: 0;
}

.copy-icon {
  font-size: 12px;
  opacity: 0.6;
}

.uuid-container:hover .copy-icon {
  opacity: 1;
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
