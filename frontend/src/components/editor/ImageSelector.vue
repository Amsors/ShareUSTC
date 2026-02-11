<template>
  <el-dialog
    v-model="dialogVisible"
    title="插入图片"
    width="800px"
    destroy-on-close
    class="image-selector-dialog"
  >
    <div class="image-selector-content">
      <!-- 上传新图片 -->
      <el-tabs v-model="activeTab">
        <el-tab-pane label="我的图床" name="gallery">
          <div v-if="loading" class="loading-state">
            <el-icon class="loading-icon" :size="32"><Loading /></el-icon>
            <span>加载中...</span>
          </div>

          <div v-else-if="images.length === 0" class="empty-state">
            <el-empty description="暂无图片">
              <el-button type="primary" @click="activeTab = 'upload'">上传图片</el-button>
            </el-empty>
          </div>

          <div v-else class="image-gallery">
            <div
              v-for="image in images"
              :key="image.id"
              class="image-item"
              :class="{ 'is-selected': selectedImage?.id === image.id }"
              @click="selectImage(image)"
            >
              <div class="image-wrapper">
                <img :src="image.url" :alt="image.originalName || 'image'" />
                <div v-if="selectedImage?.id === image.id" class="selected-overlay">
                  <el-icon :size="24"><Check /></el-icon>
                </div>
              </div>
              <div class="image-info">
                <p class="image-name" :title="image.originalName">
                  {{ truncateFileName(image.originalName || '未命名', 15) }}
                </p>
                <p class="image-size">{{ formatFileSize(image.fileSize) }}</p>
              </div>
            </div>
          </div>

          <!-- 分页 -->
          <div v-if="total > pageSize" class="pagination-wrapper">
            <el-pagination
              v-model:current-page="currentPage"
              v-model:page-size="pageSize"
              :total="total"
              :page-sizes="[12, 24, 36]"
              layout="prev, pager, next"
              @change="handlePageChange"
            />
          </div>
        </el-tab-pane>

        <el-tab-pane label="上传新图片" name="upload">
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
            <el-icon :size="48" color="#409eff"><Upload /></el-icon>
            <h3>点击或拖拽上传图片</h3>
            <p class="upload-hint">支持 JPG、JPEG、PNG 格式，单个文件最大 5MB</p>
          </div>

          <!-- 上传进度 -->
          <div v-if="uploading" class="upload-progress">
            <el-progress :percentage="uploadPercent" :stroke-width="16" status="active" />
          </div>

          <!-- 刚上传的图片预览 -->
          <div v-if="lastUploadedImage" class="last-uploaded">
            <h4>刚上传的图片</h4>
            <div class="image-preview-card" @click="selectImage(lastUploadedImage)">
              <img :src="lastUploadedImage.url" alt="预览" />
              <p>{{ lastUploadedImage.originalName }}</p>
            </div>
          </div>
        </el-tab-pane>

        <el-tab-pane label="外部链接" name="url">
          <div class="url-input-section">
            <el-input
              v-model="externalUrl"
              placeholder="请输入图片URL地址"
              size="large"
              clearable
            >
              <template #prepend>URL</template>
            </el-input>
            <div v-if="externalUrl" class="url-preview">
              <img :src="externalUrl" alt="预览" @error="handleImageError" @load="externalUrlValid = true" />
            </div>
          </div>
        </el-tab-pane>
      </el-tabs>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" :disabled="!canConfirm" @click="confirmSelection">
          插入图片
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import { ElMessage } from 'element-plus';
import { Loading, Check, Upload } from '@element-plus/icons-vue';
import { getMyImages, uploadImage } from '../../api/imageHost';
import type { Image } from '../../types/image';

const props = defineProps<{
  modelValue: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'select', url: string): void;
}>();

// 对话框可见性
const dialogVisible = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val)
});

// 标签页
const activeTab = ref('gallery');

// 图床相关
const images = ref<Image[]>([]);
const loading = ref(false);
const total = ref(0);
const currentPage = ref(1);
const pageSize = ref(12);
const selectedImage = ref<Image | null>(null);

// 上传相关
const fileInput = ref<HTMLInputElement | null>(null);
const isDragover = ref(false);
const uploading = ref(false);
const uploadPercent = ref(0);
const lastUploadedImage = ref<Image | null>(null);

// 外部链接
const externalUrl = ref('');
const externalUrlValid = ref(false);

// 是否可以确认
const canConfirm = computed(() => {
  if (activeTab.value === 'gallery' || activeTab.value === 'upload') {
    return !!selectedImage.value;
  }
  if (activeTab.value === 'url') {
    return !!externalUrl.value && externalUrlValid.value;
  }
  return false;
});

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
  // 清除选择
  selectedImage.value = null;
};

// 选择图片
const selectImage = (image: Image) => {
  selectedImage.value = image;
};

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
  target.value = '';
};

// 处理拖拽
const handleDrop = async (e: DragEvent) => {
  isDragover.value = false;
  const file = e.dataTransfer?.files[0];
  if (file) {
    if (!['image/jpeg', 'image/jpg', 'image/png'].includes(file.type)) {
      ElMessage.error('仅支持 JPG、JPEG、PNG 格式的图片');
      return;
    }
    await uploadFile(file);
  }
};

// 上传文件
const uploadFile = async (file: File) => {
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

    // 转换为Image类型
    const image: Image = {
      id: result.id,
      url: result.url,
      markdownLink: result.markdownLink,
      originalName: result.originalName,
      fileSize: result.fileSize,
      createdAt: result.createdAt
    };

    lastUploadedImage.value = image;
    selectedImage.value = image;
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

// 处理图片加载错误
const handleImageError = () => {
  externalUrlValid.value = false;
  ElMessage.error('图片加载失败，请检查URL是否正确');
};

// 确认选择
const confirmSelection = () => {
  let url = '';
  if (activeTab.value === 'gallery' || activeTab.value === 'upload') {
    url = selectedImage.value?.url || '';
  } else if (activeTab.value === 'url') {
    url = externalUrl.value;
  }

  if (url) {
    emit('select', url);
    dialogVisible.value = false;
    // 重置状态
    selectedImage.value = null;
    externalUrl.value = '';
    externalUrlValid.value = false;
  }
};

// 格式化文件大小
const formatFileSize = (bytes?: number): string => {
  if (!bytes) return '-';
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
};

// 截断文件名
const truncateFileName = (name: string, maxLength: number): string => {
  if (name.length <= maxLength) return name;
  const ext = name.split('.').pop();
  const base = name.substring(0, maxLength - 4);
  return `${base}...${ext ? '.' + ext : ''}`;
};

// 监听对话框打开
watch(dialogVisible, (val) => {
  if (val) {
    loadImages();
    // 重置状态
    selectedImage.value = null;
    lastUploadedImage.value = null;
    externalUrl.value = '';
    externalUrlValid.value = false;
    activeTab.value = 'gallery';
  }
});

onMounted(() => {
  if (dialogVisible.value) {
    loadImages();
  }
});
</script>

<style scoped>
.image-selector-content {
  min-height: 400px;
}

.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 0;
  gap: 12px;
  color: var(--el-text-color-secondary);
}

.loading-icon {
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

.empty-state {
  padding: 40px 0;
}

.image-gallery {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: 12px;
  padding: 16px 0;
}

.image-item {
  cursor: pointer;
  border-radius: 8px;
  overflow: hidden;
  background: var(--el-fill-color-lighter);
  border: 2px solid transparent;
  transition: all 0.3s;
}

.image-item:hover {
  border-color: var(--el-color-primary-light-5);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.image-item.is-selected {
  border-color: var(--el-color-primary);
}

.image-wrapper {
  position: relative;
  width: 100%;
  height: 120px;
  background: var(--el-fill-color);
  overflow: hidden;
}

.image-wrapper img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.selected-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(64, 158, 255, 0.7);
  display: flex;
  justify-content: center;
  align-items: center;
  color: white;
}

.image-info {
  padding: 8px;
}

.image-name {
  margin: 0 0 4px;
  font-size: 12px;
  color: var(--el-text-color-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.image-size {
  margin: 0;
  font-size: 11px;
  color: var(--el-text-color-secondary);
}

.pagination-wrapper {
  margin-top: 16px;
  display: flex;
  justify-content: center;
}

/* 上传区域 */
.upload-area {
  border: 2px dashed var(--el-border-color);
  border-radius: 8px;
  padding: 40px 20px;
  text-align: center;
  cursor: pointer;
  transition: all 0.3s;
  margin: 16px 0;
}

.upload-area:hover,
.upload-area.is-dragover {
  border-color: var(--el-color-primary);
  background-color: var(--el-fill-color-lighter);
}

.upload-area h3 {
  margin: 16px 0 8px;
  color: var(--el-text-color-primary);
  font-weight: 500;
}

.upload-hint {
  color: var(--el-text-color-secondary);
  font-size: 14px;
  margin: 0;
}

.upload-progress {
  margin: 16px 0;
}

/* 刚上传的图片 */
.last-uploaded {
  margin-top: 24px;
  padding-top: 24px;
  border-top: 1px solid var(--el-border-color);
}

.last-uploaded h4 {
  margin: 0 0 12px;
  color: var(--el-text-color-primary);
}

.image-preview-card {
  cursor: pointer;
  border-radius: 8px;
  overflow: hidden;
  border: 2px solid var(--el-border-color);
  transition: all 0.3s;
  max-width: 200px;
}

.image-preview-card:hover {
  border-color: var(--el-color-primary);
}

.image-preview-card img {
  width: 100%;
  height: 120px;
  object-fit: cover;
}

.image-preview-card p {
  margin: 0;
  padding: 8px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* URL输入 */
.url-input-section {
  padding: 24px 0;
}

.url-preview {
  margin-top: 16px;
  border-radius: 8px;
  overflow: hidden;
  border: 1px solid var(--el-border-color);
  max-height: 300px;
  display: flex;
  justify-content: center;
  align-items: center;
  background: var(--el-fill-color-lighter);
}

.url-preview img {
  max-width: 100%;
  max-height: 300px;
  object-fit: contain;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}
</style>
