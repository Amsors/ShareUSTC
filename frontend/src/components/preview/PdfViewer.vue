<template>
  <div class="pdf-viewer">
    <div v-if="loading" class="loading-container">
      <el-skeleton :rows="10" animated />
      <p class="loading-text">正在加载 PDF...</p>
    </div>

    <div v-else-if="error" class="error-container">
      <el-empty description="PDF 加载失败" />
      <el-button type="primary" @click="loadPdf">重试</el-button>
    </div>

    <div v-else class="pdf-container">
      <!-- 工具栏 -->
      <div class="pdf-toolbar">
        <div class="toolbar-left">
          <el-button circle size="small" @click="prevPage" :disabled="currentPage <= 1">
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
          <el-button circle size="small" @click="nextPage" :disabled="currentPage >= totalPages">
            <el-icon><ArrowRight /></el-icon>
          </el-button>
        </div>

        <div class="toolbar-right">
          <el-button circle size="small" @click="zoomOut" :disabled="scale <= 0.5">
            <el-icon><ZoomOut /></el-icon>
          </el-button>
          <span class="zoom-info">{{ Math.round(scale * 100) }}%</span>
          <el-button circle size="small" @click="zoomIn" :disabled="scale >= 3">
            <el-icon><ZoomIn /></el-icon>
          </el-button>
          <el-button circle size="small" @click="toggleFullscreen">
            <el-icon><FullScreen /></el-icon>
          </el-button>
        </div>
      </div>

      <!-- PDF 渲染区域 -->
      <div class="pdf-content" ref="pdfContentRef">
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
        <el-button circle size="small" @click="prevPage" :disabled="currentPage <= 1">
          <el-icon><ArrowLeft /></el-icon>
        </el-button>
        <span class="page-info">{{ currentPage }} / {{ totalPages }}</span>
        <el-button circle size="small" @click="nextPage" :disabled="currentPage >= totalPages">
          <el-icon><ArrowRight /></el-icon>
        </el-button>
        <el-divider direction="vertical" />
        <el-button circle size="small" @click="zoomOut" :disabled="scale <= 0.5">
          <el-icon><ZoomOut /></el-icon>
        </el-button>
        <span class="zoom-info">{{ Math.round(scale * 100) }}%</span>
        <el-button circle size="small" @click="zoomIn" :disabled="scale >= 3">
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
import { ref, watch, nextTick } from 'vue';
import { ArrowLeft, ArrowRight, ZoomIn, ZoomOut, FullScreen } from '@element-plus/icons-vue';
import * as pdfjsLib from 'pdfjs-dist';
import { getResourceContent } from '../../api/resource';

// 设置 PDF.js worker
// 使用 CDN 版本的 worker
pdfjsLib.GlobalWorkerOptions.workerSrc = `https://cdnjs.cloudflare.com/ajax/libs/pdf.js/${pdfjsLib.version}/pdf.worker.min.mjs`;

const props = defineProps<{
  resourceId: string;
}>();

const loading = ref(true);
const error = ref(false);
const currentPage = ref(1);
const totalPages = ref(0);
const scale = ref(1.2);
const fullscreen = ref(false);
// 使用普通变量而非 ref，避免 Vue 响应式代理破坏 PDF.js 内部私有成员
let pdfDoc: any = null;
const canvasRef = ref<HTMLCanvasElement | null>(null);
const fullscreenCanvasRef = ref<HTMLCanvasElement | null>(null);

const loadPdf = async () => {
  loading.value = true;
  error.value = false;
  try {
    console.log('[PdfViewer] 开始加载PDF, resourceId:', props.resourceId);
    const blob = await getResourceContent(props.resourceId);
    console.log('[PdfViewer] 获取到blob:', blob.type, blob.size);

    // 确保blob类型正确
    let pdfBlob = blob;
    if (!blob.type || blob.type === 'application/octet-stream') {
      console.log('[PdfViewer] Blob类型不正确，强制设置为application/pdf');
      pdfBlob = new Blob([blob], { type: 'application/pdf' });
    }

    const arrayBuffer = await pdfBlob.arrayBuffer();
    console.log('[PdfViewer] 转换为ArrayBuffer, 大小:', arrayBuffer.byteLength);

    // 检查PDF.js worker是否配置正确
    console.log('[PdfViewer] PDF.js version:', pdfjsLib.version);
    console.log('[PdfViewer] Worker src:', pdfjsLib.GlobalWorkerOptions.workerSrc);

    const loadingTask = pdfjsLib.getDocument({
      data: arrayBuffer,
      useSystemFonts: true,  // 使用系统字体支持中文显示
      cMapUrl: 'https://cdnjs.cloudflare.com/ajax/libs/pdf.js/' + pdfjsLib.version + '/cmaps/',
      cMapPacked: true,
      disableFontFace: false,  // 启用字体face以更好地支持嵌入式字体
      fontExtraProperties: true,  // 保留额外字体属性
    });
    loadingTask.onProgress = (progress: any) => {
      console.log('[PdfViewer] 加载进度:', progress);
    };

    pdfDoc = await loadingTask.promise;
    console.log('[PdfViewer] PDF文档加载成功, 页数:', pdfDoc.numPages);

    totalPages.value = pdfDoc.numPages;
    currentPage.value = 1;
    loading.value = false;
    // 等待 DOM 更新后再渲染，确保 canvasRef 已存在
    await nextTick();
    await renderPage();
  } catch (err: any) {
    console.error('[PdfViewer] PDF加载失败:', err);
    console.error('[PdfViewer] 错误详情:', err.message, err.stack);
    error.value = true;
    loading.value = false;
  }
};

const renderPage = async () => {
  if (!pdfDoc || !canvasRef.value) {
    console.warn('[PdfViewer] renderPage: pdfDoc 或 canvasRef 不存在');
    return;
  }

  console.log('[PdfViewer] 开始渲染页面:', currentPage.value, '缩放:', scale.value);

  try {
    const page = await pdfDoc.getPage(currentPage.value);
    console.log('[PdfViewer] 获取页面成功, 页面尺寸:', page.view);

    // 获取页面文本内容信息（用于调试字体问题）
    try {
      const textContent = await page.getTextContent();
      const fontNames = new Set<string>();
      textContent.items.forEach((item: any) => {
        if (item.fontName) {
          fontNames.add(item.fontName);
        }
      });
      console.log('[PdfViewer] 页面使用的字体:', Array.from(fontNames));
    } catch (e) {
      console.log('[PdfViewer] 无法获取文本内容:', e);
    }

    const canvas = canvasRef.value;
    const context = canvas.getContext('2d');
    if (!context) {
      console.error('[PdfViewer] 无法获取 canvas context');
      return;
    }

    const viewport = page.getViewport({ scale: scale.value });
    console.log('[PdfViewer] viewport 尺寸:', viewport.width, 'x', viewport.height);

    // 设置 canvas 的实际像素尺寸
    canvas.height = viewport.height;
    canvas.width = viewport.width;

    // 设置 canvas 的显示尺寸，保持正确的宽高比
    canvas.style.height = `${viewport.height}px`;
    canvas.style.width = `${viewport.width}px`;

    console.log('[PdfViewer] canvas 尺寸设置:', canvas.width, 'x', canvas.height);
    console.log('[PdfViewer] canvas 样式尺寸:', canvas.style.width, 'x', canvas.style.height);

    const renderContext = {
      canvasContext: context,
      viewport: viewport,
      // 启用背景填充，避免透明背景导致的渲染问题
      background: 'white',
    };

    console.log('[PdfViewer] 开始 render 操作');
    const renderTask = page.render(renderContext);
    await renderTask.promise;
    console.log('[PdfViewer] 渲染完成');
  } catch (err: any) {
    console.error('[PdfViewer] 渲染页面失败:', err);
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

// 监听resourceId变化
watch(() => props.resourceId, () => {
  loadPdf();
}, { immediate: true });
</script>

<style scoped>
.pdf-viewer {
  width: 100%;
}

.loading-container,
.error-container {
  padding: 40px 0;
  text-align: center;
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
