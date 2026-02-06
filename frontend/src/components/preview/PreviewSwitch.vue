<template>
  <div class="preview-switch">
    <ImageViewer
      v-if="previewType === 'image'"
      :resource-id="resourceId"
      :alt-text="resourceTitle"
    />
    <PdfViewer
      v-else-if="previewType === 'pdf'"
      :resource-id="resourceId"
    />
    <MarkdownViewer
      v-else-if="previewType === 'markdown'"
      :resource-id="resourceId"
    />
    <TxtViewer
      v-else-if="previewType === 'txt'"
      :resource-id="resourceId"
    />
    <div v-else class="unsupported-preview">
      <el-icon class="unsupported-icon"><Document /></el-icon>
      <p>该类型文件暂不支持预览</p>
      <el-button type="primary" @click="handleDownload">
        下载查看
      </el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { Document } from '@element-plus/icons-vue';
import ImageViewer from './ImageViewer.vue';
import PdfViewer from './PdfViewer.vue';
import MarkdownViewer from './MarkdownViewer.vue';
import TxtViewer from './TxtViewer.vue';
import { downloadResource } from '../../api/resource';

const props = defineProps<{
  resourceId: string;
  resourceType: string;
  resourceTitle?: string;
}>();

// 根据资源类型决定预览方式
const previewType = computed(() => {
  const type = props.resourceType.toLowerCase();

  // 图片类型
  if (['jpeg', 'jpg', 'png'].includes(type)) {
    return 'image';
  }

  // PDF类型
  if (type === 'pdf') {
    return 'pdf';
  }

  // Markdown类型
  if (['web_markdown', 'md', 'markdown'].includes(type)) {
    return 'markdown';
  }

  // 文本类型
  if (['txt', 'text'].includes(type)) {
    return 'txt';
  }

  return 'unsupported';
});

const handleDownload = async () => {
  try {
    await downloadResource(props.resourceId, props.resourceTitle || '资源');
  } catch (error: any) {
    // 错误已在API中处理
  }
};
</script>

<style scoped>
.preview-switch {
  width: 100%;
}

.unsupported-preview {
  min-height: 400px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background-color: var(--el-fill-color-light);
  border-radius: 8px;
}

.unsupported-icon {
  font-size: 64px;
  color: var(--el-text-color-secondary);
  margin-bottom: 16px;
}

.unsupported-preview p {
  color: var(--el-text-color-secondary);
  margin-bottom: 16px;
}
</style>
