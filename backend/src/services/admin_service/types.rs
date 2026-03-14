use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 仪表盘统计数据
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardStats {
    pub total_users: i64,
    pub total_resources: i64,
    pub total_downloads: i64,
    pub pending_resources: i64,
    pub pending_comments: i64,
    pub today_new_users: i64,
    pub today_new_resources: i64,
}

/// 管理员用户列表项
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AdminUserListItem {
    pub id: Uuid,
    pub sn: Option<i64>,
    pub username: String,
    pub email: Option<String>,
    pub role: String,
    pub is_verified: bool,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
}

/// 用户列表响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminUserListResponse {
    pub users: Vec<AdminUserListItem>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

/// 用户状态更新请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserStatusRequest {
    pub is_active: bool,
}

/// 用户实名信息响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRealInfoResponse {
    pub user_id: Uuid,
    pub username: String,
    pub is_verified: bool,
    pub real_name: Option<String>,
    pub student_id: Option<String>,
    pub major: Option<String>,
    pub grade: Option<String>,
}

/// 待审核资源列表项
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PendingResourceItem {
    pub id: Uuid,
    pub title: String,
    pub course_name: Option<String>,
    pub resource_type: String,
    pub category: String,
    pub uploader_id: Uuid,
    pub uploader_name: Option<String>,
    pub ai_reject_reason: Option<String>,
    pub created_at: NaiveDateTime,
}

/// 待审核资源列表响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingResourceListResponse {
    pub resources: Vec<PendingResourceItem>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

/// 资源审核请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditResourceRequest {
    pub status: String, // approved, rejected
    pub reason: Option<String>,
}

/// 管理员评论列表项
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AdminCommentItem {
    pub id: Uuid,
    pub resource_id: Uuid,
    pub resource_title: Option<String>,
    pub user_id: Uuid,
    pub user_name: Option<String>,
    pub content: String,
    pub audit_status: String,
    pub created_at: NaiveDateTime,
}

/// 评论列表响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminCommentListResponse {
    pub comments: Vec<AdminCommentItem>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

/// 通知目标枚举
#[derive(Debug, Clone)]
pub enum NotificationTarget {
    All,            // 所有用户
    Specific(Uuid), // 特定用户
}

/// 发送通知请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendNotificationRequest {
    pub target: String,        // "all" 或 "specific"
    pub user_id: Option<Uuid>, // 当 target 为 specific 时使用
    pub title: String,
    pub content: String,
    pub notification_type: String, // system, admin_message
    pub priority: String,          // normal, high
    pub link_url: Option<String>,
}

impl SendNotificationRequest {
    /// 获取通知目标
    pub fn get_target(&self) -> Result<NotificationTarget, super::error::AdminError> {
        match self.target.as_str() {
            "all" => Ok(NotificationTarget::All),
            "specific" => self
                .user_id
                .ok_or_else(|| {
                    super::error::AdminError::ValidationError(
                        "指定用户时必须提供 user_id".to_string(),
                    )
                })
                .map(NotificationTarget::Specific),
            _ => Err(super::error::AdminError::ValidationError(
                "target 必须是 all 或 specific".to_string(),
            )),
        }
    }
}

/// 用户统计
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserStats {
    pub total_users: i64,
    pub new_users_today: i64,
    pub new_users_week: i64,
    pub new_users_month: i64,
}

/// 资源类型统计
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ResourceTypeStat {
    #[sqlx(rename = "type")]
    pub resource_type: String,
    pub count: i64,
}

/// 资源统计
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceStats {
    pub total_resources: i64,
    pub pending_resources: i64,
    pub approved_resources: i64,
    pub rejected_resources: i64,
    pub type_distribution: Vec<ResourceTypeStat>,
}

/// 热门资源
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TopResource {
    pub id: Uuid,
    pub title: String,
    pub download_count: i64,
}

/// 下载统计
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadStats {
    pub total_downloads: i64,
    pub downloads_today: i64,
    pub downloads_week: i64,
    pub top_resources: Vec<TopResource>,
}

/// 评分分布
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RatingDistribution {
    pub rating_range: String,
    pub count: i64,
}

/// 互动统计
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InteractionStats {
    pub total_comments: i64,
    pub total_ratings: i64,
    pub total_likes: i64,
    pub rating_distribution: Vec<RatingDistribution>,
}

/// 详细统计数据
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetailedStats {
    pub user_stats: UserStats,
    pub resource_stats: ResourceStats,
    pub download_stats: DownloadStats,
    pub interaction_stats: InteractionStats,
}

/// 操作日志查询参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub action: Option<String>,
    pub user_id: Option<Uuid>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

/// 操作日志列表项
#[derive(Debug, sqlx::FromRow)]
pub struct AuditLogItem {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub user_name: Option<String>,
    pub action: String,
    pub target_type: Option<String>,
    pub target_id: Option<Uuid>,
    pub details: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub created_at: NaiveDateTime,
}

/// 操作日志响应项（用于序列化）
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogItemResponse {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub user_name: Option<String>,
    pub action: String,
    pub target_type: Option<String>,
    pub target_id: Option<Uuid>,
    pub details: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub created_at: String,
}

impl From<AuditLogItem> for AuditLogItemResponse {
    fn from(item: AuditLogItem) -> Self {
        Self {
            id: item.id,
            user_id: item.user_id,
            user_name: item.user_name,
            action: item.action,
            target_type: item.target_type,
            target_id: item.target_id,
            details: item.details,
            ip_address: item.ip_address,
            created_at: item.created_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
        }
    }
}

/// 操作日志列表响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogListResponse {
    pub logs: Vec<AuditLogItemResponse>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

/// 管理员资源列表项
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AdminResourceListItem {
    pub id: Uuid,
    pub title: String,
    pub course_name: Option<String>,
    pub resource_type: String,
    pub category: String,
    pub uploader_id: Uuid,
    pub uploader_name: Option<String>,
    pub author_id: Option<Uuid>,
    pub author_name: Option<String>,
    pub audit_status: String,
    pub file_size: Option<i64>,
    pub created_at: NaiveDateTime,
    pub views: Option<i32>,
    pub downloads: Option<i32>,
    pub likes: Option<i32>,
}

/// 管理员资源列表响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminResourceListResponse {
    pub resources: Vec<AdminResourceListItem>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

/// 删除收藏夹资源结果
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteFavoriteResourcesResult {
    pub deleted_count: i64,
    pub favorite_name: String,
}

/// Hash重新计算结果
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecalculateHashResult {
    pub resource_id: String,
    pub old_hash: Option<String>,
    pub new_hash: String,
    pub file_size: i64,
    pub success: bool,
    pub message: String,
}

/// 重复资源组中的单个资源
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateResourceItem {
    pub id: Uuid,
    pub title: String,
    pub course_name: Option<String>,
    pub resource_type: String,
    pub category: String,
    pub uploader_id: Uuid,
    pub uploader_name: Option<String>,
    pub file_size: Option<i64>,
    pub file_hash: String,
    pub storage_type: Option<String>,
    pub created_at: NaiveDateTime,
}

/// 重复资源组（相同hash的资源列表）
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateResourceGroup {
    pub file_hash: String,
    pub resource_count: i64,
    pub total_file_size: i64,
    pub resources: Vec<DuplicateResourceItem>,
}

/// 重复资源检测响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateResourceCheckResponse {
    pub total_groups: i64,
    pub total_duplicate_resources: i64,
    pub groups: Vec<DuplicateResourceGroup>,
}
