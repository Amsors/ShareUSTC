<template>
  <div class="notification-bell">
    <el-dropdown trigger="click" @visible-change="onDropdownVisibleChange">
      <div class="bell-wrapper">
        <el-badge :value="unreadCount" :hidden="unreadCount === 0" :max="99">
          <el-icon :size="24" class="bell-icon">
            <Bell />
          </el-icon>
        </el-badge>
      </div>

      <template #dropdown>
        <el-dropdown-menu class="notification-dropdown">
          <div class="notification-header">
            <span class="title">通知</span>
            <el-button
              v-if="unreadCount > 0"
              link
              type="primary"
              size="small"
              @click="handleMarkAllRead"
            >
              全部已读
            </el-button>
          </div>

          <div class="notification-list" v-if="recentNotifications.length > 0">
            <el-dropdown-item
              v-for="notification in recentNotifications"
              :key="notification.id"
              class="notification-item"
              :class="{ unread: !notification.isRead }"
              @click="handleNotificationClick(notification)"
            >
              <div class="notification-content">
                <div class="notification-title">{{ notification.title }}</div>
                <div class="notification-text" v-if="notification.content">
                  {{ truncateText(notification.content, 50) }}
                </div>
                <div class="notification-time">{{ formatTime(notification.createdAt) }}</div>
              </div>
              <el-badge
                v-if="!notification.isRead"
                is-dot
                type="danger"
                class="unread-dot"
              />
            </el-dropdown-item>
          </div>

          <div v-else class="notification-empty">
            <el-icon :size="40" class="empty-icon">
              <Bell />
            </el-icon>
            <p>暂无新通知</p>
          </div>

          <div class="notification-footer">
            <el-button link type="primary" @click="goToNotificationCenter">
              查看全部
            </el-button>
          </div>
        </el-dropdown-menu>
      </template>
    </el-dropdown>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import { Bell } from '@element-plus/icons-vue';
import { useNotificationStore } from '../../stores/notification';
import { useAuthStore } from '../../stores/auth';
import type { Notification } from '../../types/notification';
import { ElMessage } from 'element-plus';

const router = useRouter();
const notificationStore = useNotificationStore();
const authStore = useAuthStore();

// 状态
const unreadCount = computed(() => notificationStore.unreadCount);
const recentNotifications = computed(() =>
  notificationStore.notifications.slice(0, 5)
);

// 轮询定时器
let pollTimer: ReturnType<typeof setInterval> | null = null;

// 下拉菜单显示状态改变
async function onDropdownVisibleChange(visible: boolean) {
  if (visible && authStore.isAuthenticated) {
    // 打开下拉菜单时刷新通知列表（仅登录用户）
    await notificationStore.fetchNotifications({ perPage: 5 });
  }
}

// 处理通知点击
async function handleNotificationClick(notification: Notification) {
  // 标记为已读
  if (!notification.isRead) {
    await notificationStore.markNotificationAsRead(notification.id);
  }

  // 如果有链接，跳转到对应页面
  if (notification.linkUrl) {
    router.push(notification.linkUrl);
  }
}

// 标记全部已读
async function handleMarkAllRead() {
  try {
    await notificationStore.markAllNotificationsAsRead();
    ElMessage.success('已全部标记为已读');
  } catch (error: any) {
    if (!error.isHandled) {
      ElMessage.error('操作失败');
    }
  }
}

// 跳转到通知中心
function goToNotificationCenter() {
  router.push('/notifications');
}

// 截断文本
function truncateText(text: string, maxLength: number): string {
  if (text.length <= maxLength) return text;
  return text.slice(0, maxLength) + '...';
}

// 格式化时间
function formatTime(time: string): string {
  // 将无时区的时间字符串视为 UTC 时间
  const utcTimeString = time.endsWith('Z') ? time : `${time}Z`;
  const date = new Date(utcTimeString);
  const now = new Date();
  const diff = now.getTime() - date.getTime();

  // 小于1分钟
  if (diff < 60 * 1000) {
    return '刚刚';
  }
  // 小于1小时
  if (diff < 60 * 60 * 1000) {
    return `${Math.floor(diff / (60 * 1000))}分钟前`;
  }
  // 小于24小时
  if (diff < 24 * 60 * 60 * 1000) {
    return `${Math.floor(diff / (60 * 60 * 1000))}小时前`;
  }
  // 小于7天
  if (diff < 7 * 24 * 60 * 60 * 1000) {
    return `${Math.floor(diff / (24 * 60 * 60 * 1000))}天前`;
  }

  // 否则显示日期
  return date.toLocaleDateString('zh-CN');
}

// 开始轮询
function startPolling() {
  // 每30秒刷新一次未读数量
  pollTimer = setInterval(() => {
    notificationStore.fetchUnreadCount();
  }, 30000);
}

// 停止轮询
function stopPolling() {
  if (pollTimer) {
    clearInterval(pollTimer);
    pollTimer = null;
  }
}

// 生命周期
onMounted(() => {
  // 只有登录用户才获取通知和轮询
  if (authStore.isAuthenticated) {
    // 初始加载未读数量
    notificationStore.fetchUnreadCount();
    // 开始轮询
    startPolling();
  }
});

onUnmounted(() => {
  stopPolling();
});
</script>

<style scoped>
.notification-bell {
  display: inline-flex;
  align-items: center;
}

.bell-wrapper {
  cursor: pointer;
  padding: 8px;
  border-radius: 50%;
  transition: background-color 0.2s;
}

.bell-wrapper:hover {
  background-color: var(--el-fill-color-light);
}

.bell-icon {
  color: var(--el-text-color-regular);
}

.notification-dropdown {
  width: 360px;
  max-height: 400px;
  overflow: hidden;
}

.notification-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--el-border-color-light);
}

.notification-header .title {
  font-weight: 600;
  font-size: 16px;
}

.notification-list {
  max-height: 280px;
  overflow-y: auto;
}

.notification-item {
  display: flex;
  align-items: flex-start;
  padding: 12px 16px;
  border-bottom: 1px solid var(--el-border-color-lighter);
  cursor: pointer;
}

.notification-item:hover {
  background-color: var(--el-fill-color-light);
}

.notification-item.unread {
  background-color: var(--el-color-primary-light-9);
}

.notification-content {
  flex: 1;
  min-width: 0;
}

.notification-title {
  font-weight: 500;
  font-size: 14px;
  color: var(--el-text-color-primary);
  margin-bottom: 4px;
}

.notification-text {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-bottom: 4px;
  line-height: 1.4;
}

.notification-time {
  font-size: 12px;
  color: var(--el-text-color-placeholder);
}

.unread-dot {
  margin-left: 8px;
  margin-top: 4px;
}

.notification-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  color: var(--el-text-color-placeholder);
}

.empty-icon {
  margin-bottom: 12px;
  opacity: 0.5;
}

.notification-footer {
  display: flex;
  justify-content: center;
  padding: 12px;
  border-top: 1px solid var(--el-border-color-light);
}
</style>
