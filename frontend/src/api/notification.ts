import type {
  Notification,
  NotificationListQuery,
  NotificationListResponse,
  UnreadCountResponse,
  MarkAllReadResponse,
} from '../types/notification';
import request from './request';

/**
 * 获取通知列表
 */
export async function getNotifications(
  params?: NotificationListQuery
): Promise<NotificationListResponse> {
  return request.get('/notifications', { params }) as Promise<NotificationListResponse>;
}

/**
 * 标记单条通知为已读
 */
export async function markAsRead(notificationId: string): Promise<void> {
  return request.put(`/notifications/${notificationId}/read`);
}

/**
 * 标记所有通知为已读
 */
export async function markAllAsRead(): Promise<MarkAllReadResponse> {
  return request.put('/notifications/read-all') as Promise<MarkAllReadResponse>;
}

/**
 * 获取未读通知数量
 */
export async function getUnreadCount(): Promise<number> {
  const response = await request.get('/notifications/unread-count') as UnreadCountResponse;
  return response.count;
}

/**
 * 获取高优先级通知
 */
export async function getPriorityNotifications(): Promise<Notification[]> {
  return request.get('/notifications/priority') as Promise<Notification[]>;
}

/**
 * 关闭（标记已读）高优先级通知
 */
export async function dismissPriorityNotification(notificationId: string): Promise<void> {
  return request.put(`/notifications/priority/${notificationId}/dismiss`);
}
