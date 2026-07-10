use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{LikeStatusResponse, LikeToggleResponse};

/// 点赞服务错误类型
#[derive(Debug, thiserror::Error)]
pub enum LikeError {
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),
}

impl actix_web::ResponseError for LikeError {
    fn error_response(&self) -> actix_web::HttpResponse {
        match self {
            LikeError::Database(e) => {
                log::error!("[Like] 数据库错误 | error={}", e);
                crate::utils::internal_error("服务器内部错误")
            }
        }
    }
}

pub struct LikeService;

impl LikeService {
    /// 切换点赞状态（点赞/取消点赞）
    /// 使用数据库约束（唯一索引）避免竞态条件
    pub async fn toggle_like(
        pool: &PgPool,
        resource_id: Uuid,
        user_id: Uuid,
    ) -> Result<LikeToggleResponse, LikeError> {
        let is_liked;
        let message;

        // 尝试插入点赞记录
        // 利用唯一索引 (resource_id, user_id) 来避免重复点赞
        let insert_result = sqlx::query(
            "INSERT INTO likes (resource_id, user_id) VALUES ($1, $2) ON CONFLICT (resource_id, user_id) DO NOTHING"
        )
        .bind(resource_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        if insert_result.rows_affected() > 0 {
            // 插入成功，表示之前未点赞，现在已点赞
            is_liked = true;
            message = "点赞成功".to_string();
        } else {
            // 插入失败（冲突），表示已经点赞，现在取消点赞
            sqlx::query("DELETE FROM likes WHERE resource_id = $1 AND user_id = $2")
                .bind(resource_id)
                .bind(user_id)
                .execute(pool)
                .await?;

            is_liked = false;
            message = "已取消点赞".to_string();
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
    ) -> Result<LikeStatusResponse, LikeError> {
        let is_liked = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM likes WHERE resource_id = $1 AND user_id = $2)",
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
    pub async fn get_like_count(pool: &PgPool, resource_id: Uuid) -> Result<i64, LikeError> {
        let count =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM likes WHERE resource_id = $1")
                .bind(resource_id)
                .fetch_one(pool)
                .await?;

        Ok(count)
    }

    /// 更新资源统计表中的点赞数
    async fn update_like_count(pool: &PgPool, resource_id: Uuid) -> Result<(), LikeError> {
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
