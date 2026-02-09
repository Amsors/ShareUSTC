import request from './request';

/**
 * 管理员API封装
 */

// 仪表盘统计
export const getDashboardStats = () => {
  return request.get('/admin/dashboard');
};

// 用户管理
export const getUserList = (page: number = 1, perPage: number = 20) => {
  return request.get('/admin/users', {
    params: { page, perPage }
  });
};

export const updateUserStatus = (userId: string, isActive: boolean) => {
  return request.put(`/admin/users/${userId}/status`, { isActive });
};

// 资源审核
export const getPendingResources = (page: number = 1, perPage: number = 20) => {
  return request.get('/admin/resources/pending', {
    params: { page, perPage }
  });
};

export const auditResource = (resourceId: string, status: string, reason?: string) => {
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
) => {
  const params: Record<string, any> = { page, perPage };
  if (auditStatus) {
    params.auditStatus = auditStatus;
  }
  return request.get('/admin/comments', { params });
};

export const deleteComment = (commentId: string) => {
  return request.delete(`/admin/comments/${commentId}`);
};

export const auditComment = (commentId: string, status: string) => {
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
