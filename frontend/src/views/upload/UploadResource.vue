<template>
  <div class="upload-resource-page">
    <div class="page-header">
      <h1>上传资源</h1>
      <p class="subtitle">分享你的学习资料，帮助更多同学</p>
    </div>

    <div class="upload-container">
      <!-- 步骤条 -->
      <el-steps :active="currentStep" finish-status="success" simple class="upload-steps">
        <el-step title="选择文件" />
        <el-step title="填写信息" />
        <el-step title="AI审核" />
        <el-step title="完成" />
      </el-steps>

      <!-- 步骤1: 选择文件 -->
      <div v-if="currentStep === 0" class="step-content">
        <!-- 上传方式选项卡 -->
        <el-tabs v-model="uploadMode" class="upload-mode-tabs">
          <el-tab-pane label="上传文件" name="file">
            <FileUploader
              v-model="selectedFile"
              :max-size-mb="100"
              :disabled="isUploading"
            />
          </el-tab-pane>
          <el-tab-pane label="在线编写 Markdown" name="markdown">
            <div class="markdown-editor-section">
              <div class="editor-header">
                <el-input
                  v-model="markdownFileName"
                  placeholder="请输入文件名（不含扩展名）"
                  class="filename-input"
                >
                  <template #append>.md</template>
                </el-input>
              </div>
              <MarkdownEditor
                v-model="markdownContent"
                :auto-save-key="'upload_markdown_draft'"
                placeholder="开始编写你的 Markdown 内容..."
                style="height: 500px;"
              />
            </div>
          </el-tab-pane>
        </el-tabs>

        <div class="step-actions">
          <el-button
            type="primary"
            size="large"
            :disabled="!canProceedToStep2"
            @click="goToStep(1)"
          >
            下一步
            <el-icon class="el-icon--right"><ArrowRight /></el-icon>
          </el-button>
        </div>
      </div>

      <!-- 步骤2: 填写信息 -->
      <div v-if="currentStep === 1" class="step-content">
        <el-card class="file-preview-card">
          <div class="file-preview">
            <el-icon class="file-icon"><Document /></el-icon>
            <div class="file-info">
              <span class="file-name">{{ selectedFile?.name }}</span>
              <span class="file-size">{{ formatFileSize(selectedFile?.size) }}</span>
            </div>
            <el-button type="primary" link @click="goToStep(0)">
              {{ uploadMode === 'file' ? '更换文件' : '返回编辑' }}
            </el-button>
          </div>
        </el-card>

        <MetadataForm
          ref="metadataFormRef"
          v-model="metadata"
          :resource-type="detectedResourceType"
        />

        <div class="step-actions">
          <el-button size="large" @click="goToStep(0)">上一步</el-button>
          <el-button type="primary" size="large" :loading="isUploading" @click="handleUpload">
            开始上传
          </el-button>
        </div>
      </div>

      <!-- 步骤3: AI审核 -->
      <div v-if="currentStep === 2" class="step-content">
        <div class="ai-audit-status">
          <el-icon v-if="auditStatus === 'checking'" class="audit-icon is-loading"><Loading /></el-icon>
          <el-icon v-else-if="auditStatus === 'passed'" class="audit-icon is-success"><CircleCheck /></el-icon>
          <el-icon v-else class="audit-icon is-error"><CircleClose /></el-icon>

          <h3>{{ auditStatusText }}</h3>
          <p v-if="auditMessage" class="audit-message">{{ auditMessage }}</p>

          <!-- 审核进度条 -->
          <div v-if="auditStatus === 'checking'" class="audit-progress">
            <el-progress :percentage="uploadProgress" :stroke-width="8" />
            <p class="progress-text">正在上传和审核...</p>
          </div>
        </div>

        <div v-if="auditStatus !== 'checking'" class="step-actions">
          <el-button
            v-if="auditStatus === 'rejected'"
            size="large"
            @click="currentStep = 0"
          >
            重新上传
          </el-button>
          <el-button
            v-if="auditStatus === 'passed'"
            type="primary"
            size="large"
            @click="goToResourceDetail"
          >
            查看资源
          </el-button>
        </div>
      </div>

      <!-- 步骤4: 完成 -->
      <div v-if="currentStep === 3" class="step-content">
        <div class="upload-success">
          <el-icon class="success-icon"><CircleCheck /></el-icon>
          <h3>上传成功！</h3>
          <p>资源已通过AI审核并发布</p>

          <div class="success-actions">
            <el-button type="primary" size="large" @click="goToResourceDetail">
              查看资源
            </el-button>
            <el-button size="large" @click="resetAndUpload">
              继续上传
            </el-button>
          </div>
        </div>
      </div>
    </div>

    <!-- 上传说明 -->
    <el-card class="upload-tips" shadow="never">
      <template #header>
        <span>上传须知</span>
      </template>
      <ul>
        <li>支持上传：PDF、PPT、PPTX、DOC、DOCX、TXT、Markdown、图片、ZIP等格式</li>
        <li>在线编写 Markdown 可直接在网页中编辑并保存为 .md 文件</li>
        <li>单个文件大小限制：100MB</li>
        <li>资源需经过AI审核，违规内容将被拒绝</li>
        <li>请确保上传资源不侵犯他人版权</li>
        <li>优质资源将获得更多曝光和下载</li>
      </ul>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useRouter } from 'vue-router';
import { ElMessage } from 'element-plus';
import {
  ArrowRight,
  Document,
  Loading,
  CircleCheck,
  CircleClose
} from '@element-plus/icons-vue';
import FileUploader from '../../components/upload/FileUploader.vue';
import MetadataForm from '../../components/upload/MetadataForm.vue';
import MarkdownEditor from '../../components/editor/MarkdownEditor.vue';
import { uploadResource } from '../../api/resource';
import {
  formatFileSize,
  getResourceTypeFromFileName,
  ResourceType,
  type ResourceTypeType,
  type ResourceCategoryType
} from '../../types/resource';

const router = useRouter();

// 当前步骤
const currentStep = ref(0);

// 上传模式：'file' 或 'markdown'
const uploadMode = ref<'file' | 'markdown'>('file');

// 选中的文件（文件上传模式）
const selectedFile = ref<File | null>(null);

// Markdown 编辑器内容（在线编写模式）
const markdownContent = ref('');
const markdownFileName = ref('');

// 元数据表单引用
const metadataFormRef = ref<InstanceType<typeof MetadataForm>>();

// 元数据（使用 ref 而非 reactive，因为 v-model 需要可重新赋值的绑定）
const metadata = ref({
  title: '',
  courseName: '',
  category: 'other' as ResourceCategoryType,
  tags: [] as string[],
  description: ''
});

// 上传状态
const isUploading = ref(false);
const uploadProgress = ref(0);

// 审核状态
const auditStatus = ref<'checking' | 'passed' | 'rejected'>('checking');
const auditMessage = ref('');
const uploadedResourceId = ref('');

// 检测到的资源类型
const detectedResourceType = computed<ResourceTypeType>(() => {
  if (uploadMode.value === 'markdown') {
    return ResourceType.WebMarkdown;
  }
  if (!selectedFile.value) return 'other';
  return getResourceTypeFromFileName(selectedFile.value.name);
});

// 是否可以进入步骤2
const canProceedToStep2 = computed(() => {
  if (uploadMode.value === 'file') {
    return !!selectedFile.value;
  } else {
    // Markdown 模式：需要文件名和内容
    return markdownFileName.value.trim() && markdownContent.value.trim();
  }
});

// 审核状态文本
const auditStatusText = computed(() => {
  switch (auditStatus.value) {
    case 'checking':
      return 'AI 正在审核资源...';
    case 'passed':
      return 'AI 审核通过！';
    case 'rejected':
      return 'AI 审核未通过';
    default:
      return '';
  }
});

// 跳转到指定步骤
const goToStep = (step: number) => {
  // 如果从步骤2返回步骤1，且是markdown模式，需要清空selectedFile
  if (currentStep.value === 1 && step === 0 && uploadMode.value === 'markdown') {
    selectedFile.value = null;
  }
  currentStep.value = step;
};

// 将 Markdown 内容转换为 File 对象
const createMarkdownFile = (): File => {
  const blob = new Blob([markdownContent.value], { type: 'text/markdown' });
  const fileName = markdownFileName.value.endsWith('.md')
    ? markdownFileName.value
    : `${markdownFileName.value}.md`;
  return new File([blob], fileName, { type: 'text/markdown' });
};

// 处理上传
const handleUpload = async () => {
  // 如果是 Markdown 模式，先创建文件
  if (uploadMode.value === 'markdown') {
    selectedFile.value = createMarkdownFile();
  }

  if (!selectedFile.value) {
    ElMessage.warning(uploadMode.value === 'file' ? '请先选择文件' : '请编写 Markdown 内容');
    return;
  }

  // 验证表单
  const isValid = await metadataFormRef.value?.validate();
  if (!isValid) return;

  // 开始上传
  isUploading.value = true;
  uploadProgress.value = 0;
  auditStatus.value = 'checking';
  currentStep.value = 2;

  try {
    const request = {
      title: metadata.value.title,
      courseName: metadata.value.courseName || undefined,
      resourceType: detectedResourceType.value,
      category: metadata.value.category,
      tags: metadata.value.tags.length > 0 ? metadata.value.tags : undefined,
      description: metadata.value.description || undefined
    };

    const response = await uploadResource(
      request,
      selectedFile.value,
      (progress) => {
        uploadProgress.value = progress;
      }
    );

    uploadedResourceId.value = response.id;
    auditStatus.value = 'passed';
    auditMessage.value = response.aiMessage || '资源已通过AI审核并发布';

    // 清除 Markdown 草稿
    if (uploadMode.value === 'markdown') {
      localStorage.removeItem('markdown_draft_upload_markdown_draft');
    }

    // 延迟后显示成功页面
    setTimeout(() => {
      currentStep.value = 3;
    }, 1000);

    ElMessage.success('上传成功！');
  } catch (error: any) {
    auditStatus.value = 'rejected';
    auditMessage.value = error.message || '上传失败，请重试';
    if (!error.isHandled) {
      ElMessage.error(error.message || '上传失败');
    }
  } finally {
    isUploading.value = false;
  }
};

// 查看资源详情
const goToResourceDetail = () => {
  if (uploadedResourceId.value) {
    router.push(`/resources/${uploadedResourceId.value}`);
  }
};

// 重置并继续上传
const resetAndUpload = () => {
  selectedFile.value = null;
  markdownContent.value = '';
  markdownFileName.value = '';
  metadata.value.title = '';
  metadata.value.courseName = '';
  metadata.value.category = 'other';
  metadata.value.tags = [];
  metadata.value.description = '';
  uploadProgress.value = 0;
  auditStatus.value = 'checking';
  auditMessage.value = '';
  uploadedResourceId.value = '';
  currentStep.value = 0;
  uploadMode.value = 'file';
  metadataFormRef.value?.resetFields();
};
</script>

<style scoped>
.upload-resource-page {
  max-width: 800px;
  margin: 0 auto;
  padding: 24px;
}

.page-header {
  text-align: center;
  margin-bottom: 32px;
}

.page-header h1 {
  font-size: 28px;
  font-weight: 600;
  margin-bottom: 8px;
  color: var(--el-text-color-primary);
}

.subtitle {
  color: var(--el-text-color-secondary);
  font-size: 16px;
}

.upload-container {
  background: var(--el-bg-color);
  border-radius: 8px;
  padding: 32px;
  margin-bottom: 24px;
}

.upload-steps {
  margin-bottom: 32px;
}

.step-content {
  min-height: 300px;
}

.step-actions {
  display: flex;
  justify-content: center;
  gap: 16px;
  margin-top: 32px;
}

/* 上传模式选项卡样式 */
.upload-mode-tabs {
  margin-bottom: 16px;
}

.upload-mode-tabs :deep(.el-tabs__header) {
  margin-bottom: 24px;
}

.markdown-editor-section {
  border: 1px solid var(--el-border-color);
  border-radius: 8px;
  overflow: hidden;
}

.editor-header {
  padding: 16px;
  background-color: var(--el-fill-color-lighter);
  border-bottom: 1px solid var(--el-border-color);
}

.filename-input {
  max-width: 400px;
}

.file-preview-card {
  margin-bottom: 24px;
}

.file-preview {
  display: flex;
  align-items: center;
  gap: 16px;
}

.file-icon {
  font-size: 40px;
  color: var(--el-color-primary);
}

.file-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.file-name {
  font-size: 16px;
  font-weight: 500;
  color: var(--el-text-color-primary);
}

.file-size {
  font-size: 14px;
  color: var(--el-text-color-secondary);
}

.ai-audit-status {
  text-align: center;
  padding: 48px 0;
}

.audit-icon {
  font-size: 64px;
  margin-bottom: 24px;
}

.audit-icon.is-loading {
  color: var(--el-color-primary);
  animation: rotating 2s linear infinite;
}

.audit-icon.is-success {
  color: var(--el-color-success);
}

.audit-icon.is-error {
  color: var(--el-color-danger);
}

@keyframes rotating {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.ai-audit-status h3 {
  font-size: 20px;
  font-weight: 500;
  margin-bottom: 12px;
}

.audit-message {
  color: var(--el-text-color-secondary);
  margin-bottom: 24px;
}

.audit-progress {
  max-width: 400px;
  margin: 0 auto;
}

.progress-text {
  margin-top: 12px;
  color: var(--el-text-color-secondary);
}

.upload-success {
  text-align: center;
  padding: 48px 0;
}

.success-icon {
  font-size: 80px;
  color: var(--el-color-success);
  margin-bottom: 24px;
}

.upload-success h3 {
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 12px;
}

.upload-success p {
  color: var(--el-text-color-secondary);
  margin-bottom: 32px;
}

.success-actions {
  display: flex;
  justify-content: center;
  gap: 16px;
}

.upload-tips {
  background: var(--el-fill-color-lighter);
}

.upload-tips ul {
  padding-left: 20px;
  margin: 0;
}

.upload-tips li {
  margin-bottom: 8px;
  color: var(--el-text-color-regular);
  line-height: 1.6;
}

.upload-tips li:last-child {
  margin-bottom: 0;
}
</style>
