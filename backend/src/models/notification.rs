use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 通知实体（数据库表结构）
#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct Notification {
    pub id: Uuid,
    pub recipient_id: Option<Uuid>,
    pub title: String,
    pub content: String,
    pub notification_type: String,
    pub priority: String,
    pub is_read: bool,
    pub link_url: Option<String>,
    pub created_at: NaiveDateTime,
}

/// 通知类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationType {
    /// 审核结果
    AuditResult,
    /// 申领结果
    ClaimResult,
    /// 评论回复
    CommentReply,
    /// 评分提醒
    RatingReminder,
    /// 管理员消息
    AdminMessage,
    /// 系统通知
    System,
}

impl NotificationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NotificationType::AuditResult => "audit_result",
            NotificationType::ClaimResult => "claim_result",
            NotificationType::CommentReply => "comment_reply",
            NotificationType::RatingReminder => "rating_reminder",
            NotificationType::AdminMessage => "admin_message",
            NotificationType::System => "system",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "audit_result" => Some(NotificationType::AuditResult),
            "claim_result" => Some(NotificationType::ClaimResult),
            "comment_reply" => Some(NotificationType::CommentReply),
            "rating_reminder" => Some(NotificationType::RatingReminder),
            "admin_message" => Some(NotificationType::AdminMessage),
            "system" => Some(NotificationType::System),
            _ => None,
        }
    }
}

/// 通知优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationPriority {
    /// 高优先级（弹窗显示）
    High,
    /// 普通优先级
    Normal,
}

impl NotificationPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            NotificationPriority::High => "high",
            NotificationPriority::Normal => "normal",
        }
    }
}

/// 创建通知请求（内部使用）
#[derive(Debug)]
pub struct CreateNotificationRequest {
    pub recipient_id: Option<Uuid>,
    pub title: String,
    pub content: String,
    pub notification_type: NotificationType,
    pub priority: NotificationPriority,
    pub link_url: Option<String>,
}

/// 通知响应（API 返回）
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationResponse {
    pub id: Uuid,
    pub recipient_id: Option<Uuid>,
    pub title: String,
    pub content: String,
    #[serde(rename = "type")]
    pub notification_type: String,
    pub priority: String,
    pub is_read: bool,
    pub link_url: Option<String>,
    pub created_at: String,
}

impl From<Notification> for NotificationResponse {
    fn from(notification: Notification) -> Self {
        Self {
            id: notification.id,
            recipient_id: notification.recipient_id,
            title: notification.title,
            content: notification.content,
            notification_type: notification.notification_type,
            priority: notification.priority,
            is_read: notification.is_read,
            link_url: notification.link_url,
            created_at: notification
                .created_at
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string(),
        }
    }
}

/// 通知列表查询参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    /// 是否只返回未读通知
    pub unread_only: Option<bool>,
}

/// 通知列表响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationListResponse {
    pub notifications: Vec<NotificationResponse>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub unread_count: i64,
}

/// 未读通知数量响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnreadCountResponse {
    pub count: i64,
}
