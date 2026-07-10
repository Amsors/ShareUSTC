// 管理端相关类型定义
// 注意：此处的 User/Resource/Comment 为管理端自有结构，与 types/auth 的 User、
// types/resource 的 Resource 同名但字段不同，仅供管理端接口使用。

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
  details: Record<string, unknown> | null;
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

// 重新计算资源hash
export interface RecalculateHashResult {
  resourceId: string;
  oldHash: string | null;
  newHash: string;
  fileSize: number;
  success: boolean;
  message: string;
}

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
