<template>
  <el-dialog
    v-model="visible"
    title="下载到文件夹"
    width="480px"
    :close-on-click-modal="false"
    :close-on-press-escape="false"
    :show-close="!isProcessing"
    class="folder-download-progress-modal"
  >
    <div class="progress-content">
      <!-- 状态图标 -->
      <div class="status-icon">
        <el-icon v-if="progress.status === 'selecting'" :size="48" color="#909399">
          <FolderOpened />
        </el-icon>
        <el-icon v-else-if="progress.status === 'downloading'" class="icon-spin" :size="48" color="#409EFF">
          <Download />
        </el-icon>
        <el-icon v-else-if="progress.status === 'completed'" :size="48" color="#67C23A">
          <CircleCheck />
        </el-icon>
        <el-icon v-else-if="progress.status === 'error'" :size="48" color="#F56C6C">
          <CircleClose />
        </el-icon>
        <el-icon v-else-if="progress.status === 'cancelled'" :size="48" color="#909399">
          <CircleClose />
        </el-icon>
      </div>

      <!-- 状态文本 -->
      <div class="status-text">
        <template v-if="progress.status === 'selecting'">
          <p class="main-text">请选择保存文件夹</p>
          <p class="sub-text">系统将打开文件夹选择器</p>
        </template>
        <template v-else-if="progress.status === 'downloading'">
          <p class="main-text">正在下载文件...</p>
          <p class="sub-text" v-if="progress.currentFile">
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
          <p class="sub-text">
            {{ progress.currentIndex }} / {{ progress.totalFiles }} 个文件
          </p>
        </template>
        <template v-else-if="progress.status === 'completed'">
          <p class="main-text">下载完成!</p>
          <p class="sub-text">文件已保存到所选文件夹</p>
        </template>
        <template v-else-if="progress.status === 'cancelled'">
          <p class="main-text">已取消</p>
          <p class="sub-text">用户取消了文件夹选择</p>
        </template>
        <template v-else-if="progress.status === 'error'">
          <p class="main-text">下载失败</p>
          <p class="sub-text error-text">{{ progress.error || '请稍后重试' }}</p>
        </template>
      </div>

      <!-- 进度条 -->
      <div v-if="progress.status !== 'selecting' && progress.status !== 'cancelled'" class="progress-bar-section">
        <el-progress
          :percentage="progress.percent"
          :status="progressStatus"
          :stroke-width="12"
          :show-text="true"
        />
      </div>

      <!-- 统计信息 -->
      <div class="stats-info" v-if="progress.totalFiles > 0 && progress.status !== 'selecting'">
        <el-tag size="small" type="info">
          共 {{ progress.totalFiles }} 个文件
        </el-tag>
        <el-tag v-if="progress.savedCount > 0" size="small" type="success">
          <el-icon><Check /></el-icon>
          成功 {{ progress.savedCount }} 个
        </el-tag>
        <el-tag v-if="progress.failedCount > 0" size="small" type="danger">
          <el-icon><Close /></el-icon>
          失败 {{ progress.failedCount }} 个
        </el-tag>
      </div>

      <!-- 提示信息 -->
      <div class="tips-section">
        <el-alert
          v-if="progress.status === 'selecting'"
          title="请选择空的或专门用于保存下载文件的文件夹"
          type="info"
          :closable="false"
          show-icon
        />
        <el-alert
          v-if="progress.status === 'downloading'"
          title="下载中请勿关闭页面"
          type="warning"
          :closable="false"
          show-icon
        />
        <el-alert
          v-if="progress.status === 'completed' && progress.savedCount > 0"
          title="下载完成"
          type="success"
          :closable="false"
          show-icon
          :description="`已成功保存 ${progress.savedCount} 个文件到所选文件夹`"
        />
      </div>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <el-button
          v-if="isProcessing"
          type="danger"
          @click="handleCancel"
        >
          取消下载
        </el-button>
        <el-button
          v-else-if="progress.status === 'completed'"
          type="primary"
          @click="handleClose"
        >
          完成
        </el-button>
        <el-button
          v-else-if="progress.status === 'error' || progress.status === 'cancelled'"
          type="primary"
          @click="handleRetry"
        >
          重试
        </el-button>
        <el-button
          v-else
          @click="handleClose"
        >
          关闭
        </el-button>
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
  Close
} from '@element-plus/icons-vue';
import type { FolderDownloadProgress } from '../../utils/browserZip';

interface Props {
  modelValue: boolean;
  progress: FolderDownloadProgress;
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: false,
});

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
  'cancel': [];
  'retry': [];
  'close': [];
}>();

const visible = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value),
});

const isProcessing = computed(() => {
  return props.progress.status === 'downloading';
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
.folder-download-progress-modal {
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
    color: #303133;
    margin: 0 0 8px;
  }

  .sub-text {
    font-size: 14px;
    color: #606266;
    margin: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;

    &.error-text {
      color: #F56C6C;
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
