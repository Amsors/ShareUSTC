use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 收藏夹实体结构体（对应数据库 favorites 表）
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Favorite {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub created_at: NaiveDateTime,
}

/// 收藏夹资源关联实体（对应数据库 favorite_resources 表）
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct FavoriteResource {
    pub favorite_id: Uuid,
    pub resource_id: Uuid,
    pub added_at: NaiveDateTime,
}

/// 创建收藏夹请求 DTO
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFavoriteRequest {
    pub name: String,
}

impl CreateFavoriteRequest {
    /// 验证请求数据
    pub fn validate(&self) -> Result<(), String> {
        let name = self.name.trim();
        if name.is_empty() {
            return Err("收藏夹名称不能为空".to_string());
        }
        if name.len() > 100 {
            return Err("收藏夹名称不能超过100个字符".to_string());
        }
        Ok(())
    }
}

/// 创建收藏夹响应 DTO
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFavoriteResponse {
    pub id: Uuid,
    pub name: String,
    pub created_at: NaiveDateTime,
}

/// 更新收藏夹请求 DTO
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFavoriteRequest {
    pub name: String,
}

impl UpdateFavoriteRequest {
    /// 验证请求数据
    pub fn validate(&self) -> Result<(), String> {
        let name = self.name.trim();
        if name.is_empty() {
            return Err("收藏夹名称不能为空".to_string());
        }
        if name.len() > 100 {
            return Err("收藏夹名称不能超过100个字符".to_string());
        }
        Ok(())
    }
}

/// 收藏夹列表项 DTO
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteListItem {
    pub id: Uuid,
    pub name: String,
    pub resource_count: i64,
    pub created_at: String,
}

/// 收藏夹列表响应 DTO
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteListResponse {
    pub favorites: Vec<FavoriteListItem>,
    pub total: i64,
}

/// 收藏夹资源项 DTO
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteResourceItem {
    pub id: Uuid,
    pub title: String,
    pub course_name: Option<String>,
    pub resource_type: String,
    pub category: String,
    pub tags: Option<Vec<String>>,
    pub file_size: Option<i64>,
    pub added_at: String,
    pub stats: FavoriteResourceStats,
}

/// 收藏夹资源统计 DTO
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteResourceStats {
    pub views: i32,
    pub downloads: i32,
    pub likes: i32,
    pub avg_difficulty: Option<f64>,
    pub avg_quality: Option<f64>,
    pub avg_detail: Option<f64>,
    pub rating_count: i32,
}

/// 收藏夹详情响应 DTO
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteDetailResponse {
    pub id: Uuid,
    pub name: String,
    pub created_at: String,
    pub resource_count: i64,
    pub resources: Vec<FavoriteResourceItem>,
}

/// 添加资源到收藏夹请求 DTO
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddToFavoriteRequest {
    pub resource_id: Uuid,
}

/// 检查资源收藏状态响应 DTO
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckResourceInFavoriteResponse {
    pub in_favorites: Vec<Uuid>,
    pub is_favorited: bool,
}
