<template>
  <div class="txt-viewer">
    <div v-if="loading" class="loading-container">
      <el-skeleton :rows="10" animated />
    </div>
    <div v-else-if="error" class="error-container">
      <el-empty description="文本加载失败" />
      <el-button type="primary" @click="loadContent">重试</el-button>
    </div>
    <div v-else class="text-container">
      <pre class="text-content">{{ content }}</pre>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { getResourceContent } from '../../api/resource';

const props = defineProps<{
  resourceId: string;
}>();

const loading = ref(true);
const error = ref(false);
const content = ref('');

const loadContent = async () => {
  loading.value = true;
  error.value = false;
  try {
    const blob = await getResourceContent(props.resourceId);
    const text = await blob.text();
    // 限制显示长度（防止超大文本文件）
    const maxLength = 100000; // 10万字符
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
.txt-viewer {
  width: 100%;
  min-height: 300px;
}

.loading-container,
.error-container {
  padding: 40px 0;
  text-align: center;
}

.text-container {
  background-color: var(--el-fill-color-light);
  border-radius: 8px;
  padding: 16px;
  max-height: 600px;
  overflow: auto;
}

.text-content {
  margin: 0;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 14px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-wrap: break-word;
  color: var(--el-text-color-primary);
}
</style>
