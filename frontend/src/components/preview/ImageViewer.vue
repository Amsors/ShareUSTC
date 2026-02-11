<template>
  <div class="image-viewer">
    <div v-if="loading" class="loading-container">
      <span class="loading-text">加载中...</span>
    </div>
    <div v-else-if="error" class="error-container">
      <el-empty description="图片加载失败" />
      <el-button type="primary" @click="loadImage">重试</el-button>
    </div>
    <div v-else class="image-container">
      <img
        :src="imageUrl"
        :alt="altText"
        class="preview-image"
        @load="handleLoad"
        @error="handleError"
        @click="toggleFullscreen"
      />
      <div class="image-overlay">
        <el-button
          circle
          size="small"
          @click.stop="zoomIn"
          :disabled="scale >= 3"
        >
          <el-icon><ZoomIn /></el-icon>
        </el-button>
        <el-button
          circle
          size="small"
          @click.stop="zoomOut"
          :disabled="scale <= 0.5"
        >
          <el-icon><ZoomOut /></el-icon>
        </el-button>
        <el-button circle size="small" @click.stop="toggleFullscreen">
          <el-icon><FullScreen /></el-icon>
        </el-button>
      </div>
    </div>

    <!-- 全屏预览 -->
    <el-dialog
      v-model="fullscreen"
      :title="altText"
      width="90%"
      destroy-on-close
      class="fullscreen-preview"
    >
      <img :src="imageUrl" class="fullscreen-image" />
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { ZoomIn, ZoomOut, FullScreen } from '@element-plus/icons-vue';
import { getResourceContent } from '../../api/resource';

const props = defineProps<{
  resourceId: string;
  altText?: string;
}>();

const loading = ref(true);
const error = ref(false);
const imageUrl = ref('');
const scale = ref(1);
const fullscreen = ref(false);

const loadImage = async () => {
  loading.value = true;
  error.value = false;
  try {
    console.log('[ImageViewer] 开始加载图片, resourceId:', props.resourceId);
    const blob = await getResourceContent(props.resourceId);
    console.log('[ImageViewer] 获取到blob:', blob.type, blob.size);

    // 确保blob类型正确
    let imageBlob = blob;
    if (!blob.type || blob.type === 'application/octet-stream') {
      // 尝试从文件扩展名推断类型
      const ext = props.altText?.split('.').pop()?.toLowerCase();
      if (ext === 'png') {
        imageBlob = new Blob([blob], { type: 'image/png' });
      } else if (ext === 'jpg' || ext === 'jpeg') {
        imageBlob = new Blob([blob], { type: 'image/jpeg' });
      }
      console.log('[ImageViewer] 修正后的blob类型:', imageBlob.type);
    }

    const url = URL.createObjectURL(imageBlob);
    console.log('[ImageViewer] 创建的URL:', url);
    imageUrl.value = url;
    // 对于 blob URL，图片加载事件可能不可靠，这里直接设置 loading 为 false
    // 实际图片加载状态由 img 的 @load 和 @error 处理
    loading.value = false;
  } catch (err: any) {
    console.error('[ImageViewer] 加载失败:', err);
    error.value = true;
    loading.value = false;
  }
};

const handleLoad = () => {
  loading.value = false;
};

const handleError = () => {
  error.value = true;
  loading.value = false;
};

const zoomIn = () => {
  if (scale.value < 3) {
    scale.value += 0.25;
  }
};

const zoomOut = () => {
  if (scale.value > 0.5) {
    scale.value -= 0.25;
  }
};

const toggleFullscreen = () => {
  fullscreen.value = !fullscreen.value;
};

// 监听resourceId变化重新加载
watch(() => props.resourceId, () => {
  if (imageUrl.value) {
    URL.revokeObjectURL(imageUrl.value);
  }
  loadImage();
}, { immediate: true });
</script>

<style scoped>
.image-viewer {
  width: 100%;
  min-height: 300px;
}

.loading-container,
.error-container {
  padding: 40px 0;
  text-align: center;
}

.loading-text {
  color: #909399;
  font-size: 14px;
}

.image-container {
  position: relative;
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 300px;
  background-color: var(--el-fill-color-light);
  border-radius: 8px;
  overflow: hidden;
}

.preview-image {
  max-width: 100%;
  max-height: 600px;
  object-fit: contain;
  cursor: zoom-in;
  transition: transform 0.2s ease;
}

.image-overlay {
  position: absolute;
  bottom: 16px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  gap: 8px;
  padding: 8px 16px;
  background-color: rgba(0, 0, 0, 0.6);
  border-radius: 24px;
  opacity: 0;
  transition: opacity 0.3s ease;
}

.image-container:hover .image-overlay {
  opacity: 1;
}

.image-overlay :deep(.el-button) {
  color: #fff;
  background-color: transparent;
  border: none;
}

.image-overlay :deep(.el-button:hover) {
  background-color: rgba(255, 255, 255, 0.2);
}

.fullscreen-image {
  width: 100%;
  max-height: 80vh;
  object-fit: contain;
}
</style>
