<template>
  <div class="edit-markdown-page">
    <div class="page-header">
      <div class="header-left">
        <el-button link @click="goBack">
          <el-icon><ArrowLeft /></el-icon>
          返回
        </el-button>
        <h1>编辑 Markdown 资源</h1>
      </div>
      <div class="header-actions">
        <el-button @click="loadDraft" v-if="hasDraft">
          <el-icon><Document /></el-icon>
          恢复草稿
        </el-button>
        <el-button @click="saveDraft">
          <el-icon><DocumentChecked /></el-icon>
          保存草稿
        </el-button>
        <el-button type="primary" :loading="saving" @click="handleSave">
          <el-icon><Check /></el-icon>
          保存修改
        </el-button>
      </div>
    </div>

    <div v-if="loading" class="loading-container">
      <el-icon class="loading-icon" :size="48"><Loading /></el-icon>
      <p>加载中...</p>
    </div>

    <div v-else-if="error" class="error-container">
      <el-empty :description="error" />
      <el-button type="primary" @click="loadContent">重试</el-button>
    </div>

    <template v-else>
      <div class="resource-info">
        <el-descriptions :column="3" border>
          <el-descriptions-item label="资源标题">{{ resource?.title }}</el-descriptions-item>
          <el-descriptions-item label="资源类型">
            <el-tag type="success">Markdown</el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="创建时间">{{ formatTime(resource?.createdAt) }}</el-descriptions-item>
        </el-descriptions>
      </div>

      <div class="editor-wrapper">
        <MarkdownEditor
          ref="editorRef"
          v-model="content"
          :resource-id="resourceId"
          :auto-save-key="`resource_${resourceId}`"
          @change="handleContentChange"
          @save-draft="onDraftSaved"
        />
      </div>
    </template>

    <!-- 预览对话框 -->
    <el-dialog
      v-model="previewVisible"
      title="预览"
      width="900px"
      destroy-on-close
    >
      <div class="preview-content markdown-body" v-html="renderedContent"></div>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { ElMessage, ElMessageBox } from 'element-plus';
import {
  ArrowLeft,
  Check,
  Document,
  DocumentChecked,
  Loading
} from '@element-plus/icons-vue';
import MarkdownIt from 'markdown-it';
import MarkdownEditor from '../../components/editor/MarkdownEditor.vue';
import { getResourceDetail, getResourceRawContent, updateResourceContent } from '../../api/resource';
import type { ResourceDetail } from '../../types/resource';

const route = useRoute();
const router = useRouter();
const resourceId = route.params.id as string;

// 状态
const loading = ref(false);
const saving = ref(false);
const error = ref('');
const content = ref('');
const resource = ref<ResourceDetail | null>(null);
const hasDraft = ref(false);
const previewVisible = ref(false);

// 初始化 MarkdownIt
const md = new MarkdownIt({
  html: false,
  breaks: true,
  linkify: true,
  typographer: true
});

// 渲染预览内容
const renderedContent = computed(() => {
  return md.render(content.value);
});

// 加载资源详情
const loadResource = async () => {
  try {
    resource.value = await getResourceDetail(resourceId);
    // 检查是否是 Markdown 类型
    if (resource.value.resourceType !== 'web_markdown') {
      error.value = '只有 Markdown 类型资源可以在线编辑';
      return;
    }
  } catch (err: any) {
    error.value = err.message || '加载资源失败';
  }
};

// 加载内容
const loadContent = async () => {
  loading.value = true;
  error.value = '';
  try {
    await loadResource();
    if (error.value) return;

    const response = await getResourceRawContent(resourceId);
    content.value = response.content;
    checkDraft();
  } catch (err: any) {
    error.value = err.message || '加载内容失败';
  } finally {
    loading.value = false;
  }
};

// 处理内容变化
const handleContentChange = () => {
  checkDraft();
};

// 检查是否有草稿
const checkDraft = () => {
  const draft = localStorage.getItem(`markdown_draft_resource_${resourceId}`);
  hasDraft.value = !!draft && draft !== content.value;
};

// 保存草稿
const saveDraft = () => {
  if (content.value) {
    localStorage.setItem(`markdown_draft_resource_${resourceId}`, content.value);
    hasDraft.value = false;
    ElMessage.success('草稿已保存');
  }
};

// 加载草稿
const loadDraft = async () => {
  const draft = localStorage.getItem(`markdown_draft_resource_${resourceId}`);
  if (draft) {
    try {
      await ElMessageBox.confirm(
        '确定要加载草稿吗？当前内容将被覆盖。',
        '确认加载草稿',
        {
          confirmButtonText: '加载',
          cancelButtonText: '取消',
          type: 'warning'
        }
      );
      content.value = draft;
      ElMessage.success('草稿已加载');
    } catch {
      // 用户取消
    }
  }
};

// 草稿保存回调
const onDraftSaved = () => {
  ElMessage.success('草稿已自动保存');
};

// 保存修改
const handleSave = async () => {
  if (!content.value.trim()) {
    ElMessage.warning('内容不能为空');
    return;
  }

  saving.value = true;
  try {
    await updateResourceContent(resourceId, {
      content: content.value
    });

    // 清除草稿
    localStorage.removeItem(`markdown_draft_resource_${resourceId}`);
    hasDraft.value = false;

    ElMessage.success('保存成功');

    // 询问是否返回详情页
    try {
      await ElMessageBox.confirm(
        '修改已保存，是否返回资源详情页？',
        '保存成功',
        {
          confirmButtonText: '返回详情页',
          cancelButtonText: '继续编辑',
          type: 'success'
        }
      );
      router.push(`/resources/${resourceId}`);
    } catch {
      // 用户选择继续编辑
    }
  } catch (err: any) {
    ElMessage.error(err.message || '保存失败');
  } finally {
    saving.value = false;
  }
};

// 返回上一页
const goBack = () => {
  if (hasDraft.value) {
    ElMessageBox.confirm(
      '您有未保存的草稿，确定要离开吗？',
      '确认离开',
      {
        confirmButtonText: '离开',
        cancelButtonText: '取消',
        type: 'warning'
      }
    ).then(() => {
      router.back();
    }).catch(() => {
      // 用户取消
    });
  } else {
    router.back();
  }
};

// 格式化时间
const formatTime = (time?: string) => {
  if (!time) return '-';
  const utcTimeString = time.endsWith('Z') ? time : `${time}Z`;
  return new Date(utcTimeString).toLocaleString('zh-CN');
};

// 页面加载提示
onMounted(() => {
  loadContent();

  // 监听页面关闭事件，提醒保存
  window.addEventListener('beforeunload', (e) => {
    if (hasDraft.value) {
      e.preventDefault();
      e.returnValue = '';
    }
  });
});
</script>

<style scoped>
.edit-markdown-page {
  min-height: 100vh;
  background-color: #f5f7fa;
  display: flex;
  flex-direction: column;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 24px;
  background: #fff;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  position: sticky;
  top: 0;
  z-index: 100;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.header-left h1 {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  color: #303133;
}

.header-actions {
  display: flex;
  gap: 12px;
}

.loading-container,
.error-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  gap: 16px;
  padding: 48px;
}

.loading-icon {
  animation: rotating 2s linear infinite;
  color: #409eff;
}

@keyframes rotating {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.resource-info {
  padding: 16px 24px;
  background: #fff;
  border-bottom: 1px solid #e4e7ed;
}

.editor-wrapper {
  flex: 1;
  padding: 16px 24px;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.editor-wrapper :deep(.markdown-editor-wrapper) {
  flex: 1;
  min-height: 0;
}

/* 预览对话框样式 */
.preview-content {
  max-height: 600px;
  overflow: auto;
  padding: 16px;
}

.markdown-body :deep(h1) {
  font-size: 2em;
  border-bottom: 1px solid #e4e7ed;
  padding-bottom: 0.3em;
  margin-bottom: 1em;
}

.markdown-body :deep(h2) {
  font-size: 1.5em;
  border-bottom: 1px solid #e4e7ed;
  padding-bottom: 0.3em;
  margin: 1.5em 0 1em;
}

.markdown-body :deep(h3) {
  font-size: 1.25em;
  margin: 1.5em 0 1em;
}

.markdown-body :deep(p) {
  margin: 1em 0;
  line-height: 1.8;
}

.markdown-body :deep(img) {
  max-width: 100%;
  height: auto;
  border-radius: 4px;
}

.markdown-body :deep(code) {
  background-color: #f5f7fa;
  padding: 0.2em 0.4em;
  border-radius: 3px;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
}

.markdown-body :deep(pre) {
  background-color: #f5f7fa;
  padding: 16px;
  border-radius: 8px;
  overflow-x: auto;
}

.markdown-body :deep(pre code) {
  background-color: transparent;
  padding: 0;
}

.markdown-body :deep(blockquote) {
  border-left: 4px solid #409eff;
  padding-left: 1em;
  margin: 1em 0;
  color: #606266;
}

.markdown-body :deep(table) {
  width: 100%;
  border-collapse: collapse;
  margin: 1em 0;
}

.markdown-body :deep(th),
.markdown-body :deep(td) {
  border: 1px solid #dcdfe6;
  padding: 8px 12px;
  text-align: left;
}

.markdown-body :deep(th) {
  background-color: #f5f7fa;
}
</style>
