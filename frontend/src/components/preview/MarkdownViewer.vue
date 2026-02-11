<template>
  <div class="markdown-viewer">
    <div v-if="loading" class="loading-container">
      <span class="loading-text">加载中...</span>
    </div>
    <div v-else-if="error" class="error-container">
      <el-empty description="内容加载失败" />
      <el-button type="primary" @click="loadContent">重试</el-button>
    </div>
    <div v-else class="markdown-container">
      <div class="markdown-body" v-html="renderedContent"></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import MarkdownIt from 'markdown-it';
import { getResourceContent } from '../../api/resource';

const props = defineProps<{
  resourceId: string;
}>();

const loading = ref(true);
const error = ref(false);
const content = ref('');

// 初始化 MarkdownIt（使用安全模式）
const md = new MarkdownIt({
  html: false,        // 禁用 HTML 标签，防止 XSS
  breaks: true,       // 转换换行符为 <br>
  linkify: true,      // 自动转换 URL 为链接
  typographer: true,  // 启用排版美化
});

const renderedContent = computed(() => {
  return md.render(content.value);
});

const loadContent = async () => {
  loading.value = true;
  error.value = false;
  try {
    const blob = await getResourceContent(props.resourceId);
    const text = await blob.text();
    // 限制显示长度
    const maxLength = 100000;
    if (text.length > maxLength) {
      content.value = text.substring(0, maxLength) + '\n\n... (文件内容过长，已截断显示)';
    } else {
      content.value = text;
    }
    loading.value = false;
  } catch (err) {
    error.value = true;
    loading.value = false;
  }
};

watch(() => props.resourceId, () => {
  loadContent();
}, { immediate: true });
</script>

<style scoped>
.markdown-viewer {
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

.markdown-container {
  background-color: var(--el-bg-color);
  border-radius: 8px;
  padding: 24px;
  max-height: 600px;
  overflow: auto;
}

.markdown-body {
  font-size: 16px;
  line-height: 1.8;
  color: var(--el-text-color-primary);
}

/* Markdown 样式 */
.markdown-body :deep(h1) {
  font-size: 2em;
  border-bottom: 1px solid var(--el-border-color);
  padding-bottom: 0.3em;
  margin-bottom: 1em;
}

.markdown-body :deep(h2) {
  font-size: 1.5em;
  border-bottom: 1px solid var(--el-border-color);
  padding-bottom: 0.3em;
  margin: 1.5em 0 1em;
}

.markdown-body :deep(h3) {
  font-size: 1.25em;
  margin: 1.5em 0 1em;
}

.markdown-body :deep(p) {
  margin: 1em 0;
}

.markdown-body :deep(ul),
.markdown-body :deep(ol) {
  padding-left: 2em;
  margin: 1em 0;
}

.markdown-body :deep(li) {
  margin: 0.5em 0;
}

.markdown-body :deep(code) {
  background-color: var(--el-fill-color);
  padding: 0.2em 0.4em;
  border-radius: 3px;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 0.9em;
}

.markdown-body :deep(pre) {
  background-color: var(--el-fill-color-dark);
  padding: 16px;
  border-radius: 8px;
  overflow-x: auto;
  margin: 1em 0;
}

.markdown-body :deep(pre code) {
  background-color: transparent;
  padding: 0;
}

.markdown-body :deep(blockquote) {
  border-left: 4px solid var(--el-border-color);
  padding-left: 1em;
  margin: 1em 0;
  color: var(--el-text-color-secondary);
}

.markdown-body :deep(a) {
  color: var(--el-color-primary);
  text-decoration: none;
}

.markdown-body :deep(a:hover) {
  text-decoration: underline;
}

.markdown-body :deep(table) {
  width: 100%;
  border-collapse: collapse;
  margin: 1em 0;
}

.markdown-body :deep(th),
.markdown-body :deep(td) {
  border: 1px solid var(--el-border-color);
  padding: 8px 12px;
  text-align: left;
}

.markdown-body :deep(th) {
  background-color: var(--el-fill-color-light);
}

.markdown-body :deep(img) {
  max-width: 100%;
  height: auto;
  border-radius: 4px;
}
</style>
