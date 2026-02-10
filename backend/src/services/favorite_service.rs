use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    AddToFavoriteRequest, CheckResourceInFavoriteResponse, CreateFavoriteRequest,
    CreateFavoriteResponse, Favorite, FavoriteDetailResponse, FavoriteListItem,
    FavoriteListResponse, FavoriteResourceItem, FavoriteResourceStats, UpdateFavoriteRequest,
};
use crate::services::ResourceError;

pub struct FavoriteService;

impl FavoriteService {
    /// 创建收藏夹
    pub async fn create_favorite(
        pool: &PgPool,
        user_id: Uuid,
        request: CreateFavoriteRequest,
    ) -> Result<CreateFavoriteResponse, ResourceError> {
        // 验证请求
        request.validate().map_err(ResourceError::ValidationError)?;

        let name = request.name.trim();

        // 检查是否已存在同名收藏夹
        let existing = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM favorites WHERE user_id = $1 AND name = $2"
        )
        .bind(user_id)
        .bind(name)
        .fetch_one(pool)
        .await?;

        if existing > 0 {
            return Err(ResourceError::ValidationError(
                "您已存在同名收藏夹".to_string()
            ));
        }

        // 创建收藏夹
        let favorite = sqlx::query_as::<_, Favorite>(
            r#"
            INSERT INTO favorites (user_id, name)
            VALUES ($1, $2)
            RETURNING id, user_id, name, created_at
            "#,
        )
        .bind(user_id)
        .bind(name)
        .fetch_one(pool)
        .await
        .map_err(|e| ResourceError::DatabaseError(e.to_string()))?;

        Ok(CreateFavoriteResponse {
            id: favorite.id,
            name: favorite.name,
            created_at: favorite.created_at,
        })
    }

    /// 获取用户的收藏夹列表
    pub async fn get_user_favorites(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<FavoriteListResponse, ResourceError> {
        // 获取收藏夹列表及资源数量
        let rows = sqlx::query!(
            r#"
            SELECT
                f.id,
                f.name,
                f.created_at,
                COUNT(fr.resource_id) as resource_count
            FROM favorites f
            LEFT JOIN favorite_resources fr ON f.id = fr.favorite_id
            WHERE f.user_id = $1
            GROUP BY f.id, f.name, f.created_at
            ORDER BY f.created_at DESC
            "#,
            user_id
        )
        .fetch_all(pool)
        .await?;

        let favorites: Vec<FavoriteListItem> = rows
            .into_iter()
            .map(|row| FavoriteListItem {
                id: row.id,
                name: row.name,
                resource_count: row.resource_count.unwrap_or(0),
                created_at: row.created_at.map(|dt| dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
                    .unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
            })
            .collect();

        let total = favorites.len() as i64;

        Ok(FavoriteListResponse { favorites, total })
    }

    /// 获取收藏夹详情
    pub async fn get_favorite_detail(
        pool: &PgPool,
        favorite_id: Uuid,
        user_id: Uuid,
    ) -> Result<FavoriteDetailResponse, ResourceError> {
        // 验证收藏夹所有权
        let favorite = sqlx::query_as::<_, Favorite>(
            "SELECT * FROM favorites WHERE id = $1 AND user_id = $2"
        )
        .bind(favorite_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        let favorite = match favorite {
            Some(f) => f,
            None => return Err(ResourceError::NotFound("收藏夹不存在".to_string())),
        };

        // 获取收藏夹中的资源列表
        let rows = sqlx::query!(
            r#"
            SELECT
                r.id,
                r.title,
                r.course_name,
                r.resource_type,
                r.category,
                r.tags,
                r.file_size,
                fr.added_at,
                rs.views,
                rs.downloads,
                rs.likes,
                rs.avg_difficulty,
                rs.avg_quality,
                rs.avg_detail,
                rs.rating_count
            FROM favorite_resources fr
            JOIN resources r ON fr.resource_id = r.id
            LEFT JOIN resource_stats rs ON r.id = rs.resource_id
            WHERE fr.favorite_id = $1
            ORDER BY fr.added_at DESC
            "#,
            favorite_id
        )
        .fetch_all(pool)
        .await?;

        let resources: Vec<FavoriteResourceItem> = rows
            .into_iter()
            .map(|row| {
                // 解析 tags JSON 字段
                let tags: Option<Vec<String>> = row.tags.and_then(|t| {
                    serde_json::from_value::<Vec<String>>(t).ok()
                });

                FavoriteResourceItem {
                    id: row.id,
                    title: row.title,
                    course_name: row.course_name,
                    resource_type: row.resource_type.unwrap_or_default(),
                    category: row.category.unwrap_or_default(),
                    tags,
                    file_size: row.file_size,
                    added_at: row.added_at.map(|dt| dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
                        .unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
                    stats: FavoriteResourceStats {
                        views: row.views.unwrap_or(0),
                        downloads: row.downloads.unwrap_or(0),
                        likes: row.likes.unwrap_or(0),
                        avg_difficulty: row.avg_difficulty,
                        avg_quality: row.avg_quality,
                        avg_detail: row.avg_detail,
                        rating_count: row.rating_count.unwrap_or(0),
                    },
                }
            })
            .collect();

        let resource_count = resources.len() as i64;

        Ok(FavoriteDetailResponse {
            id: favorite.id,
            name: favorite.name,
            created_at: favorite.created_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            resource_count,
            resources,
        })
    }

    /// 更新收藏夹
    pub async fn update_favorite(
        pool: &PgPool,
        favorite_id: Uuid,
        user_id: Uuid,
        request: UpdateFavoriteRequest,
    ) -> Result<(), ResourceError> {
        // 验证请求
        request.validate().map_err(ResourceError::ValidationError)?;

        let name = request.name.trim();

        // 检查收藏夹是否存在且属于当前用户
        let existing = sqlx::query_as::<_, Favorite>(
            "SELECT * FROM favorites WHERE id = $1 AND user_id = $2"
        )
        .bind(favorite_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        if existing.is_none() {
            return Err(ResourceError::NotFound("收藏夹不存在".to_string()));
        }

        // 检查是否已存在其他同名收藏夹
        let duplicate = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM favorites WHERE user_id = $1 AND name = $2 AND id != $3"
        )
        .bind(user_id)
        .bind(name)
        .bind(favorite_id)
        .fetch_one(pool)
        .await?;

        if duplicate > 0 {
            return Err(ResourceError::ValidationError(
                "您已存在同名收藏夹".to_string()
            ));
        }

        // 更新收藏夹
        sqlx::query("UPDATE favorites SET name = $1 WHERE id = $2")
            .bind(name)
            .bind(favorite_id)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// 删除收藏夹
    pub async fn delete_favorite(
        pool: &PgPool,
        favorite_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), ResourceError> {
        // 检查收藏夹是否存在且属于当前用户
        let result = sqlx::query(
            "DELETE FROM favorites WHERE id = $1 AND user_id = $2"
        )
        .bind(favorite_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(ResourceError::NotFound("收藏夹不存在".to_string()));
        }

        Ok(())
    }

    /// 添加资源到收藏夹
    pub async fn add_resource_to_favorite(
        pool: &PgPool,
        favorite_id: Uuid,
        user_id: Uuid,
        request: AddToFavoriteRequest,
    ) -> Result<(), ResourceError> {
        // 检查收藏夹是否存在且属于当前用户
        let favorite = sqlx::query_as::<_, Favorite>(
            "SELECT * FROM favorites WHERE id = $1 AND user_id = $2"
        )
        .bind(favorite_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        if favorite.is_none() {
            return Err(ResourceError::NotFound("收藏夹不存在".to_string()));
        }

        // 检查资源是否存在
        let resource_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM resources WHERE id = $1)"
        )
        .bind(request.resource_id)
        .fetch_one(pool)
        .await?;

        if !resource_exists {
            return Err(ResourceError::NotFound("资源不存在".to_string()));
        }

        // 检查资源是否已在收藏夹中
        let already_in = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM favorite_resources WHERE favorite_id = $1 AND resource_id = $2)"
        )
        .bind(favorite_id)
        .bind(request.resource_id)
        .fetch_one(pool)
        .await?;

        if already_in {
            return Err(ResourceError::ValidationError(
                "资源已在收藏夹中".to_string()
            ));
        }

        // 添加资源到收藏夹
        sqlx::query(
            "INSERT INTO favorite_resources (favorite_id, resource_id) VALUES ($1, $2)"
        )
        .bind(favorite_id)
        .bind(request.resource_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 从收藏夹移除资源
    pub async fn remove_resource_from_favorite(
        pool: &PgPool,
        favorite_id: Uuid,
        resource_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), ResourceError> {
        // 检查收藏夹是否存在且属于当前用户
        let favorite = sqlx::query_as::<_, Favorite>(
            "SELECT * FROM favorites WHERE id = $1 AND user_id = $2"
        )
        .bind(favorite_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        if favorite.is_none() {
            return Err(ResourceError::NotFound("收藏夹不存在".to_string()));
        }

        // 删除关联
        let result = sqlx::query(
            "DELETE FROM favorite_resources WHERE favorite_id = $1 AND resource_id = $2"
        )
        .bind(favorite_id)
        .bind(resource_id)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(ResourceError::NotFound("资源不在该收藏夹中".to_string()));
        }

        Ok(())
    }

    /// 检查资源在哪些收藏夹中
    pub async fn check_resource_in_favorites(
        pool: &PgPool,
        user_id: Uuid,
        resource_id: Uuid,
    ) -> Result<CheckResourceInFavoriteResponse, ResourceError> {
        // 检查资源是否存在
        let resource_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM resources WHERE id = $1)"
        )
        .bind(resource_id)
        .fetch_one(pool)
        .await?;

        if !resource_exists {
            return Err(ResourceError::NotFound("资源不存在".to_string()));
        }

        // 获取包含该资源的所有收藏夹ID
        let favorite_ids = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT f.id
            FROM favorites f
            JOIN favorite_resources fr ON f.id = fr.favorite_id
            WHERE f.user_id = $1 AND fr.resource_id = $2
            "#,
        )
        .bind(user_id)
        .bind(resource_id)
        .fetch_all(pool)
        .await?;

        let is_favorited = !favorite_ids.is_empty();

        Ok(CheckResourceInFavoriteResponse {
            in_favorites: favorite_ids,
            is_favorited,
        })
    }

    /// 获取收藏夹中所有资源的文件路径（用于打包下载）
    pub async fn get_favorite_resource_paths(
        pool: &PgPool,
        favorite_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<(Uuid, String, String)>, ResourceError> {
        // 检查收藏夹是否存在且属于当前用户
        let favorite = sqlx::query_as::<_, Favorite>(
            "SELECT * FROM favorites WHERE id = $1 AND user_id = $2"
        )
        .bind(favorite_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        if favorite.is_none() {
            return Err(ResourceError::NotFound("收藏夹不存在".to_string()));
        }

        // 获取资源文件路径
        let rows = sqlx::query_as::<_, (Uuid, String, String)>(
            r#"
            SELECT r.id, r.title, r.file_path
            FROM favorite_resources fr
            JOIN resources r ON fr.resource_id = r.id
            WHERE fr.favorite_id = $1
            "#,
        )
        .bind(favorite_id)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }
}
