<template>
  <div class="image-host-page">
    <!-- 导航栏 -->
    <nav class="navbar">
      <div class="nav-brand">
        <h1 @click="$router.push('/')" style="cursor: pointer;">ShareUSTC</h1>
      </div>
      <div class="nav-links">
        <router-link to="/">首页</router-link>
        <router-link to="/profile">个人中心</router-link>
        <el-dropdown @command="handleCommand">
          <span class="user-info">
            {{ authStore.user?.username }}
            <el-icon><ArrowDown /></el-icon>
          </span>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="settings">账号设置</el-dropdown-item>
              <el-dropdown-item command="logout" divided>退出登录</el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </div>
    </nav>

    <div class="image-host-container">
      <h1 class="page-title">
        <el-icon><Picture /></el-icon>
        图床
      </h1>
      <p class="page-subtitle">上传图片并获取 Markdown 引用链接</p>

      <!-- 上传区域 -->
      <el-card class="upload-card">
        <div
          class="upload-area"
          :class="{ 'is-dragover': isDragover }"
          @dragover.prevent="isDragover = true"
          @dragleave.prevent="isDragover = false"
          @drop.prevent="handleDrop"
          @click="triggerFileInput"
        >
          <input
            ref="fileInput"
            type="file"
            accept=".jpg,.jpeg,.png"
            style="display: none"
            @change="handleFileSelect"
          />
          <el-icon :size="64" color="#409eff"><Upload /></el-icon>
          <h3>点击或拖拽上传图片</h3>
          <p class="upload-hint">支持 JPG、JPEG、PNG 格式，单个文件最大 5MB</p>
        </div>

        <!-- 上传进度 -->
        <div v-if="uploading" class="upload-progress">
          <el-progress :percentage="uploadPercent" :stroke-width="16" status="active" />
        </div>
      </el-card>

      <!-- 上传结果 -->
      <el-card v-if="lastUploadedImage" class="result-card">
        <template #header>
          <div class="result-header">
            <span>上传成功</span>
            <el-tag type="success">{{ formatFileSize(lastUploadedImage.fileSize) }}</el-tag>
          </div>
        </template>

        <div class="result-content">
          <!-- 图片预览 -->
          <div class="image-preview">
            <img :src="lastUploadedImage.url" alt="预览" />
          </div>

          <!-- 链接信息 -->
          <div class="link-section">
            <div class="link-item">
              <span class="link-label">图片链接：</span>
              <el-input
                v-model="lastUploadedImage.url"
                readonly
                class="link-input"
              >
                <template #append>
                  <el-button @click="copyUrl(lastUploadedImage.url)">
                    <el-icon><CopyDocument /></el-icon>
                  </el-button>
                </template>
              </el-input>
            </div>

            <div class="link-item">
              <span class="link-label">Markdown：</span>
              <el-input
                v-model="lastUploadedImage.markdownLink"
                readonly
                class="link-input"
              >
                <template #append>
                  <el-button @click="copyMarkdown(lastUploadedImage.markdownLink)">
                    <el-icon><CopyDocument /></el-icon>
                  </el-button>
                </template>
              </el-input>
            </div>
          </div>
        </div>
      </el-card>

      <!-- 我的图片 -->
      <el-card class="gallery-card">
        <template #header>
          <div class="gallery-header">
            <span>我的图片</span>
            <el-button type="primary" size="small" @click="loadImages" :loading="loading">
              <el-icon><Refresh /></el-icon>
              刷新
            </el-button>
          </div>
        </template>

        <div v-if="loading" class="loading-state">
          <el-skeleton :rows="3" animated />
        </div>

        <div v-else-if="images.length === 0" class="empty-state">
          <el-empty description="暂无上传的图片">
            <el-button type="primary" @click="triggerFileInput">立即上传</el-button>
          </el-empty>
        </div>

        <div v-else class="image-gallery">
          <div
            v-for="image in images"
            :key="image.id"
            class="image-item"
            @click="showImageDetail(image)"
          >
            <div class="image-wrapper">
              <img :src="image.url" :alt="image.originalName || 'image'" />
              <div class="image-overlay">
                <el-button
                  type="primary"
                  circle
                  size="small"
                  @click.stop="copyMarkdown(image.markdownLink)"
                >
                  <el-icon><CopyDocument /></el-icon>
                </el-button>
                <el-button
                  type="danger"
                  circle
                  size="small"
                  @click.stop="confirmDelete(image)"
                >
                  <el-icon><Delete /></el-icon>
                </el-button>
              </div>
            </div>
            <div class="image-info">
              <p class="image-name" :title="image.originalName">{{ image.originalName || '未命名' }}</p>
              <p class="image-meta">
                {{ formatFileSize(image.fileSize) }} · {{ formatDate(image.createdAt) }}
              </p>
            </div>
          </div>
        </div>

        <!-- 分页 -->
        <div v-if="total > 0" class="pagination-wrapper">
          <el-pagination
            v-model:current-page="currentPage"
            v-model:page-size="pageSize"
            :total="total"
            layout="prev, pager, next, jumper"
            @change="handlePageChange"
          />
        </div>
      </el-card>
    </div>

    <!-- 图片详情对话框 -->
    <el-dialog
      v-model="detailDialogVisible"
      title="图片详情"
      width="600px"
      destroy-on-close
    >
      <div v-if="selectedImage" class="image-detail">
        <img :src="selectedImage.url" alt="详情" class="detail-image" />
        <div class="detail-info">
          <p><strong>文件名：</strong>{{ selectedImage.originalName || '未命名' }}</p>
          <p><strong>大小：</strong>{{ formatFileSize(selectedImage.fileSize) }}</p>
          <p><strong>上传时间：</strong>{{ formatDateTime(selectedImage.createdAt) }}</p>
        </div>
        <div class="detail-links">
          <el-input v-model="selectedImage.url" readonly>
            <template #prepend>URL</template>
            <template #append>
              <el-button @click="copyUrl(selectedImage.url)">复制</el-button>
            </template>
          </el-input>
          <el-input v-model="selectedImage.markdownLink" readonly class="mt-2">
            <template #prepend>Markdown</template>
            <template #append>
              <el-button @click="copyMarkdown(selectedImage.markdownLink)">复制</el-button>
            </template>
          </el-input>
        </div>
      </div>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '../stores/auth';
import {
  uploadImage,
  getMyImages,
  deleteImage,
  copyToClipboard,
  formatFileSize
} from '../api/imageHost';
import type { Image, ImageUploadResponse } from '../types/image';
import {
  ArrowDown,
  Picture,
  Upload,
  CopyDocument,
  Refresh,
  Delete
} from '@element-plus/icons-vue';
import { ElMessage, ElMessageBox } from 'element-plus';

const router = useRouter();
const authStore = useAuthStore();

// 上传相关
const fileInput = ref<HTMLInputElement | null>(null);
const isDragover = ref(false);
const uploading = ref(false);
const uploadPercent = ref(0);
const lastUploadedImage = ref<ImageUploadResponse | null>(null);

// 图片列表
const images = ref<Image[]>([]);
const loading = ref(false);
const total = ref(0);
const currentPage = ref(1);
const pageSize = ref(12);

// 详情对话框
const detailDialogVisible = ref(false);
const selectedImage = ref<Image | null>(null);

// 触发文件选择
const triggerFileInput = () => {
  fileInput.value?.click();
};

// 处理文件选择
const handleFileSelect = async (e: Event) => {
  const target = e.target as HTMLInputElement;
  const file = target.files?.[0];
  if (file) {
    await uploadFile(file);
  }
  // 清空input，允许重复选择同一文件
  target.value = '';
};

// 处理拖拽
const handleDrop = async (e: DragEvent) => {
  isDragover.value = false;
  const file = e.dataTransfer?.files[0];
  if (file) {
    // 检查文件类型
    if (!['image/jpeg', 'image/jpg', 'image/png'].includes(file.type)) {
      ElMessage.error('仅支持 JPG、JPEG、PNG 格式的图片');
      return;
    }
    await uploadFile(file);
  }
};

// 上传文件
const uploadFile = async (file: File) => {
  // 检查文件大小 (5MB)
  if (file.size > 5 * 1024 * 1024) {
    ElMessage.error('文件大小超过 5MB 限制');
    return;
  }

  uploading.value = true;
  uploadPercent.value = 0;

  try {
    const result = await uploadImage(file, (percent) => {
      uploadPercent.value = percent;
    });

    lastUploadedImage.value = result;
    ElMessage.success('上传成功');

    // 刷新图片列表
    await loadImages();
  } catch (error: any) {
    if (!error.isHandled) {
      ElMessage.error(error.message || '上传失败');
    }
  } finally {
    uploading.value = false;
  }
};

// 加载图片列表
const loadImages = async () => {
  loading.value = true;
  try {
    const result = await getMyImages({
      page: currentPage.value,
      perPage: pageSize.value
    });
    images.value = result.images;
    total.value = result.total;
  } catch (error: any) {
    if (!error.isHandled) {
      ElMessage.error(error.message || '加载图片列表失败');
    }
  } finally {
    loading.value = false;
  }
};

// 分页变化
const handlePageChange = () => {
  loadImages();
};

// 显示图片详情
const showImageDetail = (image: Image) => {
  selectedImage.value = image;
  detailDialogVisible.value = true;
};

// 确认删除
const confirmDelete = async (image: Image) => {
  try {
    await ElMessageBox.confirm(
      `确定要删除图片 "${image.originalName || '未命名'}" 吗？`,
      '确认删除',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      }
    );

    await deleteImage(image.id);
    ElMessage.success('删除成功');

    // 如果删除的是最后一张，且当前页不是第一页，则返回上一页
    if (images.value.length === 1 && currentPage.value > 1) {
      currentPage.value--;
    }

    await loadImages();
  } catch (error: any) {
    if (error !== 'cancel' && !error.isHandled) {
      ElMessage.error(error.message || '删除失败');
    }
  }
};

// 复制URL
const copyUrl = async (url: string) => {
  const success = await copyToClipboard(url);
  if (success) {
    ElMessage.success('链接已复制到剪贴板');
  } else {
    ElMessage.error('复制失败');
  }
};

// 复制Markdown
const copyMarkdown = async (markdown: string) => {
  const success = await copyToClipboard(markdown);
  if (success) {
    ElMessage.success('Markdown 已复制到剪贴板');
  } else {
    ElMessage.error('复制失败');
  }
};

// 格式化日期
const formatDate = (dateString?: string) => {
  if (!dateString) return '-';
  // 将无时区的时间字符串视为 UTC 时间
  const utcTimeString = dateString.endsWith('Z') ? dateString : `${dateString}Z`;
  return new Date(utcTimeString).toLocaleDateString('zh-CN');
};

// 格式化日期时间
const formatDateTime = (dateString?: string) => {
  if (!dateString) return '-';
  // 将无时区的时间字符串视为 UTC 时间
  const utcTimeString = dateString.endsWith('Z') ? dateString : `${dateString}Z`;
  return new Date(utcTimeString).toLocaleString('zh-CN');
};

// 下拉菜单处理
const handleCommand = async (command: string) => {
  if (command === 'logout') {
    try {
      await ElMessageBox.confirm('确定要退出登录吗？', '提示', {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      });
      await authStore.logoutUser();
      router.push('/');
    } catch (error) {
      // 用户取消
    }
  } else if (command === 'settings') {
    router.push('/profile');
  }
};

onMounted(() => {
  loadImages();
});
</script>

<style scoped>
.image-host-page {
  min-height: 100vh;
  background-color: #f5f7fa;
}

.navbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 40px;
  height: 60px;
  background-color: #fff;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  position: sticky;
  top: 0;
  z-index: 100;
}

.nav-brand h1 {
  margin: 0;
  color: #409eff;
  font-size: 24px;
}

.nav-links {
  display: flex;
  align-items: center;
  gap: 24px;
}

.nav-links a {
  text-decoration: none;
  color: #606266;
  font-size: 14px;
  transition: color 0.3s;
}

.nav-links a:hover {
  color: #409eff;
}

.nav-links a.router-link-active {
  color: #409eff;
  font-weight: 500;
}

.user-info {
  cursor: pointer;
  color: #606266;
  display: flex;
  align-items: center;
  gap: 4px;
}

.image-host-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 24px;
}

.page-title {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  margin: 0 0 8px;
  color: #303133;
  font-size: 28px;
}

.page-subtitle {
  text-align: center;
  color: #909399;
  margin: 0 0 24px;
}

.upload-card {
  margin-bottom: 24px;
}

.upload-area {
  border: 2px dashed #dcdfe6;
  border-radius: 8px;
  padding: 40px 20px;
  text-align: center;
  cursor: pointer;
  transition: all 0.3s;
}

.upload-area:hover,
.upload-area.is-dragover {
  border-color: #409eff;
  background-color: #f5f7fa;
}

.upload-area h3 {
  margin: 16px 0 8px;
  color: #303133;
}

.upload-hint {
  color: #909399;
  font-size: 14px;
  margin: 0;
}

.upload-progress {
  margin-top: 16px;
}

.result-card {
  margin-bottom: 24px;
}

.result-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.result-content {
  display: flex;
  gap: 24px;
  flex-wrap: wrap;
}

.image-preview {
  flex: 0 0 200px;
}

.image-preview img {
  width: 100%;
  max-height: 200px;
  object-fit: contain;
  border-radius: 4px;
  border: 1px solid #ebeef5;
}

.link-section {
  flex: 1;
  min-width: 300px;
}

.link-item {
  margin-bottom: 16px;
}

.link-item:last-child {
  margin-bottom: 0;
}

.link-label {
  display: block;
  margin-bottom: 8px;
  color: #606266;
  font-size: 14px;
}

.gallery-card {
  margin-bottom: 24px;
}

.gallery-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.loading-state,
.empty-state {
  padding: 40px 0;
}

.image-gallery {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 16px;
}

.image-item {
  cursor: pointer;
  border-radius: 8px;
  overflow: hidden;
  background-color: #fff;
  border: 1px solid #ebeef5;
  transition: box-shadow 0.3s;
}

.image-item:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.image-wrapper {
  position: relative;
  width: 100%;
  height: 150px;
  background-color: #f5f7fa;
  overflow: hidden;
}

.image-wrapper img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.image-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 12px;
  opacity: 0;
  transition: opacity 0.3s;
}

.image-wrapper:hover .image-overlay {
  opacity: 1;
}

.image-info {
  padding: 12px;
}

.image-name {
  margin: 0 0 4px;
  font-size: 14px;
  color: #303133;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.image-meta {
  margin: 0;
  font-size: 12px;
  color: #909399;
}

.pagination-wrapper {
  margin-top: 24px;
  display: flex;
  justify-content: center;
}

.image-detail {
  text-align: center;
}

.detail-image {
  max-width: 100%;
  max-height: 300px;
  object-fit: contain;
  margin-bottom: 16px;
  border-radius: 4px;
}

.detail-info {
  text-align: left;
  background-color: #f5f7fa;
  padding: 16px;
  border-radius: 4px;
  margin-bottom: 16px;
}

.detail-info p {
  margin: 8px 0;
  color: #606266;
}

.detail-links {
  text-align: left;
}

.mt-2 {
  margin-top: 12px;
}
</style>
