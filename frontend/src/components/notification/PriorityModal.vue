<template>
  <el-dialog
    v-model="visible"
    title="重要通知"
    width="500px"
    :close-on-click-modal="false"
    :close-on-press-escape="false"
    :show-close="false"
    class="priority-modal"
    align-center
  >
    <div v-if="currentNotification" class="priority-content">
      <div class="priority-icon">
        <el-icon :size="48" color="#f56c6c">
          <WarningFilled />
        </el-icon>
      </div>

      <h3 class="priority-title">{{ currentNotification.title }}</h3>
      <p class="priority-text">{{ currentNotification.content }}</p>

      <div v-if="currentNotification.linkUrl" class="priority-link">
        <el-link type="primary" @click="handleLinkClick">
          点击查看详情
        </el-link>
      </div>

      <div class="priority-meta">
        <span>剩余 {{ pendingCount }} 条未读通知</span>
        <span class="priority-time">{{ formatTime(currentNotification.createdAt) }}</span>
      </div>
    </div>

    <template #footer>
      <div class="priority-footer">
        <el-button
          v-if="pendingCount > 1"
          @click="handleDismissAndNext"
          :loading="dismissing"
        >
          下一条
        </el-button>
        <el-button type="primary" @click="handleDismiss" :loading="dismissing">
          {{ pendingCount > 1 ? '我知道了' : '关闭' }}
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useRouter } from 'vue-router';
import { WarningFilled } from '@element-plus/icons-vue';
import { useNotificationStore } from '../../stores/notification';
import type { Notification } from '../../types/notification';
import { ElMessage } from 'element-plus';

const router = useRouter();
const notificationStore = useNotificationStore();

// 状态
const visible = ref(false);
const dismissing = ref(false);
const currentNotification = ref<Notification | null>(null);

// 计算属性
const pendingNotifications = computed(() => notificationStore.priorityNotifications);
const pendingCount = computed(() => pendingNotifications.value.length);

// 显示下一条通知
function showNext() {
  if (pendingNotifications.value.length > 0) {
    const nextNotification = pendingNotifications.value[0];
    if (nextNotification) {
      currentNotification.value = nextNotification;
      visible.value = true;
      return;
    }
  }
  currentNotification.value = null;
  visible.value = false;
}

// 公开方法：检查并显示高优先级通知
async function checkAndShowPriorityNotifications() {
  await notificationStore.fetchPriorityNotifications();
  if (pendingCount.value > 0 && !visible.value) {
    showNext();
  }
}

// 关闭当前通知
async function handleDismiss() {
  if (!currentNotification.value) return;

  dismissing.value = true;
  try {
    await notificationStore.dismissPriority(currentNotification.value.id);
    showNext();
  } catch (error: any) {
    if (!error.isHandled) {
      ElMessage.error('操作失败');
    }
  } finally {
    dismissing.value = false;
  }
}

// 关闭并显示下一条
async function handleDismissAndNext() {
  if (!currentNotification.value) return;

  dismissing.value = true;
  try {
    await notificationStore.dismissPriority(currentNotification.value.id);
    // 短暂延迟后显示下一条，让用户有视觉反馈
    setTimeout(() => {
      showNext();
      dismissing.value = false;
    }, 200);
  } catch (error: any) {
    if (!error.isHandled) {
      ElMessage.error('操作失败');
    }
    dismissing.value = false;
  }
}

// 点击链接
function handleLinkClick() {
  if (currentNotification.value?.linkUrl) {
    // 先标记为已读
    notificationStore.dismissPriority(currentNotification.value.id);
    visible.value = false;
    // 然后跳转
    router.push(currentNotification.value.linkUrl);
  }
}

// 格式化时间
function formatTime(time: string): string {
  const utcTimeString = time.endsWith('Z') ? time : `${time}Z`;
  const date = new Date(utcTimeString);
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  });
}

// 暴露方法给父组件
defineExpose({
  checkAndShowPriorityNotifications
});
</script>

<style scoped>
.priority-modal :deep(.el-dialog__header) {
  text-align: center;
  padding: 20px;
  border-bottom: 1px solid var(--el-border-color-light);
}

.priority-modal :deep(.el-dialog__title) {
  font-size: 18px;
  font-weight: 600;
  color: var(--el-color-danger);
}

.priority-modal :deep(.el-dialog__body) {
  padding: 24px;
}

.priority-modal :deep(.el-dialog__footer) {
  border-top: 1px solid var(--el-border-color-light);
  padding: 16px 20px;
}

.priority-content {
  text-align: center;
}

.priority-icon {
  margin-bottom: 16px;
}

.priority-title {
  margin: 0 0 16px 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  line-height: 1.4;
}

.priority-text {
  margin: 0 0 20px 0;
  font-size: 15px;
  color: var(--el-text-color-regular);
  line-height: 1.6;
  white-space: pre-wrap;
  text-align: left;
}

.priority-link {
  margin-bottom: 16px;
}

.priority-meta {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
  color: var(--el-text-color-placeholder);
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px dashed var(--el-border-color-lighter);
}

.priority-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}
</style>
