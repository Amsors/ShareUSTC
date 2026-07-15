<template>
  <div class="markdown-editor-wrapper" :class="{ 'is-auto-grow': autoGrow }">
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
    <div ref="editorContainerRef" class="editor-container">
      <md-editor
        v-model="content"
        :toolbars="toolbars"
        :footers="footers"
        catalog-layout="flat"
        placeholder="开始编写你的 Markdown 内容..."
        class="md-editor"
        @on-upload-img="handleUploadImg"
        @on-change="handleChange"
      />
    </div>

    <!-- 图片选择器弹窗 -->
    <ImageSelector v-model="showImageSelector" @select="insertImage" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue';
import { ElMessage } from 'element-plus';
import { Picture } from '@element-plus/icons-vue';
import { config, MdEditor } from 'md-editor-v3';
// 工具栏/底栏类型未从包入口导出，从内部类型路径引入（类型导入编译期擦除，不影响运行时）
import type { ToolbarNames, Footers } from 'md-editor-v3/lib/types/MdEditor/type';
import 'md-editor-v3/lib/style.css';
import ImageSelector from '@/components/editor/ImageSelector.vue';
import { useResizableMarkdownCatalog } from '@/composables/useResizableMarkdownCatalog';
import { uploadImage } from '@/api/imageHost';
import { getErrorMessage, isHandledError } from '@/api/request';
import logger from '@/utils/logger';
import {
  withMarkdownSourceAutoGrow,
  withMarkdownSourceLineNumbers,
} from '@/utils/markdownEditorConfig';

// 为 CodeMirror 源文件编辑区启用行号及按内容测高。
config({
  codeMirrorExtensions: (extensions) =>
    withMarkdownSourceAutoGrow(withMarkdownSourceLineNumbers(extensions)),
});

// 定义props和emits
const props = defineProps<{
  modelValue: string;
  resourceId?: string;
  autoSaveKey?: string;
  autoGrow?: boolean;
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
const editorContainerRef = ref<HTMLElement>();

useResizableMarkdownCatalog(editorContainerRef);

// 工具栏配置
const toolbars: ToolbarNames[] = [
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
  'catalog',
];

// 底部工具栏
const footers: Footers[] = ['markdownTotal'];

// 字数统计
const wordCount = computed(() => {
  // 移除Markdown语法标记后计算字数
  const plainText = content.value
    .replace(/[#*_~`[\](){}|]/g, '')
    .replace(/!\[.*?\]\(.*?\)/g, '[图片]')
    .replace(/\[.*?\]\(.*?\)/g, '$1')
    .replace(/```[\s\S]*?```/g, '[代码块]')
    .replace(/`.*?`/g, '$1');
  return plainText.length;
});

// 监听props变化
watch(
  () => props.modelValue,
  (newVal) => {
    if (newVal !== content.value) {
      content.value = newVal || '';
    }
  },
  { immediate: true }
);

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
  } catch (error) {
    logger.error('[MarkdownEditor]', '图片上传失败', error);
    if (!isHandledError(error)) {
      ElMessage.error(getErrorMessage(error, '图片上传失败'));
    }
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
  getContent: () => content.value,
});
</script>

<style scoped src="./markdownEditor.css"></style>
