use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 评论实体
#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct Comment {
    pub id: Uuid,
    pub resource_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub audit_status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// 创建评论请求
#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    pub content: String,
}

/// 评论响应（包含用户信息）
#[derive(Debug, Serialize)]
pub struct CommentResponse {
    pub id: Uuid,
    #[serde(rename = "resourceId")]
    pub resource_id: Uuid,
    #[serde(rename = "userId")]
    pub user_id: Uuid,
    #[serde(rename = "userName")]
    pub user_name: String,
    #[serde(rename = "userAvatar")]
    pub user_avatar: Option<String>,
    pub content: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,  // 使用 String 类型，在构造时格式化为 ISO 8601 格式
}

/// 评论列表查询
#[derive(Debug, Deserialize)]
pub struct CommentListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

/// 评论列表响应
#[derive(Debug, Serialize)]
pub struct CommentListResponse {
    pub comments: Vec<CommentResponse>,
    pub total: i64,
    pub page: i64,
    #[serde(rename = "perPage")]
    pub per_page: i64,
}
