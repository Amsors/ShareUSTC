<template>
  <div class="notification-list">
    <div
      v-for="notification in notifications"
      :key="notification.id"
      class="notification-item"
      :class="{ unread: !notification.isRead }"
      @click="handleClick(notification)"
    >
      <div class="notification-icon" :class="getIconClass(notification.type)">
        <el-icon size="20">
          <component :is="getIcon(notification.type)" />
        </el-icon>
      </div>

      <div class="notification-content">
        <div class="notification-header">
          <h4 class="notification-title">{{ notification.title }}</h4>
          <el-tag
            v-if="notification.priority === 'high'"
            type="danger"
            size="small"
            effect="dark"
          >
            重要
          </el-tag>
        </div>

        <p class="notification-text">{{ notification.content }}</p>

        <div class="notification-footer">
          <span class="notification-time">{{ formatTime(notification.createdAt) }}</span>

          <el-button
            v-if="!notification.isRead"
            link
            type="primary"
            size="small"
            @click.stop="handleRead(notification)"
          >
            标记已读
          </el-button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// 无需额外导入 computed
import {
  Bell,
  Check,
  Document,
  Star,
  Warning,
  ChatDotRound,
} from '@element-plus/icons-vue';
import type { Notification, NotificationType } from '../../types/notification';

const props = defineProps<{
  notifications: Notification[];
}>();

const emit = defineEmits<{
  read: [notification: Notification];
  click: [notification: Notification];
}>();

// 根据通知类型获取图标
function getIcon(type: NotificationType) {
  switch (type) {
    case 'audit_result':
      return Document;
    case 'claim_result':
      return Check;
    case 'comment_reply':
      return ChatDotRound;
    case 'rating_reminder':
      return Star;
    case 'admin_message':
      return Warning;
    case 'system':
    default:
      return Bell;
  }
}

// 根据通知类型获取样式类
function getIconClass(type: NotificationType) {
  switch (type) {
    case 'audit_result':
      return 'type-audit';
    case 'claim_result':
      return 'type-claim';
    case 'comment_reply':
      return 'type-comment';
    case 'rating_reminder':
      return 'type-rating';
    case 'admin_message':
      return 'type-admin';
    case 'system':
    default:
      return 'type-system';
  }
}

// 处理点击
function handleClick(notification: Notification) {
  emit('click', notification);
}

// 处理标记已读
function handleRead(notification: Notification) {
  emit('read', notification);
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

  // 否则显示日期时间
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  });
}
</script>

<style scoped>
.notification-list {
  display: flex;
  flex-direction: column;
  gap: 1px;
  background-color: var(--el-border-color-lighter);
}

.notification-item {
  display: flex;
  align-items: flex-start;
  gap: 16px;
  padding: 16px;
  background-color: var(--el-bg-color);
  cursor: pointer;
  transition: background-color 0.2s;
}

.notification-item:hover {
  background-color: var(--el-fill-color-light);
}

.notification-item.unread {
  background-color: var(--el-color-primary-light-9);
}

.notification-item.unread .notification-title {
  font-weight: 600;
}

.notification-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border-radius: 50%;
  flex-shrink: 0;
}

.notification-icon.type-audit {
  background-color: var(--el-color-info-light-9);
  color: var(--el-color-info);
}

.notification-icon.type-claim {
  background-color: var(--el-color-success-light-9);
  color: var(--el-color-success);
}

.notification-icon.type-comment {
  background-color: var(--el-color-primary-light-9);
  color: var(--el-color-primary);
}

.notification-icon.type-rating {
  background-color: var(--el-color-warning-light-9);
  color: var(--el-color-warning);
}

.notification-icon.type-admin {
  background-color: var(--el-color-danger-light-9);
  color: var(--el-color-danger);
}

.notification-icon.type-system {
  background-color: var(--el-color-info-light-9);
  color: var(--el-text-color-secondary);
}

.notification-content {
  flex: 1;
  min-width: 0;
}

.notification-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.notification-title {
  margin: 0;
  font-size: 15px;
  font-weight: 500;
  color: var(--el-text-color-primary);
}

.notification-text {
  margin: 0 0 8px 0;
  font-size: 14px;
  color: var(--el-text-color-regular);
  line-height: 1.5;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.notification-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.notification-time {
  font-size: 12px;
  color: var(--el-text-color-placeholder);
}
</style>
