import { apiClient as client } from './client';

/**
 * 管理员API封装
 */

// 仪表盘统计
export const getDashboardStats = () => {
  return client.get('/admin/dashboard');
};

// 用户管理
export const getUserList = (page: number = 1, perPage: number = 20) => {
  return client.get('/admin/users', {
    params: { page, perPage }
  });
};

export const updateUserStatus = (userId: string, isActive: boolean) => {
  return client.put(`/admin/users/${userId}/status`, { isActive });
};

// 资源审核
export const getPendingResources = (page: number = 1, perPage: number = 20) => {
  return client.get('/admin/resources/pending', {
    params: { page, perPage }
  });
};

export const auditResource = (resourceId: string, status: string, reason?: string) => {
  return client.put(`/admin/resources/${resourceId}/audit`, {
    status,
    reason
  });
};

// 评论管理
export const getCommentList = (
  page: number = 1,
  perPage: number = 20,
  auditStatus?: string
) => {
  const params: Record<string, any> = { page, perPage };
  if (auditStatus) {
    params.auditStatus = auditStatus;
  }
  return client.get('/admin/comments', { params });
};

export const deleteComment = (commentId: string) => {
  return client.delete(`/admin/comments/${commentId}`);
};

export const auditComment = (commentId: string, status: string) => {
  return client.put(`/admin/comments/${commentId}/audit`, { status });
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
