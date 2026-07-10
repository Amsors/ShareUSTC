<template>
  <div class="settings-page">
    <div class="settings-container">
      <h1 class="page-title">设置</h1>

      <div class="settings-sections">
        <!-- 界面设置 -->
        <section class="settings-section">
          <h2 class="section-title">
            <el-icon><Setting /></el-icon>
            界面设置
          </h2>

          <div class="setting-item">
            <div class="setting-info">
              <div class="setting-label">首页用户指南弹窗</div>
              <div class="setting-desc">进入首页时显示用户指南</div>
            </div>
            <div class="setting-control">
              <el-switch
                v-model="showUserGuide"
                active-text="显示"
                inactive-text="隐藏"
                @change="handleUserGuideChange"
              />
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <div class="setting-label">资源页面指南弹窗</div>
              <div class="setting-desc">进入资源列表页面时显示使用指南</div>
            </div>
            <div class="setting-control">
              <el-switch
                v-model="showResourceGuide"
                active-text="显示"
                inactive-text="隐藏"
                @change="handleResourceGuideChange"
              />
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <div class="setting-label">自动加载 PDF 资料预览</div>
              <div class="setting-desc">
                <template v-if="pdfPreviewVerified">
                  通过检测后，可自动加载 PDF 预览以提升体验
                </template>
                <template v-else> 需要通过检测才能启用此功能 </template>
              </div>
            </div>
            <div class="setting-control">
              <el-switch
                v-model="autoLoadPdfPreview"
                :disabled="!pdfPreviewVerified"
                active-text="开启"
                inactive-text="关闭"
                @change="handleAutoLoadPdfPreviewChange"
              />
              <div v-if="!pdfPreviewVerified" class="setting-hint">
                <el-link type="primary" @click="goToPdfPreviewChallenge"> 前往检测 </el-link>
              </div>
            </div>
          </div>

          <div
            v-if="pdfPreviewVerified && autoLoadPdfPreview"
            class="setting-item threshold-setting"
          >
            <div class="setting-info">
              <div class="setting-label">自动加载大小阈值</div>
              <div class="setting-desc">超过此大小的 PDF 不会自动加载，需要手动点击加载</div>
            </div>
            <div class="setting-control">
              <el-input-number
                v-model="pdfPreviewSizeThreshold"
                :min="1"
                :max="100"
                size="small"
                class="threshold-input"
                @change="handleThresholdChange"
              >
                <template #suffix>MB</template>
              </el-input-number>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <div class="setting-label">搜索页面过滤空资源</div>
              <div class="setting-desc">
                在搜索页面的课程/教师下拉菜单中，只显示已绑定资源的课程/教师
              </div>
            </div>
            <div class="setting-control">
              <el-switch
                v-model="filterEmptyResources"
                active-text="开启"
                inactive-text="关闭"
                @change="handleFilterEmptyResourcesChange"
              />
            </div>
          </div>
        </section>

        <!-- 缓存管理 -->
        <section class="settings-section">
          <h2 class="section-title">
            <el-icon><Collection /></el-icon>
            本地缓存管理
            <el-tag v-if="cacheStats.totalEntries > 0" type="info" size="small">
              {{ cacheStats.totalEntries }} 个文件
            </el-tag>
          </h2>

          <div class="cache-content">
            <!-- 缓存统计 -->
            <div class="cache-stats">
              <div class="stat-item">
                <div class="stat-value">{{ formatSize(cacheStats.totalSize) }}</div>
                <div class="stat-label">缓存占用</div>
              </div>
              <div class="stat-item">
                <div class="stat-value">{{ cacheStats.totalEntries }}</div>
                <div class="stat-label">缓存文件数</div>
              </div>
              <div class="stat-item">
                <div class="stat-value">{{ formatDuration(cacheAge) }}</div>
                <div class="stat-label">最旧缓存</div>
              </div>
            </div>

            <!-- 操作按钮 -->
            <div class="cache-actions">
              <el-button
                type="primary"
                :disabled="cacheStats.totalEntries === 0"
                :loading="clearingExpired"
                @click="handleClearExpired"
              >
                <el-icon><Timer /></el-icon>
                清理过期缓存
              </el-button>

              <el-button
                type="danger"
                plain
                :disabled="cacheStats.totalEntries === 0"
                :loading="clearingAll"
                @click="handleClearAll"
              >
                <el-icon><Delete /></el-icon>
                清空所有缓存
              </el-button>

              <el-button :loading="loading" @click="refreshStats">
                <el-icon><Refresh /></el-icon>
                刷新
              </el-button>
            </div>

            <!-- 缓存说明 -->
            <div class="cache-info">
              <el-alert type="info" :closable="false">
                <template #title>缓存说明</template>
                <ul>
                  <li>预览和下载的资源会自动缓存到本地浏览器</li>
                  <li>缓存有效期为 24 小时，过期后会自动清理</li>
                  <li>单个文件最大缓存 100MB，总缓存上限 1GB</li>
                  <li>缓存可以减少网络请求，加快资源访问速度</li>
                </ul>
              </el-alert>
            </div>
          </div>
        </section>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useRouter } from 'vue-router';
import { Collection, Timer, Delete, Refresh, Setting } from '@element-plus/icons-vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { resourceCache, type CacheStats } from '../utils/resourceCache';
import logger from '../utils/logger';

const router = useRouter();

// 用户指南弹窗 LocalStorage 键名
const GUIDE_MODAL_KEY = 'userGuideModalClosed';
const RESOURCE_GUIDE_MODAL_KEY = 'resourceGuideModalClosed';

// PDF 预览设置 LocalStorage 键名
const PDF_PREVIEW_VERIFIED_KEY = 'pdfPreviewVerified';
const PDF_PREVIEW_AUTO_LOAD_KEY = 'pdfPreviewAutoLoad';
const PDF_PREVIEW_USER_ENABLED_KEY = 'pdfPreviewUserEnabled';

// 缓存状态
const loading = ref(false);
const clearingExpired = ref(false);
const clearingAll = ref(false);
const cacheStats = ref<CacheStats>({
  totalEntries: 0,
  totalSize: 0,
  oldestEntry: 0,
});

const cacheAge = computed(() => {
  if (!cacheStats.value.oldestEntry) return 0;
  return Date.now() - cacheStats.value.oldestEntry;
});

// 格式化文件大小
const formatSize = (bytes: number): string => {
  if (bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB'];
  const k = 1024;
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + units[i];
};

// 格式化时长
const formatDuration = (ms: number): string => {
  if (ms === 0) return '-';
  const minutes = Math.floor(ms / 60000);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);

  if (days > 0) return `${days} 天`;
  if (hours > 0) return `${hours} 小时`;
  if (minutes > 0) return `${minutes} 分钟`;
  return '刚刚';
};

// 刷新统计信息
const refreshStats = async () => {
  loading.value = true;
  try {
    cacheStats.value = await resourceCache.getStats();
  } catch {
    ElMessage.error('获取缓存信息失败');
  } finally {
    loading.value = false;
  }
};

// 清理过期缓存
const handleClearExpired = async () => {
  clearingExpired.value = true;
  try {
    const count = await resourceCache.clearExpired();
    ElMessage.success(`已清理 ${count} 个过期缓存`);
    await refreshStats();
  } catch {
    ElMessage.error('清理失败');
  } finally {
    clearingExpired.value = false;
  }
};

// 清空所有缓存
const handleClearAll = async () => {
  try {
    await ElMessageBox.confirm(
      '确定要清空所有本地缓存吗？这将删除所有已缓存的资源文件，下次访问需要重新下载。',
      '确认清空',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning',
      }
    );

    clearingAll.value = true;
    await resourceCache.clearAll();
    ElMessage.success('已清空所有缓存');
    await refreshStats();
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('清空失败');
    }
  } finally {
    clearingAll.value = false;
  }
};

onMounted(() => {
  refreshStats();
  loadUserGuideSetting();
  loadResourceGuideSetting();
  loadPdfPreviewSettings();
  loadFilterEmptyResourcesSetting();
});

// 用户指南设置
const showUserGuide = ref(true);
const showResourceGuide = ref(true);

// 搜索页面过滤空资源设置 LocalStorage 键名
const FILTER_EMPTY_RESOURCES_KEY = 'filterEmptyResources';

// 搜索页面过滤空资源设置
const filterEmptyResources = ref(true);

// PDF 预览设置
const autoLoadPdfPreview = ref(false);
const pdfPreviewVerified = ref(false);
const pdfPreviewSizeThreshold = ref(5); // 默认 5MB

// 加载 PDF 预览设置
const loadPdfPreviewSettings = () => {
  try {
    // 检查是否已通过验证
    const verifiedData = localStorage.getItem(PDF_PREVIEW_VERIFIED_KEY);
    if (verifiedData) {
      const verified = JSON.parse(verifiedData);
      pdfPreviewVerified.value = verified.verified === true;
    } else {
      pdfPreviewVerified.value = false;
    }

    // 加载自动加载设置（默认关闭）
    const autoLoadData = localStorage.getItem(PDF_PREVIEW_AUTO_LOAD_KEY);
    if (autoLoadData) {
      const autoLoad = JSON.parse(autoLoadData);
      autoLoadPdfPreview.value = autoLoad.enabled === true;
    } else {
      autoLoadPdfPreview.value = false;
    }

    // 加载大小阈值设置（默认 5MB）
    const thresholdData = localStorage.getItem('pdfPreviewSizeThreshold');
    if (thresholdData) {
      const threshold = JSON.parse(thresholdData);
      pdfPreviewSizeThreshold.value = threshold.value || 5;
    } else {
      pdfPreviewSizeThreshold.value = 5;
    }
  } catch (e) {
    logger.warn('[Settings]', 'Failed to load PDF preview settings:', e);
    pdfPreviewVerified.value = false;
    autoLoadPdfPreview.value = false;
    pdfPreviewSizeThreshold.value = 5;
  }
};

// 处理 PDF 预览自动加载设置变化
const handleAutoLoadPdfPreviewChange = (value: boolean) => {
  try {
    localStorage.setItem(
      PDF_PREVIEW_AUTO_LOAD_KEY,
      JSON.stringify({
        enabled: value,
        timestamp: Date.now(),
      })
    );

    // 同时更新用户启用状态
    localStorage.setItem(
      PDF_PREVIEW_USER_ENABLED_KEY,
      JSON.stringify({
        enabled: true,
        timestamp: Date.now(),
      })
    );

    if (value) {
      ElMessage.success('已开启自动加载 PDF 预览');
    } else {
      ElMessage.success('已关闭自动加载 PDF 预览');
    }
  } catch (e) {
    logger.error('[Settings]', 'Failed to save PDF preview setting:', e);
    ElMessage.error('设置保存失败');
  }
};

// 前往 PDF 预览检测页面
const goToPdfPreviewChallenge = () => {
  router.push('/pdf-preview-challenge');
};

// 处理阈值变化
const handleThresholdChange = (value: number) => {
  try {
    localStorage.setItem(
      'pdfPreviewSizeThreshold',
      JSON.stringify({
        value: value,
        timestamp: Date.now(),
      })
    );
    ElMessage.success(`已设置自动加载阈值：${value}MB`);
  } catch (e) {
    logger.error('[Settings]', 'Failed to save threshold setting:', e);
    ElMessage.error('设置保存失败');
  }
};

// 加载用户指南设置
const loadUserGuideSetting = () => {
  try {
    const stored = localStorage.getItem(GUIDE_MODAL_KEY);
    if (stored) {
      const data = JSON.parse(stored);
      // 如果设置了永久关闭，则开关为 false（不显示）
      showUserGuide.value = !data.permanent;
    } else {
      // 没有设置时默认显示
      showUserGuide.value = true;
    }
  } catch (e) {
    logger.warn('[Settings]', 'Failed to parse user guide modal setting:', e);
    showUserGuide.value = true;
  }
};

// 加载资源页面指南设置
const loadResourceGuideSetting = () => {
  try {
    const stored = localStorage.getItem(RESOURCE_GUIDE_MODAL_KEY);
    if (stored) {
      const data = JSON.parse(stored);
      // 如果设置了永久关闭，则开关为 false（不显示）
      showResourceGuide.value = !data.permanent;
    } else {
      // 没有设置时默认显示
      showResourceGuide.value = true;
    }
  } catch (e) {
    logger.warn('[Settings]', 'Failed to parse resource guide modal setting:', e);
    showResourceGuide.value = true;
  }
};

// 处理用户指南设置变化
const handleUserGuideChange = (value: boolean) => {
  try {
    if (value) {
      // 开启显示：清除永久关闭设置
      localStorage.removeItem(GUIDE_MODAL_KEY);
      ElMessage.success('已开启首页用户指南弹窗');
    } else {
      // 关闭显示：设置永久关闭
      localStorage.setItem(
        GUIDE_MODAL_KEY,
        JSON.stringify({
          permanent: true,
          timestamp: Date.now(),
        })
      );
      ElMessage.success('已永久关闭首页用户指南弹窗');
    }
  } catch (e) {
    logger.error('[Settings]', 'Failed to save user guide modal setting:', e);
    ElMessage.error('设置保存失败');
  }
};

// 处理资源页面指南设置变化
const handleResourceGuideChange = (value: boolean) => {
  try {
    if (value) {
      // 开启显示：清除永久关闭设置
      localStorage.removeItem(RESOURCE_GUIDE_MODAL_KEY);
      ElMessage.success('已开启资源页面指南弹窗');
    } else {
      // 关闭显示：设置永久关闭
      localStorage.setItem(
        RESOURCE_GUIDE_MODAL_KEY,
        JSON.stringify({
          permanent: true,
          timestamp: Date.now(),
        })
      );
      ElMessage.success('已永久关闭资源页面指南弹窗');
    }
  } catch (e) {
    logger.error('[Settings]', 'Failed to save resource guide modal setting:', e);
    ElMessage.error('设置保存失败');
  }
};

// 加载搜索页面过滤空资源设置
const loadFilterEmptyResourcesSetting = () => {
  try {
    const stored = localStorage.getItem(FILTER_EMPTY_RESOURCES_KEY);
    if (stored) {
      const data = JSON.parse(stored);
      // 如果没有明确设置 enabled，默认为 true
      filterEmptyResources.value = data.enabled !== false;
    } else {
      // 没有设置时默认开启
      filterEmptyResources.value = true;
    }
  } catch (e) {
    logger.warn('[Settings]', 'Failed to parse filter empty resources setting:', e);
    filterEmptyResources.value = true;
  }
};

// 处理搜索页面过滤空资源设置变化
const handleFilterEmptyResourcesChange = (value: boolean) => {
  try {
    localStorage.setItem(
      FILTER_EMPTY_RESOURCES_KEY,
      JSON.stringify({
        enabled: value,
        timestamp: Date.now(),
      })
    );

    if (value) {
      ElMessage.success('已开启搜索页面过滤，只显示有资源的课程/教师');
    } else {
      ElMessage.success('已关闭搜索页面过滤，显示所有课程/教师');
    }
  } catch (e) {
    logger.error('[Settings]', 'Failed to save filter empty resources setting:', e);
    ElMessage.error('设置保存失败');
  }
};
</script>

<style scoped>
.settings-page {
  min-height: 100vh;
  background-color: #f5f7fa;
  padding: 40px 20px;
}

.settings-container {
  max-width: 800px;
  margin: 0 auto;
}

.page-title {
  margin: 0 0 32px;
  font-size: 28px;
  color: #303133;
  font-weight: 600;
}

.settings-sections {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.settings-section {
  background: #fff;
  border-radius: 12px;
  padding: 24px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.05);
}

.section-title {
  margin: 0 0 24px;
  font-size: 18px;
  color: #303133;
  display: flex;
  align-items: center;
  gap: 8px;
  padding-bottom: 16px;
  border-bottom: 1px solid #ebeef5;
}

.section-title .el-icon {
  color: #409eff;
}

.section-title .el-tag {
  margin-left: auto;
  font-weight: normal;
}

/* 设置项样式 */
.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 0;
  border-bottom: 1px solid #ebeef5;
}

.setting-item:last-child {
  border-bottom: none;
}

.setting-info {
  flex: 1;
  padding-right: 24px;
}

.setting-label {
  font-size: 15px;
  font-weight: 500;
  color: #303133;
  margin-bottom: 6px;
}

.setting-desc {
  font-size: 13px;
  color: #909399;
  line-height: 1.5;
}

.setting-control {
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 8px;
}

.setting-hint {
  font-size: 12px;
  color: #909399;
  margin-top: 4px;
}

/* 阈值设置样式 */
.threshold-setting {
  background-color: #f5f7fa;
  border-radius: 8px;
  margin-top: 8px;
  padding: 16px 20px;
}

.threshold-input {
  width: 120px;
}

/* 缓存内容区域 - 无内边框设计 */
.cache-content {
  padding: 0;
}

/* 缓存统计 */
.cache-stats {
  display: flex;
  gap: 24px;
  margin-bottom: 24px;
  padding: 20px;
  background-color: #f5f7fa;
  border-radius: 8px;
}

.stat-item {
  flex: 1;
  text-align: center;
}

.stat-value {
  font-size: 28px;
  font-weight: 600;
  color: #409eff;
  margin-bottom: 4px;
}

.stat-label {
  font-size: 13px;
  color: #909399;
}

/* 操作按钮 */
.cache-actions {
  display: flex;
  gap: 12px;
  margin-bottom: 20px;
  flex-wrap: wrap;
}

/* 缓存说明 */
.cache-info {
  margin-top: 8px;
}

.cache-info ul {
  margin: 8px 0 0;
  padding-left: 20px;
}

.cache-info li {
  margin: 4px 0;
  line-height: 1.6;
  color: #606266;
}

/* 响应式 */
@media (max-width: 768px) {
  .settings-page {
    padding: 20px 16px;
  }

  .page-title {
    font-size: 24px;
    margin-bottom: 20px;
  }

  .settings-section {
    padding: 16px;
  }

  .cache-stats {
    flex-direction: column;
    gap: 16px;
    padding: 16px;
  }

  .stat-value {
    font-size: 24px;
  }

  .cache-actions {
    flex-direction: column;
  }

  .cache-actions .el-button {
    width: 100%;
  }

  .setting-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 16px;
  }

  .setting-info {
    padding-right: 0;
  }

  .setting-control {
    width: 100%;
  }

  .setting-control :deep(.el-switch) {
    width: 100%;
  }
}
</style>
