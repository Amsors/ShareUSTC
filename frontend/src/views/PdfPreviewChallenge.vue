<template>
  <div class="challenge-page">
    <div class="challenge-container">
      <h1 class="page-title">PDF 预览检测</h1>

      <div v-if="loading" class="loading-state">
        <el-icon class="loading-icon" :size="32"><Loading /></el-icon>
        <p>正在加载检测资源...</p>
      </div>

      <div v-else-if="!config?.enabled" class="error-state">
        <el-icon :size="48" color="var(--el-color-warning)"><Warning /></el-icon>
        <p>PDF 预览检测功能尚未配置</p>
        <p class="sub-text">请联系管理员进行配置</p>
      </div>

      <div v-else-if="verified" class="success-state">
        <el-icon :size="64" color="var(--el-color-success)"><CircleCheck /></el-icon>
        <h2>检测通过</h2>
        <p>您的浏览器支持 PDF 预览功能</p>
        <p class="sub-text">已为您开启"自动加载 PDF 资料预览"功能</p>
        <el-button type="primary" @click="goToSettings"> 前往设置页面 </el-button>
      </div>

      <div v-else class="challenge-content">
        <el-alert type="info" :closable="false" class="challenge-info">
          <template #title>
            <strong>检测说明</strong>
          </template>
          <ul>
            <li>请在下方预览 PDF 文件，找到文件中显示的四位数字</li>
            <li>将四位数字输入到下方输入框中</li>
            <li>输入正确即可通过检测，开启自动加载 PDF 预览功能</li>
          </ul>
        </el-alert>

        <div class="preview-section">
          <div class="preview-header">
            <span class="preview-title">PDF 预览</span>
            <span class="preview-hint">请查看此 PDF 中的四位数字</span>
          </div>
          <div class="preview-container">
            <PdfViewer
              v-if="config.resourceId"
              :resource-id="config.resourceId"
              resource-type="pdf"
              resource-title="PDF预览检测"
              :auto-load="true"
              hide-download-on-error
            />
            <div v-else class="preview-error">
              <p>检测资源未配置</p>
            </div>
          </div>
        </div>

        <div class="input-section">
          <el-form @submit.prevent="handleSubmit">
            <el-form-item label="请输入 PDF 中的四位数字">
              <el-input
                v-model="inputCode"
                placeholder="请输入四位数字"
                maxlength="4"
                size="large"
                class="code-input"
              >
                <template #prefix>
                  <el-icon><Key /></el-icon>
                </template>
              </el-input>
            </el-form-item>

            <el-form-item>
              <el-button
                type="primary"
                size="large"
                :loading="verifying"
                :disabled="!isValidCode"
                @click="handleSubmit"
                class="submit-btn"
              >
                提交验证
              </el-button>
            </el-form-item>
          </el-form>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { ElMessage } from 'element-plus';
import { Loading, Warning, CircleCheck, Key } from '@element-plus/icons-vue';
import PdfViewer from '../components/preview/PdfViewer.vue';
import {
  getPdfPreviewChallengeConfig,
  verifyPdfPreviewChallenge,
  type PdfPreviewChallengeConfig,
} from '../api/resource';
import logger from '../utils/logger';

const router = useRouter();
const loading = ref(true);
const config = ref<PdfPreviewChallengeConfig | null>(null);
const inputCode = ref('');
const verifying = ref(false);
const verified = ref(false);

// LocalStorage keys
const PDF_PREVIEW_VERIFIED_KEY = 'pdfPreviewVerified';
const PDF_PREVIEW_AUTO_LOAD_KEY = 'pdfPreviewAutoLoad';
const PDF_PREVIEW_USER_ENABLED_KEY = 'pdfPreviewUserEnabled';

// 检查是否已验证
const checkVerified = () => {
  try {
    const stored = localStorage.getItem(PDF_PREVIEW_VERIFIED_KEY);
    if (stored) {
      const data = JSON.parse(stored);
      if (data.verified) {
        verified.value = true;
      }
    }
  } catch (e) {
    logger.warn('[PdfPreviewChallenge]', 'Failed to parse verified status:', e);
  }
};

// 验证输入是否为四位数字
const isValidCode = computed(() => {
  return /^\d{4}$/.test(inputCode.value);
});

// 加载配置
const loadConfig = async () => {
  loading.value = true;
  try {
    const data = await getPdfPreviewChallengeConfig();
    config.value = data;
    logger.info('[PdfPreviewChallenge]', 'Config loaded:', data);
  } catch (error) {
    logger.error('[PdfPreviewChallenge]', 'Failed to load config:', error);
    ElMessage.error('加载配置失败');
  } finally {
    loading.value = false;
  }
};

// 提交验证
const handleSubmit = async () => {
  if (!isValidCode.value) {
    ElMessage.warning('请输入四位数字');
    return;
  }

  verifying.value = true;
  try {
    const result = await verifyPdfPreviewChallenge(inputCode.value);
    if (result.success) {
      // 验证成功，保存状态
      localStorage.setItem(
        PDF_PREVIEW_VERIFIED_KEY,
        JSON.stringify({
          verified: true,
          timestamp: Date.now(),
        })
      );
      // 开启自动加载
      localStorage.setItem(
        PDF_PREVIEW_AUTO_LOAD_KEY,
        JSON.stringify({
          enabled: true,
          timestamp: Date.now(),
        })
      );
      // 允许用户手动控制
      localStorage.setItem(
        PDF_PREVIEW_USER_ENABLED_KEY,
        JSON.stringify({
          enabled: true,
          timestamp: Date.now(),
        })
      );

      verified.value = true;
      ElMessage.success('验证成功！已为您开启自动加载 PDF 预览功能');
    } else {
      ElMessage.error(result.message || '验证码错误');
      inputCode.value = '';
    }
  } catch (error) {
    logger.error('[PdfPreviewChallenge]', 'Verification failed:', error);
    ElMessage.error('验证失败，请稍后重试');
  } finally {
    verifying.value = false;
  }
};

// 前往设置页面
const goToSettings = () => {
  router.push('/settings');
};

onMounted(() => {
  checkVerified();
  loadConfig();
});
</script>

<style scoped>
.challenge-page {
  min-height: 100vh;
  background-color: #f5f7fa;
  padding: 40px 20px;
}

.challenge-container {
  max-width: 900px;
  margin: 0 auto;
}

.page-title {
  margin: 0 0 32px;
  font-size: 28px;
  color: #303133;
  font-weight: 600;
  text-align: center;
}

.loading-state,
.error-state,
.success-state {
  background: #fff;
  border-radius: 12px;
  padding: 60px 40px;
  text-align: center;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.05);
}

.loading-icon {
  animation: rotating 2s linear infinite;
  margin-bottom: 16px;
}

@keyframes rotating {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.error-state .el-icon,
.success-state .el-icon {
  margin-bottom: 16px;
}

.error-state p,
.success-state p {
  margin: 8px 0;
  font-size: 16px;
  color: #606266;
}

.error-state .sub-text,
.success-state .sub-text {
  font-size: 14px;
  color: #909399;
  margin-bottom: 24px;
}

.success-state h2 {
  margin: 16px 0 8px;
  color: #67c23a;
  font-size: 24px;
}

.challenge-content {
  background: #fff;
  border-radius: 12px;
  padding: 32px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.05);
}

.challenge-info {
  margin-bottom: 24px;
}

.challenge-info ul {
  margin: 8px 0 0;
  padding-left: 20px;
}

.challenge-info li {
  margin: 4px 0;
  line-height: 1.6;
  color: #606266;
}

.preview-section {
  margin-bottom: 32px;
}

.preview-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid #ebeef5;
}

.preview-title {
  font-size: 16px;
  font-weight: 600;
  color: #303133;
}

.preview-hint {
  font-size: 13px;
  color: #909399;
}

.preview-container {
  border: 1px solid #ebeef5;
  border-radius: 8px;
  overflow: hidden;
}

.input-section {
  max-width: 500px;
  margin: 0 auto;
  padding-top: 24px;
  border-top: 1px solid #ebeef5;
}

.code-input {
  font-size: 18px;
}

.code-input :deep(.el-input__inner) {
  text-align: center;
  letter-spacing: 8px;
}

.submit-btn {
  width: 100%;
}

/* 响应式 */
@media (max-width: 768px) {
  .challenge-page {
    padding: 20px 16px;
  }

  .page-title {
    font-size: 24px;
    margin-bottom: 20px;
  }

  .challenge-content {
    padding: 20px;
  }

  .preview-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
  }

  .input-section {
    max-width: 100%;
  }
}
</style>
