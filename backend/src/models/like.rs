use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 点赞实体
#[derive(Debug, FromRow, Serialize)]
pub struct Like {
    pub resource_id: Uuid,
    pub user_id: Uuid,
    pub created_at: NaiveDateTime,
}

/// 点赞状态响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LikeStatusResponse {
    pub is_liked: bool,
    pub like_count: i64,
}

/// 点赞操作响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LikeToggleResponse {
    pub is_liked: bool,
    pub like_count: i64,
    pub message: String,
}
