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

// 导出API对象
export const adminApi = {
  getDashboardStats,
  getUserList,
  updateUserStatus,
  getPendingResources,
  auditResource,
  getCommentList,
  deleteComment,
  auditComment
};
