<template>
  <div
    class="file-uploader"
    :class="{
      'is-dragover': isDragOver,
      'is-disabled': disabled || isCheckingHash,
    }"
    @dragenter.prevent="handleDragEnter"
    @dragover.prevent="handleDragOver"
    @dragleave.prevent="handleDragLeave"
    @drop.prevent="handleDrop"
    @click="handleClick"
  >
    <input
      ref="fileInput"
      type="file"
      class="file-input"
      :accept="accept"
      @change="handleFileChange"
    />

    <div v-if="!selectedFile" class="upload-placeholder">
      <el-icon class="upload-icon"><Upload /></el-icon>
      <div class="upload-text">
        <span class="primary-text">点击或拖拽文件到此处上传</span>
        <span class="secondary-text">
          支持 {{ acceptedExtensions.join(', ') }} 格式，最大 {{ maxSizeMB }}MB
        </span>
      </div>
    </div>

    <div v-else class="file-selected">
      <div class="file-info">
        <el-icon class="file-icon"><Document /></el-icon>
        <div class="file-details">
          <span class="file-name">{{ selectedFile.name }}</span>
          <span class="file-size">{{ formatFileSize(selectedFile.size) }}</span>
        </div>
        <el-icon class="delete-icon" @click.stop="clearFile"><Close /></el-icon>
      </div>

      <div v-if="isUploading" class="upload-progress">
        <el-progress :percentage="uploadProgress" :stroke-width="8" />
      </div>
    </div>

    <!-- 哈希检查进度 -->
    <div v-if="isCheckingHash" class="hash-checking-overlay">
      <el-icon class="checking-icon is-loading"><Loading /></el-icon>
      <span class="checking-text">正在检查文件...</span>
    </div>
  </div>

  <!-- 重复资源提示对话框 -->
  <el-dialog
    v-model="showDuplicateDialog"
    title="发现重复资源"
    width="600px"
    :close-on-click-modal="false"
    class="duplicate-dialog"
  >
    <div class="duplicate-content">
      <el-alert type="warning" :closable="false" show-icon>
        <template #title>
          <span>系统中已存在 {{ duplicateResources.length }} 个内容相同的资源</span>
        </template>
      </el-alert>

      <p class="duplicate-hint">这些资源可能与你要上传的内容重复，建议先查看已有资源：</p>

      <div class="duplicate-list">
        <div v-for="resource in duplicateResources" :key="resource.id" class="duplicate-item">
          <div class="resource-info">
            <el-icon class="resource-icon"><Document /></el-icon>
            <div class="resource-details">
              <a
                :href="`/resources/${resource.id}`"
                target="_blank"
                class="resource-title"
                @click.stop
              >
                {{ resource.title }}
                <el-icon class="link-icon"><Link /></el-icon>
              </a>
              <div class="resource-meta">
                <span class="meta-item">{{
                  ResourceTypeLabels[resource.resourceType as ResourceTypeType] ||
                  resource.resourceType
                }}</span>
                <span v-if="resource.courseName" class="meta-item">{{ resource.courseName }}</span>
                <span class="meta-item">上传者: {{ resource.uploaderName || '未知' }}</span>
                <span class="meta-item">{{ formatDate(resource.createdAt) }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleCancelUpload"> 取消上传 </el-button>
        <el-button type="primary" @click="handleContinueUpload"> 仍要上传 </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { Upload, Document, Close, Loading, Link } from '@element-plus/icons-vue';
import { ElMessage } from 'element-plus';
import {
  formatFileSize,
  SupportedExtensions,
  ResourceTypeLabels,
  type ResourceTypeType,
  type ResourceListItem,
} from '../../types/resource';
import { calculateFileHash } from '../../utils/fileHash';
import { getResourcesByFileHash } from '../../api/resource';
import logger from '../../utils/logger';

const props = defineProps<{
  modelValue?: File | null;
  accept?: string;
  maxSizeMB?: number;
  disabled?: boolean;
  isUploading?: boolean;
  uploadProgress?: number;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', file: File | null): void;
  (e: 'change', file: File | null): void;
}>();

const fileInput = ref<HTMLInputElement>();
const isDragOver = ref(false);
const isCheckingHash = ref(false);
const showDuplicateDialog = ref(false);
const duplicateResources = ref<ResourceListItem[]>([]);
const pendingFile = ref<File | null>(null);

// 支持的扩展名
const acceptedExtensions = SupportedExtensions;

// 默认 accept 值
const defaultAccept = acceptedExtensions.map((ext) => `.${ext}`).join(',');
const accept = computed(() => props.accept || defaultAccept);

// 最大文件大小 (MB)
const maxSizeMB = computed(() => props.maxSizeMB || 100);

// 选中的文件
const selectedFile = computed({
  get: () => props.modelValue || null,
  set: (value) => {
    emit('update:modelValue', value);
    emit('change', value);
  },
});

// 验证文件
const validateFile = (file: File): boolean => {
  // 检查文件大小
  const maxSize = maxSizeMB.value * 1024 * 1024;
  if (file.size > maxSize) {
    ElMessage.error(`文件大小超过限制，最大支持 ${maxSizeMB.value}MB`);
    return false;
  }

  // 检查文件类型
  const ext = file.name.split('.').pop()?.toLowerCase() || '';
  if (!acceptedExtensions.includes(ext)) {
    ElMessage.error(`不支持的文件类型: .${ext}。支持: ${acceptedExtensions.join(', ')}`);
    return false;
  }

  return true;
};

// 检查文件是否有重复
const checkDuplicateFile = async (file: File) => {
  isCheckingHash.value = true;
  pendingFile.value = file;

  try {
    logger.debug('[FileUploader]', '开始计算文件哈希: ' + file.name);

    // 计算文件哈希
    const hash = await calculateFileHash(file);
    logger.debug('[FileUploader]', '文件哈希计算完成: ' + file.name);

    // 查询是否有重复资源
    const resources = await getResourcesByFileHash(hash);

    if (resources.length > 0) {
      // 发现有重复资源
      logger.info('[FileUploader]', '发现重复资源: ' + file.name + ', 数量: ' + resources.length);
      duplicateResources.value = resources;
      showDuplicateDialog.value = true;
    } else {
      // 没有重复，直接设置文件
      logger.debug('[FileUploader]', '未发现重复资源: ' + file.name);
      selectedFile.value = file;
    }
  } catch (error: any) {
    logger.error('[FileUploader]', '检查文件哈希失败: ' + file.name + ', 错误: ' + error.message);
    // 如果哈希检查失败，仍然允许上传（降级处理）
    ElMessage.warning('文件检查失败，仍可进行上传');
    selectedFile.value = file;
  } finally {
    isCheckingHash.value = false;
  }
};

// 处理文件选择
const handleFileChange = (event: Event) => {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];

  if (file) {
    if (validateFile(file)) {
      checkDuplicateFile(file);
    }
  }

  // 重置 input 以便可以重复选择同一文件
  input.value = '';
};

// 处理点击
const handleClick = () => {
  if (props.disabled || isCheckingHash.value) return;
  fileInput.value?.click();
};

// 拖拽进入
const handleDragEnter = () => {
  if (props.disabled || isCheckingHash.value) return;
  isDragOver.value = true;
};

// 拖拽经过
const handleDragOver = () => {
  if (props.disabled || isCheckingHash.value) return;
  isDragOver.value = true;
};

// 拖拽离开
const handleDragLeave = () => {
  isDragOver.value = false;
};

// 处理拖放
const handleDrop = (event: DragEvent) => {
  if (props.disabled || isCheckingHash.value) return;
  isDragOver.value = false;

  const file = event.dataTransfer?.files[0];
  if (file) {
    if (validateFile(file)) {
      checkDuplicateFile(file);
    }
  }
};

// 清除文件
const clearFile = () => {
  selectedFile.value = null;
  pendingFile.value = null;
  duplicateResources.value = [];
};

// 取消上传
const handleCancelUpload = () => {
  showDuplicateDialog.value = false;
  pendingFile.value = null;
  duplicateResources.value = [];
  // 不设置 selectedFile，相当于取消选择
};

// 继续上传
const handleContinueUpload = () => {
  showDuplicateDialog.value = false;
  if (pendingFile.value) {
    selectedFile.value = pendingFile.value;
    pendingFile.value = null;
  }
};

// 格式化日期
const formatDate = (dateStr: string): string => {
  const date = new Date(dateStr);
  return date.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
};
</script>

<style scoped>
.file-uploader {
  border: 2px dashed var(--el-border-color);
  border-radius: 8px;
  padding: 40px 20px;
  text-align: center;
  cursor: pointer;
  transition: all 0.3s;
  background-color: var(--el-fill-color-lighter);
  position: relative;
}

.file-uploader:hover {
  border-color: var(--el-color-primary);
}

.file-uploader.is-dragover {
  border-color: var(--el-color-primary);
  background-color: var(--el-color-primary-light-9);
}

.file-uploader.is-disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.file-input {
  display: none;
}

.upload-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}

.upload-icon {
  font-size: 48px;
  color: var(--el-text-color-secondary);
}

.upload-text {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.primary-text {
  font-size: 16px;
  color: var(--el-text-color-primary);
}

.secondary-text {
  font-size: 14px;
  color: var(--el-text-color-secondary);
}

.file-selected {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.file-info {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background-color: var(--el-bg-color);
  border-radius: 4px;
}

.file-icon {
  font-size: 24px;
  color: var(--el-color-primary);
}

.file-details {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 4px;
}

.file-name {
  font-size: 14px;
  color: var(--el-text-color-primary);
  word-break: break-all;
}

.file-size {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.delete-icon {
  font-size: 16px;
  color: var(--el-text-color-secondary);
  cursor: pointer;
  transition: color 0.3s;
}

.delete-icon:hover {
  color: var(--el-color-danger);
}

.upload-progress {
  padding: 0 8px;
}

/* 哈希检查遮罩 */
.hash-checking-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(255, 255, 255, 0.9);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  border-radius: 8px;
}

.checking-icon {
  font-size: 32px;
  color: var(--el-color-primary);
}

.checking-icon.is-loading {
  animation: rotating 2s linear infinite;
}

.checking-text {
  font-size: 14px;
  color: var(--el-text-color-secondary);
}

@keyframes rotating {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

/* 重复资源对话框样式 */
.duplicate-content {
  max-height: 400px;
  overflow-y: auto;
}

.duplicate-hint {
  margin: 16px 0 12px;
  color: var(--el-text-color-regular);
  font-size: 14px;
}

.duplicate-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.duplicate-item {
  padding: 12px;
  background-color: var(--el-fill-color-lighter);
  border-radius: 6px;
  border: 1px solid var(--el-border-color-lighter);
}

.resource-info {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.resource-icon {
  font-size: 20px;
  color: var(--el-color-primary);
  margin-top: 2px;
}

.resource-details {
  flex: 1;
  min-width: 0;
}

.resource-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--el-color-primary);
  text-decoration: none;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  word-break: break-all;
}

.resource-title:hover {
  text-decoration: underline;
}

.link-icon {
  font-size: 12px;
  flex-shrink: 0;
}

.resource-meta {
  margin-top: 6px;
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.meta-item {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  background-color: var(--el-bg-color);
  padding: 2px 8px;
  border-radius: 4px;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}
</style>
