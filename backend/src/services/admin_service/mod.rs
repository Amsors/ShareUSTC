//! 管理员服务模块
//!
//! 提供管理后台相关的功能，包括：
//! - 仪表盘统计
//! - 用户管理
//! - 资源审核
//! - 评论管理
//! - 系统通知
//! - 操作日志
//! - 重复资源检测

mod error;
mod service;
mod types;

// Re-export error types
pub use error::AdminError;

// Re-export service
pub use service::AdminService;

// Re-export all types
// 这些类型是公共 API，供其他模块使用
#[allow(unused_imports)]
pub use types::{
    AdminAllResourcesQuery, AdminCommentItem, AdminCommentListQuery, AdminCommentListResponse,
    AdminPaginationQuery, AdminResourceListItem, AdminResourceListResponse, AdminUserListItem,
    AdminUserListResponse, AuditCommentRequest, AuditLogItem, AuditLogItemResponse,
    AuditLogListResponse, AuditLogQuery, AuditResourceRequest, DashboardStats,
    DeleteFavoriteResourcesResult, DetailedStats, DownloadStats, DuplicateResourceCheckResponse,
    DuplicateResourceGroup, DuplicateResourceItem, InteractionStats, NotificationTarget,
    PendingResourceItem, PendingResourceListResponse, RatingDistribution, RecalculateHashResult,
    ResourceStats, ResourceTypeStat, SendNotificationRequest, TopResource, UpdateUserStatusRequest,
    UserRealInfoResponse, UserStats,
};
