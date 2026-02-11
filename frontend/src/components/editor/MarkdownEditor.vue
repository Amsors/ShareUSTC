<template>
  <div class="markdown-editor-wrapper">
    <!-- 工具栏 -->
    <div class="editor-toolbar">
      <div class="toolbar-left">
        <el-tooltip content="插入图片">
          <el-button circle size="small" @click="showImageSelector = true">
            <el-icon><Picture /></el-icon>
          </el-button>
        </el-tooltip>
      </div>
      <div class="toolbar-right">
        <span class="word-count">{{ wordCount }} 字</span>
        <el-tag v-if="hasDraft" type="warning" size="small">有草稿</el-tag>
      </div>
    </div>

    <!-- 编辑器主体 -->
    <div class="editor-container">
      <md-editor
        v-model="content"
        :toolbars="(toolbars as any)"
        :footers="(footers as any)"
        @on-upload-img="handleUploadImg"
        @on-change="handleChange"
        placeholder="开始编写你的 Markdown 内容..."
        class="md-editor"
      />
    </div>

    <!-- 图片选择器弹窗 -->
    <ImageSelector
      v-model="showImageSelector"
      @select="insertImage"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue';
import { ElMessage } from 'element-plus';
import { Picture } from '@element-plus/icons-vue';
import { MdEditor } from 'md-editor-v3';
import 'md-editor-v3/lib/style.css';
import ImageSelector from './ImageSelector.vue';
import { uploadImage } from '../../api/imageHost';

// 定义props和emits
const props = defineProps<{
  modelValue: string;
  resourceId?: string;
  autoSaveKey?: string;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void;
  (e: 'change', value: string): void;
  (e: 'saveDraft'): void;
}>();

// 内容
const content = ref(props.modelValue || '');
const showImageSelector = ref(false);
const hasDraft = ref(false);

// 工具栏配置
// @ts-ignore
const toolbars = [
  'bold',
  'underline',
  'italic',
  'strikeThrough',
  '-',
  'title',
  'sub',
  'sup',
  'quote',
  'unorderedList',
  'orderedList',
  'task',
  '-',
  'codeRow',
  'code',
  'link',
  'table',
  'mermaid',
  'katex',
  '-',
  'revoke',
  'next',
  '=',
  'pageFullscreen',
  'preview',
  'htmlPreview',
  'catalog'
];

// 底部工具栏
// @ts-ignore
const footers = ['markdownTotal'];

// 字数统计
const wordCount = computed(() => {
  // 移除Markdown语法标记后计算字数
  const plainText = content.value
    .replace(/[#*_~`\[\](){}|]/g, '')
    .replace(/!\[.*?\]\(.*?\)/g, '[图片]')
    .replace(/\[.*?\]\(.*?\)/g, '$1')
    .replace(/```[\s\S]*?```/g, '[代码块]')
    .replace(/`.*?`/g, '$1');
  return plainText.length;
});

// 监听props变化
watch(() => props.modelValue, (newVal) => {
  if (newVal !== content.value) {
    content.value = newVal || '';
  }
}, { immediate: true });

// 监听内容变化
watch(content, (newVal) => {
  emit('update:modelValue', newVal);
  emit('change', newVal);
  checkDraft();
});

// 处理编辑器变化
const handleChange = (val: string) => {
  content.value = val;
};

// 处理图片上传（拖拽或粘贴）
const handleUploadImg = async (files: File[], callback: (urls: string[]) => void) => {
  try {
    const urls: string[] = [];
    for (const file of files) {
      const result = await uploadImage(file);
      urls.push(result.url);
    }
    callback(urls);
    ElMessage.success('图片上传成功');
  } catch (error: any) {
    ElMessage.error(error.message || '图片上传失败');
    callback([]);
  }
};

// 插入图片（从图床选择）
const insertImage = (imageUrl: string) => {
  const imageMarkdown = `\n![图片](${imageUrl})\n`;
  content.value += imageMarkdown;
  ElMessage.success('图片已插入');
};

// 检查是否有草稿
const checkDraft = () => {
  if (props.autoSaveKey) {
    const draft = localStorage.getItem(`markdown_draft_${props.autoSaveKey}`);
    hasDraft.value = !!draft && draft !== content.value;
  }
};

// 保存草稿
const saveDraft = () => {
  if (props.autoSaveKey && content.value) {
    localStorage.setItem(`markdown_draft_${props.autoSaveKey}`, content.value);
    hasDraft.value = false;
    emit('saveDraft');
  }
};

// 加载草稿
const loadDraft = (): string | null => {
  if (props.autoSaveKey) {
    return localStorage.getItem(`markdown_draft_${props.autoSaveKey}`);
  }
  return null;
};

// 清除草稿
const clearDraft = () => {
  if (props.autoSaveKey) {
    localStorage.removeItem(`markdown_draft_${props.autoSaveKey}`);
    hasDraft.value = false;
  }
};

// 自动保存定时器
let autoSaveTimer: number | null = null;

onMounted(() => {
  // 加载草稿（如果存在且与当前内容不同）
  const draft = loadDraft();
  if (draft && draft !== content.value && !content.value) {
    content.value = draft;
    hasDraft.value = true;
  }

  // 设置自动保存（每30秒）
  autoSaveTimer = window.setInterval(() => {
    if (content.value && props.autoSaveKey) {
      saveDraft();
    }
  }, 30000);
});

onBeforeUnmount(() => {
  if (autoSaveTimer) {
    clearInterval(autoSaveTimer);
  }
});

// 暴露方法
defineExpose({
  saveDraft,
  loadDraft,
  clearDraft,
  getContent: () => content.value
});
</script>

<style scoped>
.markdown-editor-wrapper {
  display: flex;
  flex-direction: column;
  height: 100%;
  border: 1px solid var(--el-border-color);
  border-radius: 8px;
  overflow: hidden;
  background: var(--el-bg-color);
}

.editor-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  border-bottom: 1px solid var(--el-border-color);
  background: var(--el-fill-color-lighter);
  flex-shrink: 0;
}

.toolbar-left {
  display: flex;
  gap: 8px;
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.word-count {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.editor-container {
  flex: 1;
  overflow: hidden;
  position: relative;
  min-height: 500px;
}

.md-editor {
  height: 100% !important;
}

/* 自定义md-editor样式 */
:deep(.md-editor) {
  --md-bk-color: var(--el-bg-color);
  --md-border-color: var(--el-border-color);
  --md-text-color: var(--el-text-color-primary);
  --md-hover-color: var(--el-fill-color-light);
}

:deep(.md-editor-toolbar) {
  border-bottom: 1px solid var(--el-border-color);
}

:deep(.md-editor-footer) {
  border-top: 1px solid var(--el-border-color);
}

:deep(.md-editor-content) {
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
}

/* 预览区域样式 */
:deep(.md-editor-preview) {
  padding: 16px;
}

:deep(.md-editor-preview h1) {
  font-size: 2em;
  border-bottom: 1px solid var(--el-border-color);
  padding-bottom: 0.3em;
  margin-bottom: 1em;
}

:deep(.md-editor-preview h2) {
  font-size: 1.5em;
  border-bottom: 1px solid var(--el-border-color);
  padding-bottom: 0.3em;
  margin: 1.5em 0 1em;
}

:deep(.md-editor-preview h3) {
  font-size: 1.25em;
  margin: 1.5em 0 1em;
}

:deep(.md-editor-preview p) {
  margin: 1em 0;
  line-height: 1.8;
}

:deep(.md-editor-preview img) {
  max-width: 100%;
  height: auto;
  border-radius: 4px;
}

:deep(.md-editor-preview code) {
  background-color: var(--el-fill-color);
  padding: 0.2em 0.4em;
  border-radius: 3px;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
}

:deep(.md-editor-preview pre) {
  background-color: var(--el-fill-color-dark);
  padding: 16px;
  border-radius: 8px;
  overflow-x: auto;
}

:deep(.md-editor-preview pre code) {
  background-color: transparent;
  padding: 0;
}

:deep(.md-editor-preview blockquote) {
  border-left: 4px solid var(--el-border-color);
  padding-left: 1em;
  margin: 1em 0;
  color: var(--el-text-color-secondary);
}

:deep(.md-editor-preview table) {
  width: 100%;
  border-collapse: collapse;
  margin: 1em 0;
}

:deep(.md-editor-preview th),
:deep(.md-editor-preview td) {
  border: 1px solid var(--el-border-color);
  padding: 8px 12px;
  text-align: left;
}

:deep(.md-editor-preview th) {
  background-color: var(--el-fill-color-light);
}
</style>
