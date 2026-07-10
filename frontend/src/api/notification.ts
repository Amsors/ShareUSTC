import type {
  Notification,
  NotificationListQuery,
  NotificationListResponse,
  UnreadCountResponse,
  MarkAllReadResponse,
} from '@/types/notification';
import request from '@/api/request';

/**
 * 获取通知列表
 */
export async function getNotifications(
  params?: NotificationListQuery
): Promise<NotificationListResponse> {
  return request({
    url: '/notifications',
    method: 'get',
    params,
  }) as Promise<NotificationListResponse>;
}

/**
 * 标记单条通知为已读
 */
export async function markAsRead(notificationId: string): Promise<void> {
  return request({ url: `/notifications/${notificationId}/read`, method: 'put' });
}

/**
 * 标记所有通知为已读
 */
export async function markAllAsRead(): Promise<MarkAllReadResponse> {
  return request({
    url: '/notifications/read-all',
    method: 'put',
  }) as Promise<MarkAllReadResponse>;
}

/**
 * 获取未读通知数量
 */
export async function getUnreadCount(): Promise<number> {
  const response = (await request({
    url: '/notifications/unread-count',
    method: 'get',
  })) as UnreadCountResponse;
  return response.count;
}

/**
 * 获取高优先级通知
 */
export async function getPriorityNotifications(): Promise<Notification[]> {
  return request({ url: '/notifications/priority', method: 'get' }) as Promise<Notification[]>;
}

/**
 * 关闭（标记已读）高优先级通知
 */
export async function dismissPriorityNotification(notificationId: string): Promise<void> {
  return request({ url: `/notifications/priority/${notificationId}/dismiss`, method: 'put' });
}
