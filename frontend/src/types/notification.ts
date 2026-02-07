/**
 * 通知类型
 */
export type NotificationType =
  | 'audit_result'
  | 'claim_result'
  | 'comment_reply'
  | 'rating_reminder'
  | 'admin_message'
  | 'system';

/**
 * 通知优先级
 */
export type NotificationPriority = 'high' | 'normal';

/**
 * 通知接口
 */
export interface Notification {
  id: string;
  recipientId: string | null;
  title: string;
  content: string;
  type: NotificationType;
  priority: NotificationPriority;
  isRead: boolean;
  linkUrl?: string;
  createdAt: string;
}

/**
 * 通知列表查询参数
 */
export interface NotificationListQuery {
  page?: number;
  perPage?: number;
  unreadOnly?: boolean;
}

/**
 * 通知列表响应
 */
export interface NotificationListResponse {
  notifications: Notification[];
  total: number;
  page: number;
  perPage: number;
  unreadCount: number;
}

/**
 * 未读数量响应
 */
export interface UnreadCountResponse {
  count: number;
}

/**
 * 全部已读响应
 */
export interface MarkAllReadResponse {
  markedCount: number;
}
