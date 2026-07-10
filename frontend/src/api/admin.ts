import request from './request';
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
    params: { page, perPage },
  });
};

export const updateUserStatus = (userId: string, isActive: boolean): Promise<void> => {
  return request.put(`/admin/users/${userId}/status`, { isActive });
};

// 用户实名信息
export interface UserRealInfo {
  userId: string;
  username: string;
  isVerified: boolean;
  realName?: string;
  studentId?: string;
  major?: string;
  grade?: string;
}

export const getUserRealInfo = (userId: string): Promise<UserRealInfo> => {
  return request.get(`/admin/users/${userId}/real-info`);
};

// 资源审核
export const getPendingResources = (
  page: number = 1,
  perPage: number = 20
): Promise<ResourceListResponse> => {
  return request.get('/admin/resources/pending', {
    params: { page, perPage },
  });
};

export const auditResource = (
  resourceId: string,
  status: string,
  reason?: string
): Promise<void> => {
  return request.put(`/admin/resources/${resourceId}/audit`, {
    status,
    reason,
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

// =====================
// 教师管理相关
// =====================

export const getTeacherList = (query: TeacherListQuery = {}): Promise<TeacherListResponse> => {
  return request.get('/admin/teachers', { params: query });
};

export const createTeacher = (data: CreateTeacherRequest): Promise<TeacherListItem> => {
  return request.post('/admin/teachers', data);
};

export const updateTeacher = (sn: number, data: UpdateTeacherRequest): Promise<TeacherListItem> => {
  return request.put(`/admin/teachers/${sn}`, data);
};

export const updateTeacherStatus = (sn: number, isActive: boolean): Promise<TeacherListItem> => {
  return request.put(`/admin/teachers/${sn}/status`, { isActive });
};

export const deleteTeacher = (sn: number): Promise<void> => {
  return request.delete(`/admin/teachers/${sn}`);
};

// =====================
// 课程管理相关
// =====================

export const getCourseList = (query: CourseListQuery = {}): Promise<CourseListResponse> => {
  return request.get('/admin/courses', { params: query });
};

export const createCourse = (data: CreateCourseRequest): Promise<CourseListItem> => {
  return request.post('/admin/courses', data);
};

export const updateCourse = (sn: number, data: UpdateCourseRequest): Promise<CourseListItem> => {
  return request.put(`/admin/courses/${sn}`, data);
};

export const updateCourseStatus = (sn: number, isActive: boolean): Promise<CourseListItem> => {
  return request.put(`/admin/courses/${sn}/status`, { isActive });
};

export const deleteCourse = (sn: number): Promise<void> => {
  return request.delete(`/admin/courses/${sn}`);
};

// =====================
// 批量删除相关
// =====================

export interface BatchDeleteTeachersResult {
  successCount: number;
  failCount: number;
  notFoundCount: number;
  failedItems: FailedTeacherDeleteItem[];
}

export interface FailedTeacherDeleteItem {
  sn: number;
  reason: string;
}

export interface BatchDeleteCoursesResult {
  successCount: number;
  failCount: number;
  notFoundCount: number;
  failedItems: FailedCourseDeleteItem[];
}

export interface FailedCourseDeleteItem {
  sn: number;
  reason: string;
}

export const batchDeleteTeachers = (sns: string): Promise<BatchDeleteTeachersResult> => {
  return request.post('/admin/teachers/batch-delete', { sns });
};

export const batchDeleteCourses = (sns: string): Promise<BatchDeleteCoursesResult> => {
  return request.post('/admin/courses/batch-delete', { sns });
};

// =====================
// 批量导入相关
// =====================

export interface BatchImportCourseItem {
  name: string;
  semester?: string;
  credits?: number;
}

export interface FailedCourseImportItem {
  name: string;
  reason: string;
}

export interface BatchImportCoursesResult {
  successCount: number;
  failCount: number;
  failedItems: FailedCourseImportItem[];
}

export interface BatchImportTeacherItem {
  name: string;
  department?: string;
}

export interface FailedTeacherImportItem {
  name: string;
  reason: string;
}

export interface BatchImportTeachersResult {
  successCount: number;
  failCount: number;
  failedItems: FailedTeacherImportItem[];
}

export const batchImportCourses = (
  courses: BatchImportCourseItem[]
): Promise<BatchImportCoursesResult> => {
  return request.post('/admin/courses/batch-import', { courses });
};

export const batchImportTeachers = (
  teachers: BatchImportTeacherItem[]
): Promise<BatchImportTeachersResult> => {
  return request.post('/admin/teachers/batch-import', { teachers });
};

// 从文件导入教师
export const batchImportTeachersFromFile = (file: File): Promise<BatchImportTeachersResult> => {
  const formData = new FormData();
  formData.append('file', file);
  return request.post('/admin/teachers/batch-import-file', formData, {
    headers: {
      'Content-Type': 'multipart/form-data',
    },
  });
};

// 从文件导入课程
export const batchImportCoursesFromFile = (file: File): Promise<BatchImportCoursesResult> => {
  const formData = new FormData();
  formData.append('file', file);
  return request.post('/admin/courses/batch-import-file', formData, {
    headers: {
      'Content-Type': 'multipart/form-data',
    },
  });
};

// =====================
// 资料管理相关
// =====================

export interface AdminResource {
  id: string;
  title: string;
  courseName?: string;
  resourceType: string;
  category: string;
  uploaderId: string;
  uploaderName?: string;
  authorId?: string;
  authorName?: string;
  auditStatus: string;
  fileSize?: number;
  createdAt: string;
  views?: number;
  downloads?: number;
  likes?: number;
}

export interface AdminResourceListResponse {
  resources: AdminResource[];
  total: number;
  page: number;
  perPage: number;
}

export interface AdminFavorite {
  id: string;
  name: string;
  resourceCount: number;
  createdAt: string;
}

export interface AdminFavoriteListResponse {
  favorites: AdminFavorite[];
  total: number;
}

export interface DeleteFavoriteResourcesResult {
  deletedCount: number;
  favoriteName: string;
}

export interface GetAllResourcesParams {
  page?: number;
  perPage?: number;
  keyword?: string;
}

export const getAllResources = (
  params: GetAllResourcesParams = {}
): Promise<AdminResourceListResponse> => {
  return request.get('/admin/resources/all', { params });
};

export const adminDeleteResource = (resourceId: string): Promise<void> => {
  return request.delete(`/admin/resources/${resourceId}`);
};

// 重新计算资源hash
export interface RecalculateHashResult {
  resourceId: string;
  oldHash: string | null;
  newHash: string;
  fileSize: number;
  success: boolean;
  message: string;
}

export const recalculateResourceHash = (resourceId: string): Promise<RecalculateHashResult> => {
  // 设置1分钟超时，因为大文件在OSS上计算hash需要较长时间
  return request.post(
    `/admin/resources/${resourceId}/recalculate-hash`,
    {},
    {
      timeout: 60000,
    }
  );
};

export const getAdminFavorites = (): Promise<AdminFavoriteListResponse> => {
  return request.get('/admin/favorites');
};

export const deleteAllFavoriteResources = (
  favoriteId: string
): Promise<DeleteFavoriteResourcesResult> => {
  return request.delete(`/admin/favorites/${favoriteId}/resources`);
};

// 重复资源检测相关类型
export interface DuplicateResourceItem {
  id: string;
  title: string;
  courseName: string | null;
  resourceType: string;
  category: string;
  uploaderId: string;
  uploaderName: string | null;
  fileSize: number | null;
  fileHash: string;
  storageType: string | null;
  createdAt: string;
}

export interface DuplicateResourceGroup {
  fileHash: string;
  resourceCount: number;
  totalFileSize: number;
  resources: DuplicateResourceItem[];
}

export interface DuplicateResourceCheckResponse {
  totalGroups: number;
  totalDuplicateResources: number;
  groups: DuplicateResourceGroup[];
}

/**
 * 检测重复资源（根据文件hash）
 */
export const checkDuplicateResources = (): Promise<DuplicateResourceCheckResponse> => {
  return request.get('/admin/duplicate-resources');
};

// 导出API对象
export const adminApi = {
  getDashboardStats,
  getUserList,
  updateUserStatus,
  getUserRealInfo,
  getPendingResources,
  auditResource,
  getCommentList,
  deleteComment,
  auditComment,
  sendNotification,
  getDetailedStats,
  getAuditLogs,
  // 教师管理
  getTeacherList,
  createTeacher,
  updateTeacher,
  updateTeacherStatus,
  deleteTeacher,
  // 课程管理
  getCourseList,
  createCourse,
  updateCourse,
  updateCourseStatus,
  deleteCourse,
  // 批量导入
  batchImportCourses,
  batchImportTeachers,
  batchImportCoursesFromFile,
  batchImportTeachersFromFile,
  // 批量删除
  batchDeleteTeachers,
  batchDeleteCourses,
  // 资料管理
  getAllResources,
  adminDeleteResource,
  recalculateResourceHash,
  getAdminFavorites,
  deleteAllFavoriteResources,
  // 重复资源检测
  checkDuplicateResources,
};
