<template>
  <el-dialog
    v-model="visible"
    title="用户指南"
    width="560px"
    :close-on-click-modal="false"
    :close-on-press-escape="true"
    class="user-guide-modal"
    align-center
    @closed="emit('closed')"
  >
    <div class="guide-content">
      <div class="guide-items">
        <div class="guide-item">
          <div class="item-number">1</div>
          <div class="item-text">
            <span>未登录用户可以查看和下载</span>
            <span class="highlight-text">所有资源</span>
            <span>，登录用户可创建收藏夹，</span>
            <span class="highlight-text">一键打包下载</span>
          </div>
        </div>

        <div class="guide-item">
          <div class="item-number">2</div>
          <div class="item-text">
            <span>欢迎</span>
            <span class="highlight-text">上传资源</span>
            <span>，或为资源</span>
            <span class="highlight-text">打分</span>
            <span>（均需要</span>
            <span class="highlight-text">注册</span>
            <span>）</span>
          </div>
        </div>

        <div class="guide-item">
          <div class="item-number">3</div>
          <div class="item-text">
            <span class="highlight-text">强烈建议</span>
            <span>使用</span>
            <span class="highlight-text">电脑</span>
            <span>的</span>
            <span class="highlight-text red-text">Chrome, Edge</span>
            <span>浏览器访问，手机端UI正在适配中</span>
          </div>
        </div>

        <div class="guide-item">
          <div class="item-number">4</div>
          <div class="item-text">
            <span>所有资源均为</span>
            <span class="highlight-text">无偿</span>
            <span>分享，禁止用于任何形式的盈利活动</span>
          </div>
        </div>
      </div>

      <div class="guide-footer-hint">
        <el-icon><CircleCheck /></el-icon>
        <span>注册简单快捷，无需邮箱验证</span>
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
import { ref } from 'vue';
import { CircleCheck } from '@element-plus/icons-vue';
import { ElMessage } from 'element-plus';
import logger from '@/utils/logger';

// 关闭后通知父组件（用于串联下一个引导弹窗）
const emit = defineEmits<{ closed: [] }>();

// 状态
const visible = ref(false);
const dontShowAgain = ref(false);

// LocalStorage 键名
const GUIDE_MODAL_KEY = 'userGuideModalClosed';

// 检查是否应该显示弹窗
function shouldShowModal(): boolean {
  try {
    const stored = localStorage.getItem(GUIDE_MODAL_KEY);
    if (stored) {
      const data = JSON.parse(stored);
      // 检查是否是永久关闭
      if (data.permanent === true) {
        return false;
      }
    }
  } catch (e) {
    // 如果解析失败，默认显示
    logger.warn('[UserGuideModal]', 'Failed to parse user guide modal setting:', e);
  }
  return true;
}

// 显示弹窗；返回本次是否真正打开（供父组件决定是否直接串联下一个弹窗）
function show(): boolean {
  if (shouldShowModal()) {
    visible.value = true;
    return true;
  }
  return false;
}

// 关闭弹窗
function handleClose() {
  // 如果勾选了"不再显示"，则保存到 localStorage
  if (dontShowAgain.value) {
    try {
      localStorage.setItem(
        GUIDE_MODAL_KEY,
        JSON.stringify({
          permanent: true,
          timestamp: Date.now(),
        })
      );
      ElMessage.success('已永久关闭用户指南弹窗，可在设置中重新开启');
    } catch (e) {
      logger.error('[UserGuideModal]', 'Failed to save user guide modal setting:', e);
    }
  }
  visible.value = false;
}

// 获取当前设置状态（供设置页面使用）
function isPermanentlyClosed(): boolean {
  try {
    const stored = localStorage.getItem(GUIDE_MODAL_KEY);
    if (stored) {
      const data = JSON.parse(stored);
      return data.permanent === true;
    }
  } catch (e) {
    logger.warn('[UserGuideModal]', 'Failed to parse user guide modal setting:', e);
  }
  return false;
}

// 设置永久关闭状态（供设置页面使用）
function setPermanentlyClosed(closed: boolean): void {
  try {
    if (closed) {
      localStorage.setItem(
        GUIDE_MODAL_KEY,
        JSON.stringify({
          permanent: true,
          timestamp: Date.now(),
        })
      );
    } else {
      // 清除永久关闭设置，下次进入资源页（首页）会显示
      localStorage.removeItem(GUIDE_MODAL_KEY);
    }
  } catch (e) {
    logger.error('[UserGuideModal]', 'Failed to save user guide modal setting:', e);
  }
}

// 是否自动弹出由父组件（资源页）编排，本组件不再自行 onMounted 触发

// 暴露方法给父组件
defineExpose({
  show,
  isPermanentlyClosed,
  setPermanentlyClosed,
});
</script>

<style scoped>
.user-guide-modal :deep(.el-dialog__header) {
  text-align: center;
  padding: 24px 20px 16px;
  border-bottom: 1px solid var(--el-border-color-light);
}

.user-guide-modal :deep(.el-dialog__title) {
  font-size: 20px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.user-guide-modal :deep(.el-dialog__body) {
  padding: 24px 28px;
}

.user-guide-modal :deep(.el-dialog__footer) {
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
  background-color: var(--el-fill-color-light);
  border-radius: var(--su-radius-md);
}

.item-number {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, var(--el-color-primary) 0%, #66b1ff 100%);
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
  color: var(--el-color-primary);
  font-weight: 600;
}

.red-text {
  color: var(--el-color-danger);
}

.guide-footer-hint {
  margin-top: 15px;
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 15px;
  background-color: var(--el-color-success-light-9);
  border-radius: var(--su-radius-lg);
  color: var(--el-color-success);
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
