use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 评分实体
#[derive(Debug, FromRow, Serialize)]
pub struct Rating {
    pub id: Uuid,
    pub resource_id: Uuid,
    pub user_id: Uuid,
    pub difficulty: Option<i32>,
    pub quality: Option<i32>,
    pub detail: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// 创建评分请求
#[derive(Debug, Deserialize)]
pub struct CreateRatingRequest {
    pub difficulty: i32,
    pub quality: i32,
    pub detail: i32,
}

/// 评分响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RatingResponse {
    pub id: Uuid,
    pub resource_id: Uuid,
    pub user_id: Uuid,
    pub difficulty: i32,
    pub quality: i32,
    pub detail: i32,
    pub created_at: NaiveDateTime,
}

impl From<Rating> for RatingResponse {
    fn from(rating: Rating) -> Self {
        Self {
            id: rating.id,
            resource_id: rating.resource_id,
            user_id: rating.user_id,
            difficulty: rating.difficulty.unwrap_or(0),
            quality: rating.quality.unwrap_or(0),
            detail: rating.detail.unwrap_or(0),
            created_at: rating.created_at,
        }
    }
}

/// 评分汇总（预留接口）
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct RatingSummary {
    pub avg_difficulty: Option<f64>,
    pub avg_quality: Option<f64>,
    pub avg_detail: Option<f64>,
    pub rating_count: Option<i64>,
}
