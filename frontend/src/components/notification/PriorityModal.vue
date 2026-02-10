<template>
  <el-dialog
    v-model="visible"
    title="重要通知"
    width="600px"
    :close-on-click-modal="false"
    :close-on-press-escape="false"
    :show-close="false"
    class="priority-modal"
    align-center
  >
    <div class="priority-notifications">
      <div
        v-for="notification in priorityNotifications"
        :key="notification.id"
        class="priority-item"
      >
        <div class="priority-icon">
          <el-icon size="32" color="#f56c6c">
            <Warning />
          </el-icon>
        </div>

        <div class="priority-content">
          <h3 class="priority-title">{{ notification.title }}</h3>
          <p class="priority-text">{{ notification.content }}</p>
          <div class="priority-time">{{ formatTime(notification.createdAt) }}</div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="priority-footer">
        <el-button
          v-if="priorityNotifications.length > 1"
          @click="handleDismissAll"
          :loading="dismissing"
        >
          全部关闭
        </el-button>
        <el-button type="primary" @click="handleDismissCurrent" :loading="dismissing">
          我知道了
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { Warning } from '@element-plus/icons-vue';
import { useNotificationStore } from '../../stores/notification';
import { useAuthStore } from '../../stores/auth';
import type { Notification } from '../../types/notification';
import { ElMessage } from 'element-plus';

const notificationStore = useNotificationStore();
const authStore = useAuthStore();

// 状态
const visible = ref(false);
const dismissing = ref(false);
const currentIndex = ref(0);

// 计算属性
const priorityNotifications = computed(() => notificationStore.priorityNotifications);
const currentNotification = computed(() => priorityNotifications.value[currentIndex.value]);
const hasUnreadPriority = computed(() => priorityNotifications.value.length > 0);

// 方法
async function checkPriorityNotifications() {
  await notificationStore.fetchPriorityNotifications();
  if (hasUnreadPriority.value) {
    visible.value = true;
    currentIndex.value = 0;
  }
}

async function handleDismissCurrent() {
  if (!currentNotification.value) return;

  dismissing.value = true;
  try {
    await notificationStore.dismissPriority(currentNotification.value.id);

    // 如果还有更多，显示下一个
    if (currentIndex.value < priorityNotifications.value.length - 1) {
      currentIndex.value++;
    } else if (priorityNotifications.value.length === 0) {
      // 全部关闭了
      visible.value = false;
    }
  } catch (error: any) {
    if (!error.isHandled) {
      ElMessage.error('操作失败');
    }
  } finally {
    dismissing.value = false;
  }
}

async function handleDismissAll() {
  dismissing.value = true;
  try {
    // 关闭所有高优先级通知
    const ids = priorityNotifications.value.map((n: Notification) => n.id);
    for (const id of ids) {
      await notificationStore.dismissPriority(id);
    }
    visible.value = false;
  } catch (error: any) {
    if (!error.isHandled) {
      ElMessage.error('操作失败');
    }
  } finally {
    dismissing.value = false;
  }
}

// 格式化时间
function formatTime(time: string): string {
  // 将无时区的时间字符串视为 UTC 时间
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

// 生命周期
onMounted(() => {
  // 延迟检查，等待认证状态初始化完成
  setTimeout(() => {
    // 只有登录用户才检查通知
    if (authStore.isAuthenticated) {
      checkPriorityNotifications();
    }
  }, 1000);
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
  padding: 0;
  max-height: 400px;
  overflow-y: auto;
}

.priority-modal :deep(.el-dialog__footer) {
  border-top: 1px solid var(--el-border-color-light);
  padding: 16px 20px;
}

.priority-notifications {
  padding: 20px;
}

.priority-item {
  display: flex;
  gap: 16px;
  padding: 16px;
  background-color: var(--el-color-danger-light-9);
  border: 1px solid var(--el-color-danger-light-7);
  border-radius: 8px;
}

.priority-icon {
  display: flex;
  align-items: flex-start;
  padding-top: 4px;
}

.priority-content {
  flex: 1;
  min-width: 0;
}

.priority-title {
  margin: 0 0 8px 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.priority-text {
  margin: 0 0 12px 0;
  font-size: 14px;
  color: var(--el-text-color-regular);
  line-height: 1.6;
  white-space: pre-wrap;
}

.priority-time {
  font-size: 12px;
  color: var(--el-text-color-placeholder);
}

.priority-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}
</style>
