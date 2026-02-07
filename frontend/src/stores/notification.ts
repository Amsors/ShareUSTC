import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { Notification, NotificationListQuery } from '../types/notification';
import {
  getNotifications,
  markAsRead,
  markAllAsRead,
  getUnreadCount,
  getPriorityNotifications,
  dismissPriorityNotification,
} from '../api/notification';

export const useNotificationStore = defineStore('notification', () => {
  // State
  const notifications = ref<Notification[]>([]);
  const unreadCount = ref(0);
  const priorityNotifications = ref<Notification[]>([]);
  const total = ref(0);
  const page = ref(1);
  const perPage = ref(20);
  const loading = ref(false);

  // Getters
  const hasUnread = computed(() => unreadCount.value > 0);
  const hasPriorityNotifications = computed(() => priorityNotifications.value.length > 0);
  const unreadNotifications = computed(() =>
    notifications.value.filter((n: Notification) => !n.isRead)
  );

  // Actions

  /**
   * 获取通知列表
   */
  async function fetchNotifications(params?: NotificationListQuery) {
    loading.value = true;
    try {
      const response = await getNotifications(params);
      notifications.value = response.notifications;
      total.value = response.total;
      page.value = response.page;
      perPage.value = response.perPage;
      unreadCount.value = response.unreadCount;
      return response;
    } finally {
      loading.value = false;
    }
  }

  /**
   * 标记单条通知为已读
   */
  async function markNotificationAsRead(notificationId: string) {
    await markAsRead(notificationId);

    // 更新本地状态
    const notification = notifications.value.find((n: Notification) => n.id === notificationId);
    if (notification && !notification.isRead) {
      notification.isRead = true;
      unreadCount.value = Math.max(0, unreadCount.value - 1);
    }

    // 也从高优先级列表中移除
    const priorityIndex = priorityNotifications.value.findIndex(
      (n: Notification) => n.id === notificationId
    );
    if (priorityIndex > -1) {
      priorityNotifications.value.splice(priorityIndex, 1);
    }
  }

  /**
   * 标记所有通知为已读
   */
  async function markAllNotificationsAsRead() {
    const response = await markAllAsRead();

    // 更新本地状态
    notifications.value.forEach((n: Notification) => {
      n.isRead = true;
    });
    unreadCount.value = 0;
    priorityNotifications.value = [];

    return response.markedCount;
  }

  /**
   * 获取未读数量
   */
  async function fetchUnreadCount() {
    const count = await getUnreadCount();
    unreadCount.value = count;
    return count;
  }

  /**
   * 获取高优先级通知
   */
  async function fetchPriorityNotifications() {
    const notifications = await getPriorityNotifications();
    priorityNotifications.value = notifications;
    return notifications;
  }

  /**
   * 关闭高优先级通知
   */
  async function dismissPriority(notificationId: string) {
    await dismissPriorityNotification(notificationId);

    // 更新本地状态
    const index = priorityNotifications.value.findIndex(
      (n) => n.id === notificationId
    );
    if (index > -1) {
      priorityNotifications.value.splice(index, 1);
    }

    // 同时标记为已读
    const notification = notifications.value.find((n: Notification) => n.id === notificationId);
    if (notification && !notification.isRead) {
      notification.isRead = true;
      unreadCount.value = Math.max(0, unreadCount.value - 1);
    }
  }

  /**
   * 加载更多通知（分页）
   */
  async function loadMoreNotifications() {
    if (notifications.value.length >= total.value) return;

    loading.value = true;
    try {
      const response = await getNotifications({
        page: page.value + 1,
        perPage: perPage.value,
      });
      notifications.value.push(...response.notifications);
      page.value = response.page;
    } finally {
      loading.value = false;
    }
  }

  /**
   * 重置状态
   */
  function reset() {
    notifications.value = [];
    unreadCount.value = 0;
    priorityNotifications.value = [];
    total.value = 0;
    page.value = 1;
    loading.value = false;
  }

  return {
    // State
    notifications,
    unreadCount,
    priorityNotifications,
    total,
    page,
    perPage,
    loading,

    // Getters
    hasUnread,
    hasPriorityNotifications,
    unreadNotifications,

    // Actions
    fetchNotifications,
    markNotificationAsRead,
    markAllNotificationsAsRead,
    fetchUnreadCount,
    fetchPriorityNotifications,
    dismissPriority,
    loadMoreNotifications,
    reset,
  };
});
