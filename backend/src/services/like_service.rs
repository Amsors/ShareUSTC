use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{Like, LikeStatusResponse, LikeToggleResponse};

pub struct LikeService;

impl LikeService {
    /// 切换点赞状态（点赞/取消点赞）
    pub async fn toggle_like(
        pool: &PgPool,
        resource_id: Uuid,
        user_id: Uuid,
    ) -> Result<LikeToggleResponse, sqlx::Error> {
        // 检查是否已经点赞
        let existing = sqlx::query_as::<_, Like>(
            "SELECT * FROM likes WHERE resource_id = $1 AND user_id = $2"
        )
        .bind(resource_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        let is_liked;
        let message;

        if existing.is_some() {
            // 取消点赞
            sqlx::query(
                "DELETE FROM likes WHERE resource_id = $1 AND user_id = $2"
            )
            .bind(resource_id)
            .bind(user_id)
            .execute(pool)
            .await?;

            is_liked = false;
            message = "已取消点赞".to_string();
        } else {
            // 添加点赞
            sqlx::query(
                "INSERT INTO likes (resource_id, user_id) VALUES ($1, $2)"
            )
            .bind(resource_id)
            .bind(user_id)
            .execute(pool)
            .await?;

            is_liked = true;
            message = "点赞成功".to_string();
        }

        // 更新资源统计中的点赞数
        Self::update_like_count(pool, resource_id).await?;

        // 获取最新的点赞数
        let like_count = Self::get_like_count(pool, resource_id).await?;

        Ok(LikeToggleResponse {
            is_liked,
            like_count,
            message,
        })
    }

    /// 检查用户是否已点赞
    pub async fn check_like_status(
        pool: &PgPool,
        resource_id: Uuid,
        user_id: Uuid,
    ) -> Result<LikeStatusResponse, sqlx::Error> {
        let is_liked = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM likes WHERE resource_id = $1 AND user_id = $2)"
        )
        .bind(resource_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        let like_count = Self::get_like_count(pool, resource_id).await?;

        Ok(LikeStatusResponse {
            is_liked,
            like_count,
        })
    }

    /// 获取资源的点赞数
    pub async fn get_like_count(
        pool: &PgPool,
        resource_id: Uuid,
    ) -> Result<i64, sqlx::Error> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM likes WHERE resource_id = $1"
        )
        .bind(resource_id)
        .fetch_one(pool)
        .await?;

        Ok(count)
    }

    /// 更新资源统计表中的点赞数
    async fn update_like_count(
        pool: &PgPool,
        resource_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        let count = Self::get_like_count(pool, resource_id).await?;

        sqlx::query(
            r#"
            INSERT INTO resource_stats (resource_id, likes)
            VALUES ($1, $2)
            ON CONFLICT (resource_id)
            DO UPDATE SET likes = EXCLUDED.likes
            "#,
        )
        .bind(resource_id)
        .bind(count as i32)
        .execute(pool)
        .await?;

        Ok(())
    }
}
