<template>
  <div class="pdf-viewer">
    <!-- 懒加载提示 -->
    <div v-if="showLazyLoadPrompt" class="lazy-load-container">
      <el-icon :size="48" color="var(--el-color-info)"><Document /></el-icon>
      <div class="lazy-load-message">
        <p class="lazy-load-title">PDF 预览</p>
        <p v-if="!isUserVerified" class="lazy-load-desc">通过检测后可自动加载预览</p>
      </div>
      <div class="lazy-load-actions">
        <el-button type="primary" size="large" @click="loadPdf">
          <el-icon class="el-icon--left"><View /></el-icon>加载预览
        </el-button>
        <el-button v-if="!isUserVerified" size="large" @click="goToChallenge">
          <el-icon class="el-icon--left"><Check /></el-icon>前往检测页面
        </el-button>
        <el-button v-else size="large" @click="enableAutoLoad">
          <el-icon class="el-icon--left"><Check /></el-icon>以后自动加载预览
        </el-button>
      </div>
    </div>

    <!-- 文件超过阈值 -->
    <div v-else-if="isFileTooLarge" class="lazy-load-container file-too-large">
      <el-icon :size="48" color="var(--el-color-warning)"><Warning /></el-icon>
      <div class="lazy-load-message">
        <p class="lazy-load-title">文件大小超过设定阈值</p>
        <p class="lazy-load-desc">
          当前文件大小 {{ formatFileSize(props.fileSize || 0) }}，超过自动加载阈值
          {{ getThresholdMB() }}MB
        </p>
        <p class="lazy-load-hint">您可以手动加载此文件，或调整自动加载阈值</p>
      </div>
      <div class="lazy-load-actions">
        <el-button type="primary" size="large" @click="loadPdf">
          <el-icon class="el-icon--left"><View /></el-icon>加载预览
        </el-button>
        <el-button size="large" @click="goToSettings">
          <el-icon class="el-icon--left"><Setting /></el-icon>编辑自动加载阈值
        </el-button>
      </div>
    </div>

    <!-- 加载中状态 -->
    <div v-else-if="loading" class="loading-container">
      <p class="loading-text">正在加载 PDF...</p>
    </div>

    <div v-else-if="error" class="error-container">
      <el-icon :size="64" color="var(--el-color-danger)"><DocumentDelete /></el-icon>
      <div class="error-message">
        <p class="error-line">
          <template v-if="hideDownloadOnError">你的浏览器不支持PDF预览！！！</template>
          <template v-else>PDF预览加载失败，请下载后自行预览</template>
        </p>
        <p class="error-line">请使用适配本网站的浏览器，如chrome、firefox、edge、via</p>
        <p class="error-line"><strong>不要使用</strong>系统自带、百度、夸克、QQ、UC等浏览器</p>
        <p class="error-line">如果更换浏览器后仍无法加载，请联系我们</p>
      </div>
      <div class="error-actions">
        <el-button v-if="!hideDownloadOnError" type="primary" @click="downloadPdf">
          <el-icon class="el-icon--left"><Download /></el-icon>下载PDF
        </el-button>
        <el-button @click="loadPdf">重试加载</el-button>
      </div>
    </div>

    <div v-else class="pdf-container">
      <!-- 工具栏 -->
      <div class="pdf-toolbar">
        <div class="toolbar-left">
          <el-button circle size="small" :disabled="currentPage <= 1" @click="prevPage">
            <el-icon><ArrowLeft /></el-icon>
          </el-button>
          <span class="page-info">
            <el-input-number
              v-model="currentPage"
              :min="1"
              :max="totalPages"
              size="small"
              class="page-input"
              @change="goToPage"
            />
            <span class="page-total">/ {{ totalPages }}</span>
          </span>
          <el-button circle size="small" :disabled="currentPage >= totalPages" @click="nextPage">
            <el-icon><ArrowRight /></el-icon>
          </el-button>
        </div>

        <div class="toolbar-right">
          <el-button circle size="small" :disabled="scale <= 0.5" @click="zoomOut">
            <el-icon><ZoomOut /></el-icon>
          </el-button>
          <span class="zoom-info">{{ Math.round(scale * 100) }}%</span>
          <el-button circle size="small" :disabled="scale >= 3" @click="zoomIn">
            <el-icon><ZoomIn /></el-icon>
          </el-button>
          <el-button circle size="small" @click="toggleFullscreen">
            <el-icon><FullScreen /></el-icon>
          </el-button>
        </div>
      </div>

      <!-- PDF 渲染区域 -->
      <div ref="pdfContentRef" class="pdf-content">
        <canvas ref="canvasRef" class="pdf-canvas"></canvas>
      </div>
    </div>

    <!-- 全屏预览 -->
    <el-dialog
      v-model="fullscreen"
      title="PDF 预览"
      width="95%"
      destroy-on-close
      class="fullscreen-pdf"
    >
      <div class="fullscreen-toolbar">
        <el-button circle size="small" :disabled="currentPage <= 1" @click="prevPage">
          <el-icon><ArrowLeft /></el-icon>
        </el-button>
        <span class="page-info">{{ currentPage }} / {{ totalPages }}</span>
        <el-button circle size="small" :disabled="currentPage >= totalPages" @click="nextPage">
          <el-icon><ArrowRight /></el-icon>
        </el-button>
        <el-divider direction="vertical" />
        <el-button circle size="small" :disabled="scale <= 0.5" @click="zoomOut">
          <el-icon><ZoomOut /></el-icon>
        </el-button>
        <span class="zoom-info">{{ Math.round(scale * 100) }}%</span>
        <el-button circle size="small" :disabled="scale >= 3" @click="zoomIn">
          <el-icon><ZoomIn /></el-icon>
        </el-button>
      </div>
      <div class="fullscreen-content">
        <canvas ref="fullscreenCanvasRef" class="fullscreen-canvas"></canvas>
      </div>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick, computed } from 'vue';
import { useRouter } from 'vue-router';
import {
  ArrowLeft,
  ArrowRight,
  ZoomIn,
  ZoomOut,
  FullScreen,
  Download,
  DocumentDelete,
  Document,
  View,
  Check,
  Warning,
  Setting,
} from '@element-plus/icons-vue';
import * as pdfjsLib from 'pdfjs-dist';
import PDFWorker from 'pdfjs-dist/build/pdf.worker.mjs?worker';
import {
  getResourcePreviewInfo,
  getResourcePreviewContent,
  downloadResource,
  type PreviewUrlResponse,
} from '../../api/resource';
import logger from '../../utils/logger';
import { ElMessage } from 'element-plus';

// 设置 PDF.js worker - 使用 Vite 的 worker 导入
pdfjsLib.GlobalWorkerOptions.workerPort = new PDFWorker();

const props = defineProps<{
  resourceId: string;
  resourceType?: string;
  resourceTitle?: string;
  /**
   * 是否强制自动加载 PDF，不检查用户验证状态
   * 用于 PDF 预览检测页面等场景
   */
  autoLoad?: boolean;
  /**
   * 文件大小（字节）
   * 用于判断是否超过自动加载阈值
   */
  fileSize?: number;
  /**
   * 是否在错误状态下隐藏下载按钮
   * 用于 PDF 预览检测页面等场景
   */
  hideDownloadOnError?: boolean;
}>();

const router = useRouter();
const loading = ref(true);
const error = ref(false);
const currentPage = ref(1);
const totalPages = ref(0);
const scale = ref(1.2);
const fullscreen = ref(false);
const showLazyLoadPrompt = ref(false);
const isFileTooLarge = ref(false); // 文件是否超过阈值

// LocalStorage keys
const PDF_PREVIEW_VERIFIED_KEY = 'pdfPreviewVerified';
const PDF_PREVIEW_AUTO_LOAD_KEY = 'pdfPreviewAutoLoad';
const PDF_PREVIEW_USER_ENABLED_KEY = 'pdfPreviewUserEnabled';

// 默认阈值 5MB
const DEFAULT_SIZE_THRESHOLD = 5 * 1024 * 1024;

// 使用普通变量而非 ref，避免 Vue 响应式代理破坏 PDF.js 内部私有成员
let pdfDoc: any = null;
const canvasRef = ref<HTMLCanvasElement | null>(null);
const fullscreenCanvasRef = ref<HTMLCanvasElement | null>(null);

// 检查用户是否已通过验证
const isUserVerified = computed(() => {
  try {
    const verifiedData = localStorage.getItem(PDF_PREVIEW_VERIFIED_KEY);
    if (!verifiedData) {
      return false;
    }
    const verified = JSON.parse(verifiedData);
    return verified.verified === true;
  } catch {
    return false;
  }
});

// 检查是否应该自动加载
const shouldAutoLoad = (): boolean => {
  // 如果强制自动加载，直接返回 true
  if (props.autoLoad) {
    return true;
  }

  try {
    // 检查用户是否已通过验证
    const verifiedData = localStorage.getItem(PDF_PREVIEW_VERIFIED_KEY);
    if (!verifiedData) {
      return false;
    }
    const verified = JSON.parse(verifiedData);
    if (!verified.verified) {
      return false;
    }

    // 检查用户是否手动禁用了自动加载
    const userEnabledData = localStorage.getItem(PDF_PREVIEW_USER_ENABLED_KEY);
    if (userEnabledData) {
      const userEnabled = JSON.parse(userEnabledData);
      if (!userEnabled.enabled) {
        return false;
      }
    }

    // 检查自动加载设置
    const autoLoadData = localStorage.getItem(PDF_PREVIEW_AUTO_LOAD_KEY);
    if (autoLoadData) {
      const autoLoad = JSON.parse(autoLoadData);
      return autoLoad.enabled === true;
    }

    return false;
  } catch (e) {
    logger.warn('[PdfViewer]', 'Failed to check auto load setting:', e);
    return false;
  }
};

// 启用自动加载
const enableAutoLoad = () => {
  try {
    localStorage.setItem(
      PDF_PREVIEW_AUTO_LOAD_KEY,
      JSON.stringify({
        enabled: true,
        timestamp: Date.now(),
      })
    );
    localStorage.setItem(
      PDF_PREVIEW_USER_ENABLED_KEY,
      JSON.stringify({
        enabled: true,
        timestamp: Date.now(),
      })
    );
    ElMessage.success('已开启自动加载 PDF 预览');
    // 重新加载当前 PDF
    loadPdf();
  } catch (e) {
    logger.error('[PdfViewer]', 'Failed to enable auto load:', e);
    ElMessage.error('设置保存失败');
  }
};

// 获取大小阈值（字节）
const getSizeThreshold = (): number => {
  try {
    const thresholdData = localStorage.getItem('pdfPreviewSizeThreshold');
    if (thresholdData) {
      const threshold = JSON.parse(thresholdData);
      return (threshold.value || 5) * 1024 * 1024; // 转换为字节
    }
  } catch (e) {
    logger.warn('[PdfViewer]', 'Failed to get threshold:', e);
  }
  return DEFAULT_SIZE_THRESHOLD;
};

// 检查文件是否超过阈值
const isOverThreshold = (): boolean => {
  if (!props.fileSize || props.fileSize <= 0) {
    return false; // 如果没有文件大小信息，默认不检查
  }
  const threshold = getSizeThreshold();
  return props.fileSize > threshold;
};

// 格式化文件大小
const formatFileSize = (bytes: number): string => {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

// 获取阈值（MB）用于显示
const getThresholdMB = (): number => {
  return Math.round(getSizeThreshold() / 1024 / 1024);
};

// 前往设置页面编辑阈值
const goToSettings = () => {
  router.push('/settings');
};

// 超时包装函数
const withTimeout = <T,>(promise: Promise<T>, timeoutMs: number, label: string): Promise<T> => {
  return Promise.race([
    promise,
    new Promise<T>((_, reject) =>
      setTimeout(() => reject(new Error(`${label} 超时 (${timeoutMs}ms)`)), timeoutMs)
    ),
  ]);
};

const loadPdf = async () => {
  loading.value = true;
  error.value = false;
  showLazyLoadPrompt.value = false;
  isFileTooLarge.value = false;
  try {
    logger.debug('[PdfViewer]', `开始加载PDF | resourceId=${props.resourceId}`);

    // 获取预览信息（带缓存）
    const previewInfo: PreviewUrlResponse = await getResourcePreviewInfo(props.resourceId);
    logger.debug(
      '[PdfViewer]',
      `获取到预览信息 | storageType=${previewInfo.storageType}, directAccess=${previewInfo.directAccess}`
    );

    // 获取内容（会自动使用缓存）
    const blob = await getResourcePreviewContent(props.resourceId, previewInfo, {
      resourceDetail:
        props.resourceTitle && props.resourceType
          ? {
              title: props.resourceTitle,
              resourceType: props.resourceType,
            }
          : undefined,
    });
    logger.debug('[PdfViewer]', `获取到blob | type=${blob.type}, size=${blob.size}`);

    // 确保blob类型正确
    let pdfBlob = blob;
    if (!blob.type || blob.type === 'application/octet-stream') {
      logger.debug('[PdfViewer]', 'Blob类型不正确，强制设置为application/pdf');
      pdfBlob = new Blob([blob], { type: 'application/pdf' });
    }

    const arrayBuffer = await pdfBlob.arrayBuffer();
    logger.debug('[PdfViewer]', `转换为ArrayBuffer | size=${arrayBuffer.byteLength}`);

    const loadingTask = pdfjsLib.getDocument({
      data: arrayBuffer,
      useSystemFonts: true, // 使用系统字体支持中文显示
      cMapUrl: 'https://unpkg.com/pdfjs-dist@' + pdfjsLib.version + '/cmaps/',
      cMapPacked: true,
      disableFontFace: false, // 启用字体face以更好地支持嵌入式字体
      fontExtraProperties: true, // 保留额外字体属性
      stopAtErrors: false,
      maxImageSize: 50 * 1024 * 1024, // 最大支持 50MB 的图片
    });
    loadingTask.onProgress = (progress: any) => {
      logger.debug('[PdfViewer]', `加载进度 | loaded=${progress.loaded}, total=${progress.total}`);
    };

    // 设置20秒超时，避免在不受支持的浏览器上无限等待
    pdfDoc = await withTimeout(loadingTask.promise, 20000, 'PDF加载');
    logger.info('[PdfViewer]', `PDF文档加载成功 | pages=${pdfDoc.numPages}`);

    totalPages.value = pdfDoc.numPages;
    currentPage.value = 1;
    loading.value = false;
    // 等待 DOM 更新后再渲染，确保 canvasRef 已存在
    await nextTick();
    await renderPage();
  } catch (err: any) {
    logger.error('[PdfViewer]', 'PDF加载失败', { message: err.message, stack: err.stack });
    error.value = true;
    loading.value = false;
  }
};

const renderPage = async () => {
  if (!pdfDoc || !canvasRef.value) {
    logger.warn('[PdfViewer]', 'renderPage: pdfDoc 或 canvasRef 不存在');
    return;
  }

  logger.debug('[PdfViewer]', `开始渲染页面 | page=${currentPage.value}, scale=${scale.value}`);

  try {
    const page = await pdfDoc.getPage(currentPage.value);
    logger.debug('[PdfViewer]', `获取页面成功 | view=${JSON.stringify(page.view)}`);

    // 获取页面文本内容信息（用于调试字体问题）
    try {
      const textContent = await page.getTextContent();
      const fontNames = new Set<string>();
      textContent.items.forEach((item: any) => {
        if (item.fontName) {
          fontNames.add(item.fontName);
        }
      });
      logger.debug('[PdfViewer]', `页面使用的字体 | fonts=${Array.from(fontNames).join(', ')}`);
    } catch (e) {
      logger.debug('[PdfViewer]', '无法获取文本内容', e);
    }

    const canvas = canvasRef.value;
    const context = canvas.getContext('2d');
    if (!context) {
      logger.error('[PdfViewer]', '无法获取 canvas context');
      return;
    }

    const viewport = page.getViewport({ scale: scale.value });
    logger.debug(
      '[PdfViewer]',
      `viewport 尺寸 | width=${viewport.width}, height=${viewport.height}`
    );

    // 设置 canvas 的实际像素尺寸
    canvas.height = viewport.height;
    canvas.width = viewport.width;

    // 设置 canvas 的显示尺寸，保持正确的宽高比
    canvas.style.height = `${viewport.height}px`;
    canvas.style.width = `${viewport.width}px`;

    logger.debug('[PdfViewer]', `canvas 尺寸 | width=${canvas.width}, height=${canvas.height}`);
    logger.debug(
      '[PdfViewer]',
      `canvas 样式尺寸 | styleWidth=${canvas.style.width}, styleHeight=${canvas.style.height}`
    );

    const renderContext = {
      canvasContext: context,
      viewport: viewport,
      // 启用背景填充，避免透明背景导致的渲染问题
      background: 'white',
    };

    logger.debug('[PdfViewer]', '开始 render 操作');
    const renderTask = page.render(renderContext);
    await renderTask.promise;
    logger.debug('[PdfViewer]', '渲染完成');
  } catch (err: any) {
    logger.error('[PdfViewer]', '渲染页面失败', err);
  }
};

const renderFullscreenPage = async () => {
  if (!pdfDoc || !fullscreenCanvasRef.value) return;

  const page = await pdfDoc.getPage(currentPage.value);
  const canvas = fullscreenCanvasRef.value;
  const context = canvas.getContext('2d');
  if (!context) return;

  const viewport = page.getViewport({ scale: scale.value * 1.5 });
  canvas.height = viewport.height;
  canvas.width = viewport.width;
  canvas.style.height = `${viewport.height}px`;
  canvas.style.width = `${viewport.width}px`;

  await page.render({
    canvasContext: context,
    viewport: viewport,
    background: 'white',
  }).promise;
};

const prevPage = async () => {
  if (currentPage.value > 1) {
    currentPage.value--;
    await renderPage();
    if (fullscreen.value) {
      await renderFullscreenPage();
    }
  }
};

const nextPage = async () => {
  if (currentPage.value < totalPages.value) {
    currentPage.value++;
    await renderPage();
    if (fullscreen.value) {
      await renderFullscreenPage();
    }
  }
};

const goToPage = async (page: number | undefined) => {
  if (page && page >= 1 && page <= totalPages.value) {
    currentPage.value = page;
    await renderPage();
  }
};

const zoomIn = async () => {
  if (scale.value < 3) {
    scale.value += 0.2;
    await renderPage();
    if (fullscreen.value) {
      await renderFullscreenPage();
    }
  }
};

const zoomOut = async () => {
  if (scale.value > 0.5) {
    scale.value -= 0.2;
    await renderPage();
    if (fullscreen.value) {
      await renderFullscreenPage();
    }
  }
};

const toggleFullscreen = async () => {
  fullscreen.value = !fullscreen.value;
  if (fullscreen.value) {
    await nextTick();
    await renderFullscreenPage();
  }
};

const downloadPdf = async () => {
  try {
    logger.info('[PdfViewer]', `开始下载PDF | resourceId=${props.resourceId}`);
    await downloadResource(props.resourceId, undefined, {
      useCache: true,
      resourceDetail:
        props.resourceTitle && props.resourceType
          ? {
              title: props.resourceTitle,
              resourceType: props.resourceType,
            }
          : undefined,
    });
    ElMessage.success('已开始下载');
  } catch (err: any) {
    logger.error('[PdfViewer]', 'PDF下载失败', err);
    ElMessage.error('下载失败，请稍后重试');
  }
};

// 前往检测页面
const goToChallenge = () => {
  router.push('/pdf-preview-challenge');
};

// 初始化：检查是否应该自动加载
const initialize = () => {
  isFileTooLarge.value = false;
  if (shouldAutoLoad()) {
    // 检查文件是否超过阈值
    if (isOverThreshold()) {
      loading.value = false;
      isFileTooLarge.value = true;
      showLazyLoadPrompt.value = false;
      logger.info(
        '[PdfViewer]',
        `文件大小超过阈值: ${formatFileSize(props.fileSize || 0)} > ${formatFileSize(getSizeThreshold())}`
      );
    } else {
      loadPdf();
    }
  } else {
    loading.value = false;
    showLazyLoadPrompt.value = true;
  }
};

// 监听resourceId变化
watch(
  () => props.resourceId,
  () => {
    initialize();
  },
  { immediate: true }
);
</script>

<style scoped>
.pdf-viewer {
  width: 100%;
}

.loading-container,
.error-container,
.lazy-load-container {
  padding: 40px 20px;
  text-align: center;
}

.error-container,
.lazy-load-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
}

.error-message,
.lazy-load-message {
  margin-top: 16px;
}

.error-line,
.lazy-load-desc {
  margin: 8px 0;
  font-size: 14px;
  color: var(--el-text-color-regular);
  line-height: 1.6;
}

.lazy-load-title {
  margin: 8px 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.error-line strong {
  color: var(--el-color-danger);
  font-weight: 600;
}

.error-actions,
.lazy-load-actions {
  display: flex;
  gap: 12px;
  margin-top: 8px;
  flex-wrap: wrap;
  justify-content: center;
}

.loading-text {
  margin-top: 16px;
  color: var(--el-text-color-secondary);
}

.pdf-container {
  background-color: var(--el-fill-color-light);
  border-radius: 8px;
  overflow: hidden;
}

.pdf-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background-color: var(--el-bg-color);
  border-bottom: 1px solid var(--el-border-color);
}

.toolbar-left,
.toolbar-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.page-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.page-input {
  width: 70px;
}

.page-total {
  color: var(--el-text-color-secondary);
}

.zoom-info {
  font-size: 14px;
  color: var(--el-text-color-regular);
  min-width: 50px;
  text-align: center;
}

.pdf-content {
  padding: 24px;
  display: flex;
  justify-content: center;
  min-height: 400px;
  max-height: 600px;
  overflow: auto;
}

.pdf-canvas {
  background-color: white;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

/* 全屏样式 */
.fullscreen-toolbar {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 12px;
  background-color: var(--el-fill-color-light);
  border-radius: 8px;
  margin-bottom: 16px;
}

.fullscreen-content {
  display: flex;
  justify-content: center;
  min-height: 500px;
  max-height: 70vh;
  overflow: auto;
  background-color: var(--el-fill-color-light);
  border-radius: 8px;
  padding: 24px;
}

.fullscreen-canvas {
  background-color: white;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}
</style>
