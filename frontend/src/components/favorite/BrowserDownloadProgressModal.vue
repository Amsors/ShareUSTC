<template>
  <el-dialog
    v-model="visible"
    title="浏览器打包下载"
    width="480px"
    :close-on-click-modal="false"
    :close-on-press-escape="false"
    :show-close="!isProcessing"
    class="download-progress-modal"
  >
    <div class="progress-content">
      <!-- 状态图标 -->
      <div class="status-icon">
        <el-icon
          v-if="progress.status === 'downloading'"
          class="icon-spin"
          :size="48"
          color="#409EFF"
        >
          <Download />
        </el-icon>
        <el-icon
          v-else-if="progress.status === 'packaging'"
          class="icon-spin"
          :size="48"
          color="#E6A23C"
        >
          <FolderOpened />
        </el-icon>
        <el-icon v-else-if="progress.status === 'completed'" :size="48" color="#67C23A">
          <CircleCheck />
        </el-icon>
        <el-icon v-else-if="progress.status === 'error'" :size="48" color="#F56C6C">
          <CircleClose />
        </el-icon>
      </div>

      <!-- 状态文本 -->
      <div class="status-text">
        <template v-if="progress.status === 'downloading'">
          <p class="main-text">正在下载文件...</p>
          <p v-if="progress.currentFile" class="sub-text">
            当前: {{ progress.currentFile }}
            <el-tag
              v-if="progress.currentFileSource === 'cache'"
              size="small"
              type="success"
              effect="dark"
              class="source-tag"
            >
              <el-icon><Check /></el-icon>
              缓存
            </el-tag>
            <el-tag
              v-else-if="progress.currentFileSource === 'download'"
              size="small"
              type="primary"
              effect="dark"
              class="source-tag"
            >
              <el-icon><Download /></el-icon>
              下载
            </el-tag>
            <el-tag
              v-else-if="progress.currentFileSource === 'error'"
              size="small"
              type="danger"
              effect="dark"
              class="source-tag"
            >
              <el-icon><Close /></el-icon>
              失败
            </el-tag>
          </p>
          <p class="sub-text">{{ progress.currentIndex }} / {{ progress.totalFiles }} 个文件</p>
        </template>
        <template v-else-if="progress.status === 'packaging'">
          <p class="main-text">正在打包...</p>
          <p class="sub-text">请稍候，正在生成 ZIP 文件</p>
        </template>
        <template v-else-if="progress.status === 'completed'">
          <p class="main-text">下载完成!</p>
          <p class="sub-text">ZIP 文件已开始下载</p>
        </template>
        <template v-else-if="progress.status === 'error'">
          <p class="main-text">下载失败</p>
          <p class="sub-text error-text">{{ progress.error || '请稍后重试' }}</p>
        </template>
      </div>

      <!-- 进度条 -->
      <div class="progress-bar-section">
        <el-progress
          :percentage="progress.percent"
          :status="progressStatus"
          :stroke-width="12"
          :show-text="true"
        />
      </div>

      <!-- 统计信息 -->
      <div v-if="progress.totalFiles > 0" class="stats-info">
        <el-tag size="small" type="info"> 共 {{ progress.totalFiles }} 个文件 </el-tag>
        <el-tag v-if="progress.cachedCount > 0" size="small" type="success">
          <el-icon><Check /></el-icon>
          缓存 {{ progress.cachedCount }} 个
        </el-tag>
        <el-tag v-if="progress.downloadedCount > 0" size="small" type="primary">
          <el-icon><Download /></el-icon>
          下载 {{ progress.downloadedCount }} 个
        </el-tag>
        <el-tag v-if="progress.failedCount > 0" size="small" type="danger">
          <el-icon><Close /></el-icon>
          失败 {{ progress.failedCount }} 个
        </el-tag>
      </div>

      <!-- 提示信息 -->
      <div class="tips-section">
        <el-alert
          v-if="showOssTip"
          title="混合存储模式"
          type="info"
          :closable="false"
          show-icon
          description="收藏夹包含云端和本地存储的资源，将自动选择最优下载方式"
        />
        <el-alert
          v-if="progress.status === 'downloading' && progress.currentIndex > 0"
          title="下载中请勿关闭页面"
          type="warning"
          :closable="false"
          show-icon
        />
        <el-alert
          v-if="progress.status === 'completed' && progress.cachedCount > 0"
          title="已使用本地缓存"
          type="success"
          :closable="false"
          show-icon
          :description="`本次下载使用了 ${progress.cachedCount} 个本地缓存文件，节省了流量和时间`"
        />
      </div>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <el-button v-if="isProcessing" type="danger" @click="handleCancel"> 取消下载 </el-button>
        <el-button v-else-if="progress.status === 'completed'" type="primary" @click="handleClose">
          完成
        </el-button>
        <el-button v-else-if="progress.status === 'error'" type="primary" @click="handleRetry">
          重试
        </el-button>
        <el-button v-else @click="handleClose"> 关闭 </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import {
  Download,
  FolderOpened,
  CircleCheck,
  CircleClose,
  Check,
  Close,
} from '@element-plus/icons-vue';
import type { DownloadProgress } from '../../utils/browserZip';

interface Props {
  modelValue: boolean;
  progress: DownloadProgress;
  showOssTip?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  showOssTip: false,
});

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
  cancel: [];
  retry: [];
  close: [];
}>();

const visible = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value),
});

const isProcessing = computed(() => {
  return props.progress.status === 'downloading' || props.progress.status === 'packaging';
});

const progressStatus = computed(() => {
  if (props.progress.status === 'error') return 'exception';
  if (props.progress.status === 'completed') return 'success';
  return '';
});

const handleCancel = () => {
  emit('cancel');
};

const handleRetry = () => {
  emit('retry');
};

const handleClose = () => {
  visible.value = false;
  emit('close');
};
</script>

<style scoped lang="scss">
.download-progress-modal {
  :deep(.el-dialog__body) {
    padding-top: 10px;
    padding-bottom: 10px;
  }
}

.progress-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}

.status-icon {
  .icon-spin {
    animation: spin 2s linear infinite;
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

.status-text {
  text-align: center;

  .main-text {
    font-size: 18px;
    font-weight: 600;
    color: var(--el-text-color-primary);
    margin: 0 0 8px;
  }

  .sub-text {
    font-size: 14px;
    color: var(--el-text-color-regular);
    margin: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;

    &.error-text {
      color: var(--el-color-danger);
    }

    .source-tag {
      margin-left: 4px;
    }
  }
}

.progress-bar-section {
  width: 100%;
  padding: 0 10px;
}

.stats-info {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  justify-content: center;

  :deep(.el-tag) {
    display: inline-flex;
    flex-direction: row;
    align-items: center;
    justify-content: center;
    gap: 4px;
    height: 24px;
    line-height: 1;
    vertical-align: middle;

    .el-icon {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      font-size: 12px;
      height: 12px;
      width: 12px;
      line-height: 1;
      vertical-align: middle;
    }

    span {
      display: inline-block;
      vertical-align: middle;
      line-height: 1;
    }
  }
}

.tips-section {
  width: 100%;

  :deep(.el-alert) {
    margin-bottom: 8px;

    &:last-child {
      margin-bottom: 0;
    }
  }
}

.dialog-footer {
  display: flex;
  justify-content: center;
  gap: 12px;
}
</style>
