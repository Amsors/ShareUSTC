import request from './request';

/**
 * 管理员API封装
 */

// 类型定义
export interface DashboardStats {
  totalUsers: number;
  totalResources: number;
  totalDownloads: number;
  pendingResources: number;
  pendingComments: number;
  todayNewUsers: number;
  todayNewResources: number;
}

export interface User {
  id: string;
  username: string;
  email: string | null;
  role: string;
  isVerified: boolean;
  isActive: boolean;
  createdAt: string;
}

export interface UserListResponse {
  users: User[];
  total: number;
}

export interface Resource {
  id: string;
  title: string;
  courseName: string | null;
  resourceType: string;
  category: string;
  uploaderId: string;
  uploaderName: string | null;
  aiRejectReason: string | null;
  createdAt: string;
}

export interface ResourceListResponse {
  resources: Resource[];
  total: number;
}

export interface Comment {
  id: string;
  resourceId: string;
  resourceTitle: string | null;
  userId: string;
  userName: string | null;
  content: string;
  auditStatus: string;
  createdAt: string;
}

export interface CommentListResponse {
  comments: Comment[];
  total: number;
}

// 仪表盘统计
export const getDashboardStats = (): Promise<DashboardStats> => {
  return request.get('/admin/dashboard');
};

// 用户管理
export const getUserList = (page: number = 1, perPage: number = 20): Promise<UserListResponse> => {
  return request.get('/admin/users', {
    params: { page, perPage }
  });
};

export const updateUserStatus = (userId: string, isActive: boolean): Promise<void> => {
  return request.put(`/admin/users/${userId}/status`, { isActive });
};

// 资源审核
export const getPendingResources = (page: number = 1, perPage: number = 20): Promise<ResourceListResponse> => {
  return request.get('/admin/resources/pending', {
    params: { page, perPage }
  });
};

export const auditResource = (resourceId: string, status: string, reason?: string): Promise<void> => {
  return request.put(`/admin/resources/${resourceId}/audit`, {
    status,
    reason
  });
};

// 评论管理
export const getCommentList = (
  page: number = 1,
  perPage: number = 20,
  auditStatus?: string
): Promise<CommentListResponse> => {
  const params: Record<string, any> = { page, perPage };
  if (auditStatus) {
    params.auditStatus = auditStatus;
  }
  return request.get('/admin/comments', { params });
};

export const deleteComment = (commentId: string): Promise<void> => {
  return request.delete(`/admin/comments/${commentId}`);
};

export const auditComment = (commentId: string, status: string): Promise<void> => {
  return request.put(`/admin/comments/${commentId}/audit`, { status });
};

// =====================
// 发送通知相关
// =====================

export type NotificationTarget = 'all' | 'specific';
export type NotificationType = 'system' | 'admin_message';
export type NotificationPriority = 'normal' | 'high';

export interface SendNotificationRequest {
  target: NotificationTarget;
  userId?: string;
  title: string;
  content: string;
  notificationType: NotificationType;
  priority: NotificationPriority;
  linkUrl?: string;
}

export const sendNotification = (data: SendNotificationRequest): Promise<void> => {
  return request.post('/admin/notifications', data);
};

// =====================
// 详细统计相关
// =====================

export interface UserStats {
  totalUsers: number;
  newUsersToday: number;
  newUsersWeek: number;
  newUsersMonth: number;
}

export interface ResourceTypeStat {
  resourceType: string;
  count: number;
}

export interface ResourceStats {
  totalResources: number;
  pendingResources: number;
  approvedResources: number;
  rejectedResources: number;
  typeDistribution: ResourceTypeStat[];
}

export interface TopResource {
  id: string;
  title: string;
  downloadCount: number;
}

export interface DownloadStats {
  totalDownloads: number;
  downloadsToday: number;
  downloadsWeek: number;
  topResources: TopResource[];
}

export interface RatingDistribution {
  ratingRange: string;
  count: number;
}

export interface InteractionStats {
  totalComments: number;
  totalRatings: number;
  totalLikes: number;
  ratingDistribution: RatingDistribution[];
}

export interface DetailedStats {
  userStats: UserStats;
  resourceStats: ResourceStats;
  downloadStats: DownloadStats;
  interactionStats: InteractionStats;
}

export const getDetailedStats = (): Promise<DetailedStats> => {
  return request.get('/admin/stats/detailed');
};

// =====================
// 操作日志相关
// =====================

export interface AuditLogItem {
  id: string;
  userId: string | null;
  userName: string | null;
  action: string;
  targetType: string | null;
  targetId: string | null;
  details: Record<string, any> | null;
  ipAddress: string | null;
  createdAt: string;
}

export interface AuditLogListResponse {
  logs: AuditLogItem[];
  total: number;
  page: number;
  perPage: number;
}

export interface AuditLogQuery {
  page?: number;
  perPage?: number;
  action?: string;
  userId?: string;
  startDate?: string;
  endDate?: string;
}

export const getAuditLogs = (query: AuditLogQuery = {}): Promise<AuditLogListResponse> => {
  return request.get('/admin/audit-logs', { params: query });
};

// 导出API对象
export const adminApi = {
  getDashboardStats,
  getUserList,
  updateUserStatus,
  getPendingResources,
  auditResource,
  getCommentList,
  deleteComment,
  auditComment,
  sendNotification,
  getDetailedStats,
  getAuditLogs
};
