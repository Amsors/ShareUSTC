<template>
  <el-dialog
    v-model="visible"
    title="资源页面使用指南"
    width="560px"
    :close-on-click-modal="false"
    :close-on-press-escape="true"
    class="resource-guide-modal"
    align-center
  >
    <div class="guide-content">
      <div class="guide-items">
        <div class="guide-item">
          <div class="item-number">1</div>
          <div class="item-text">
            <span>可输入</span>
            <span class="highlight-text">课程/教师名</span>
            <span>进行搜索，支持多选</span>
            <!-- <span class="highlight-text red-text">这样是红色</span> -->
          </div>
        </div>

        <div class="guide-item">
          <div class="item-number">2</div>
          <div class="item-text">
            <span>数据来源于教务处公布的近三个学期的课程信息(26春、25秋、25夏)</span>
          </div>
        </div>

        <div class="guide-item">
          <div class="item-number">3</div>
          <div class="item-text">
            <span class="highlight-text">登录</span>
            <span>后可在此页面将资源</span>
            <span class="highlight-text">一键加入</span>
            <span>收藏夹</span>
          </div>
        </div>

        <div class="guide-item">
          <div class="item-number">4</div>
          <div class="item-text">
            <span>使用</span>
            <span class="highlight-text">鼠标中键</span>
            <span>可以在新页面打开资源详情，不会丢失搜索词条</span>
            <!-- <span class="highlight-text red-text">这样是红色</span> -->
          </div>
        </div>

        <div class="guide-item">
          <div class="item-number">5</div>
          <div class="item-text">
            <span>部分资源</span>
            <span class="highlight-text">课程划分可能有误</span>
            <span
              >，请勾选所有相关课程。如选择 数学分析 课程资料时，可以同时勾选
              “数学分析【未指定】”、“数学分析(B1)” 等多个选项</span
            >
            <!-- <span class="highlight-text red-text">这样是红色</span> -->
          </div>
        </div>
      </div>

      <div class="guide-footer-hint">
        <el-icon><Collection /></el-icon>
        <span>创建收藏夹，高效管理你的学习资源</span>
      </div>
    </div>

    <template #footer>
      <div class="guide-footer">
        <div class="footer-left">
          <el-checkbox v-model="dontShowAgain" size="small"> 不再显示 </el-checkbox>
        </div>
        <div class="footer-right">
          <el-button type="primary" size="default" @click="handleClose"> 我知道了 </el-button>
        </div>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { Collection } from '@element-plus/icons-vue';
import { ElMessage } from 'element-plus';
import logger from '../../utils/logger';

// 状态
const visible = ref(false);
const dontShowAgain = ref(false);

// LocalStorage 键名
const RESOURCE_GUIDE_MODAL_KEY = 'resourceGuideModalClosed';

// 检查是否应该显示弹窗
function shouldShowModal(): boolean {
  try {
    const stored = localStorage.getItem(RESOURCE_GUIDE_MODAL_KEY);
    if (stored) {
      const data = JSON.parse(stored);
      // 检查是否是永久关闭
      if (data.permanent === true) {
        return false;
      }
    }
  } catch (e) {
    // 如果解析失败，默认显示
    logger.warn('[ResourceGuideModal]', 'Failed to parse resource guide modal setting:', e);
  }
  return true;
}

// 显示弹窗
function show() {
  if (shouldShowModal()) {
    visible.value = true;
  }
}

// 关闭弹窗
function handleClose() {
  // 如果勾选了"不再显示"，则保存到 localStorage
  if (dontShowAgain.value) {
    try {
      localStorage.setItem(
        RESOURCE_GUIDE_MODAL_KEY,
        JSON.stringify({
          permanent: true,
          timestamp: Date.now(),
        })
      );
      ElMessage.success('已永久关闭资源页面指南弹窗，可在设置中重新开启');
    } catch (e) {
      logger.error('[ResourceGuideModal]', 'Failed to save resource guide modal setting:', e);
    }
  }
  visible.value = false;
}

// 获取当前设置状态（供设置页面使用）
function isPermanentlyClosed(): boolean {
  try {
    const stored = localStorage.getItem(RESOURCE_GUIDE_MODAL_KEY);
    if (stored) {
      const data = JSON.parse(stored);
      return data.permanent === true;
    }
  } catch (e) {
    logger.warn('[ResourceGuideModal]', 'Failed to parse resource guide modal setting:', e);
  }
  return false;
}

// 设置永久关闭状态（供设置页面使用）
function setPermanentlyClosed(closed: boolean): void {
  try {
    if (closed) {
      localStorage.setItem(
        RESOURCE_GUIDE_MODAL_KEY,
        JSON.stringify({
          permanent: true,
          timestamp: Date.now(),
        })
      );
    } else {
      // 清除永久关闭设置，下次进入资源页面会显示
      localStorage.removeItem(RESOURCE_GUIDE_MODAL_KEY);
    }
  } catch (e) {
    logger.error('[ResourceGuideModal]', 'Failed to save resource guide modal setting:', e);
  }
}

// 页面加载时检查是否显示
onMounted(() => {
  // 延迟一点显示，让页面先加载完成
  setTimeout(() => {
    show();
  }, 500);
});

// 暴露方法给父组件
defineExpose({
  show,
  isPermanentlyClosed,
  setPermanentlyClosed,
});
</script>

<style scoped>
.resource-guide-modal :deep(.el-dialog__header) {
  text-align: center;
  padding: 24px 20px 16px;
  border-bottom: 1px solid var(--el-border-color-light);
}

.resource-guide-modal :deep(.el-dialog__title) {
  font-size: 20px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.resource-guide-modal :deep(.el-dialog__body) {
  padding: 24px 28px;
}

.resource-guide-modal :deep(.el-dialog__footer) {
  border-top: 1px solid var(--el-border-color-light);
  padding: 16px 24px;
}

.guide-content {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.guide-items {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.guide-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 8px;
  background-color: #f5f7fa;
  border-radius: 10px;
  transition: all 0.3s ease;
}

.guide-item:hover {
  background-color: #ecf5ff;
  transform: translateX(4px);
}

.item-number {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #409eff 0%, #66b1ff 100%);
  color: #fff;
  font-size: 14px;
  font-weight: 600;
  border-radius: 50%;
  flex-shrink: 0;
}

.item-text {
  flex: 1;
  font-size: 14px;
  line-height: 1.7;
  color: var(--el-text-color-regular);
}

.highlight-text {
  color: #409eff;
  font-weight: 600;
}

.red-text {
  color: #f56c6c;
}

.guide-footer-hint {
  margin-top: 15px;
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 15px;
  background-color: #ecf5ff;
  border-radius: 20px;
  color: #409eff;
  font-size: 13px;
}

.guide-footer-hint .el-icon {
  font-size: 16px;
}

.guide-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

.footer-left {
  display: flex;
  align-items: center;
}

.footer-left :deep(.el-checkbox__label) {
  font-size: 16px;
  color: var(--el-text-color-secondary);
}

.footer-right {
  display: flex;
  gap: 10px;
}
</style>
