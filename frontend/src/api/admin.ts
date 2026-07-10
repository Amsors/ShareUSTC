import request from '@/api/request';
import type {
  TeacherListItem,
  TeacherListResponse,
  CreateTeacherRequest,
  UpdateTeacherRequest,
  TeacherListQuery,
} from '@/types/teacher';
import type {
  CourseListItem,
  CourseListResponse,
  CreateCourseRequest,
  UpdateCourseRequest,
  CourseListQuery,
} from '@/types/course';
import type {
  DashboardStats,
  UserListResponse,
  UserRealInfo,
  ResourceListResponse,
  CommentListResponse,
  SendNotificationRequest,
  DetailedStats,
  AuditLogListResponse,
  AuditLogQuery,
  BatchDeleteTeachersResult,
  BatchDeleteCoursesResult,
  BatchImportCourseItem,
  BatchImportCoursesResult,
  BatchImportTeacherItem,
  BatchImportTeachersResult,
  AdminResourceListResponse,
  GetAllResourcesParams,
  RecalculateHashResult,
  AdminFavoriteListResponse,
  DeleteFavoriteResourcesResult,
  DuplicateResourceCheckResponse,
} from '@/types/admin';

/**
 * 管理员API封装
 */

// 仪表盘统计
export const getDashboardStats = (): Promise<DashboardStats> => {
  return request({ url: '/admin/dashboard', method: 'get' });
};

// 用户管理
export const getUserList = (page: number = 1, perPage: number = 20): Promise<UserListResponse> => {
  return request({ url: '/admin/users', method: 'get', params: { page, perPage } });
};

export const updateUserStatus = (userId: string, isActive: boolean): Promise<void> => {
  return request({ url: `/admin/users/${userId}/status`, method: 'put', data: { isActive } });
};

export const getUserRealInfo = (userId: string): Promise<UserRealInfo> => {
  return request({ url: `/admin/users/${userId}/real-info`, method: 'get' });
};

// 资源审核
export const getPendingResources = (
  page: number = 1,
  perPage: number = 20
): Promise<ResourceListResponse> => {
  return request({ url: '/admin/resources/pending', method: 'get', params: { page, perPage } });
};

export const auditResource = (
  resourceId: string,
  status: string,
  reason?: string
): Promise<void> => {
  return request({
    url: `/admin/resources/${resourceId}/audit`,
    method: 'put',
    data: { status, reason },
  });
};

// 评论管理
export const getCommentList = (
  page: number = 1,
  perPage: number = 20,
  auditStatus?: string
): Promise<CommentListResponse> => {
  const params: Record<string, string | number> = { page, perPage };
  if (auditStatus) {
    params.auditStatus = auditStatus;
  }
  return request({ url: '/admin/comments', method: 'get', params });
};

export const deleteComment = (commentId: string): Promise<void> => {
  return request({ url: `/admin/comments/${commentId}`, method: 'delete' });
};

export const auditComment = (commentId: string, status: string): Promise<void> => {
  return request({ url: `/admin/comments/${commentId}/audit`, method: 'put', data: { status } });
};

// =====================
// 发送通知相关
// =====================

export const sendNotification = (data: SendNotificationRequest): Promise<void> => {
  return request({ url: '/admin/notifications', method: 'post', data });
};

// =====================
// 详细统计相关
// =====================

export const getDetailedStats = (): Promise<DetailedStats> => {
  return request({ url: '/admin/stats/detailed', method: 'get' });
};

// =====================
// 操作日志相关
// =====================

export const getAuditLogs = (query: AuditLogQuery = {}): Promise<AuditLogListResponse> => {
  return request({ url: '/admin/audit-logs', method: 'get', params: query });
};

// =====================
// 教师管理相关
// =====================

export const getTeacherList = (query: TeacherListQuery = {}): Promise<TeacherListResponse> => {
  return request({ url: '/admin/teachers', method: 'get', params: query });
};

export const createTeacher = (data: CreateTeacherRequest): Promise<TeacherListItem> => {
  return request({ url: '/admin/teachers', method: 'post', data });
};

export const updateTeacher = (sn: number, data: UpdateTeacherRequest): Promise<TeacherListItem> => {
  return request({ url: `/admin/teachers/${sn}`, method: 'put', data });
};

export const updateTeacherStatus = (sn: number, isActive: boolean): Promise<TeacherListItem> => {
  return request({ url: `/admin/teachers/${sn}/status`, method: 'put', data: { isActive } });
};

export const deleteTeacher = (sn: number): Promise<void> => {
  return request({ url: `/admin/teachers/${sn}`, method: 'delete' });
};

// =====================
// 课程管理相关
// =====================

export const getCourseList = (query: CourseListQuery = {}): Promise<CourseListResponse> => {
  return request({ url: '/admin/courses', method: 'get', params: query });
};

export const createCourse = (data: CreateCourseRequest): Promise<CourseListItem> => {
  return request({ url: '/admin/courses', method: 'post', data });
};

export const updateCourse = (sn: number, data: UpdateCourseRequest): Promise<CourseListItem> => {
  return request({ url: `/admin/courses/${sn}`, method: 'put', data });
};

export const updateCourseStatus = (sn: number, isActive: boolean): Promise<CourseListItem> => {
  return request({ url: `/admin/courses/${sn}/status`, method: 'put', data: { isActive } });
};

export const deleteCourse = (sn: number): Promise<void> => {
  return request({ url: `/admin/courses/${sn}`, method: 'delete' });
};

// =====================
// 批量删除相关
// =====================

export const batchDeleteTeachers = (sns: string): Promise<BatchDeleteTeachersResult> => {
  return request({ url: '/admin/teachers/batch-delete', method: 'post', data: { sns } });
};

export const batchDeleteCourses = (sns: string): Promise<BatchDeleteCoursesResult> => {
  return request({ url: '/admin/courses/batch-delete', method: 'post', data: { sns } });
};

// =====================
// 批量导入相关
// =====================

export const batchImportCourses = (
  courses: BatchImportCourseItem[]
): Promise<BatchImportCoursesResult> => {
  return request({ url: '/admin/courses/batch-import', method: 'post', data: { courses } });
};

export const batchImportTeachers = (
  teachers: BatchImportTeacherItem[]
): Promise<BatchImportTeachersResult> => {
  return request({ url: '/admin/teachers/batch-import', method: 'post', data: { teachers } });
};

// 从文件导入教师
export const batchImportTeachersFromFile = (file: File): Promise<BatchImportTeachersResult> => {
  const formData = new FormData();
  formData.append('file', file);
  return request({
    url: '/admin/teachers/batch-import-file',
    method: 'post',
    data: formData,
    headers: {
      'Content-Type': 'multipart/form-data',
    },
  });
};

// 从文件导入课程
export const batchImportCoursesFromFile = (file: File): Promise<BatchImportCoursesResult> => {
  const formData = new FormData();
  formData.append('file', file);
  return request({
    url: '/admin/courses/batch-import-file',
    method: 'post',
    data: formData,
    headers: {
      'Content-Type': 'multipart/form-data',
    },
  });
};

// =====================
// 资料管理相关
// =====================

export const getAllResources = (
  params: GetAllResourcesParams = {}
): Promise<AdminResourceListResponse> => {
  return request({ url: '/admin/resources/all', method: 'get', params });
};

export const adminDeleteResource = (resourceId: string): Promise<void> => {
  return request({ url: `/admin/resources/${resourceId}`, method: 'delete' });
};

// 重新计算资源hash
export const recalculateResourceHash = (resourceId: string): Promise<RecalculateHashResult> => {
  // 设置1分钟超时，因为大文件在OSS上计算hash需要较长时间
  return request({
    url: `/admin/resources/${resourceId}/recalculate-hash`,
    method: 'post',
    data: {},
    timeout: 60000,
  });
};

export const getAdminFavorites = (): Promise<AdminFavoriteListResponse> => {
  return request({ url: '/admin/favorites', method: 'get' });
};

export const deleteAllFavoriteResources = (
  favoriteId: string
): Promise<DeleteFavoriteResourcesResult> => {
  return request({ url: `/admin/favorites/${favoriteId}/resources`, method: 'delete' });
};

/**
 * 检测重复资源（根据文件hash）
 */
export const checkDuplicateResources = (): Promise<DuplicateResourceCheckResponse> => {
  return request({ url: '/admin/duplicate-resources', method: 'get' });
};
