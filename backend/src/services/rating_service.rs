use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    CreateRatingRequest, Rating, RatingResponse, RatingSummary,
};

pub struct RatingService;

impl RatingService {
    /// 创建或更新评分
    pub async fn create_or_update_rating(
        pool: &PgPool,
        resource_id: Uuid,
        user_id: Uuid,
        request: CreateRatingRequest,
    ) -> Result<RatingResponse, sqlx::Error> {
        // 验证评分范围
        if request.difficulty < 1 || request.difficulty > 10
            || request.quality < 1 || request.quality > 10
            || request.detail < 1 || request.detail > 10
        {
            return Err(sqlx::Error::RowNotFound); // 使用错误类型表示验证失败
        }

        // 插入或更新评分
        let rating = sqlx::query_as::<_, Rating>(
            r#"
            INSERT INTO ratings (resource_id, user_id, difficulty, quality, detail)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (resource_id, user_id)
            DO UPDATE SET
                difficulty = EXCLUDED.difficulty,
                quality = EXCLUDED.quality,
                detail = EXCLUDED.detail,
                updated_at = CURRENT_TIMESTAMP
            RETURNING *
            "#,
        )
        .bind(resource_id)
        .bind(user_id)
        .bind(request.difficulty)
        .bind(request.quality)
        .bind(request.detail)
        .fetch_one(pool)
        .await?;

        // 更新资源统计
        Self::update_resource_stats(pool, resource_id).await?;

        Ok(rating.into())
    }

    /// 获取用户对资源的评分
    pub async fn get_user_rating(
        pool: &PgPool,
        resource_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<RatingResponse>, sqlx::Error> {
        let rating = sqlx::query_as::<_, Rating>(
            "SELECT * FROM ratings WHERE resource_id = $1 AND user_id = $2"
        )
        .bind(resource_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        Ok(rating.map(|r| r.into()))
    }

    /// 删除评分
    pub async fn delete_rating(
        pool: &PgPool,
        resource_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "DELETE FROM ratings WHERE resource_id = $1 AND user_id = $2"
        )
        .bind(resource_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        // 更新资源统计
        Self::update_resource_stats(pool, resource_id).await?;

        Ok(())
    }

    /// 获取评分汇总
    pub async fn get_rating_summary(
        pool: &PgPool,
        resource_id: Uuid,
    ) -> Result<RatingSummary, sqlx::Error> {
        let summary = sqlx::query_as::<_, RatingSummary>(
            r#"
            SELECT
                AVG(difficulty) as avg_difficulty,
                AVG(quality) as avg_quality,
                AVG(detail) as avg_detail,
                COUNT(*) as rating_count
            FROM ratings
            WHERE resource_id = $1
            "#,
        )
        .bind(resource_id)
        .fetch_one(pool)
        .await?;

        Ok(summary)
    }

    /// 更新资源统计表中的评分数据
    async fn update_resource_stats(
        pool: &PgPool,
        resource_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO resource_stats (resource_id, avg_difficulty, avg_quality, avg_detail, rating_count)
            SELECT
                $1,
                AVG(difficulty),
                AVG(quality),
                AVG(detail),
                COUNT(*)::INTEGER
            FROM ratings
            WHERE resource_id = $1
            ON CONFLICT (resource_id)
            DO UPDATE SET
                avg_difficulty = EXCLUDED.avg_difficulty,
                avg_quality = EXCLUDED.avg_quality,
                avg_detail = EXCLUDED.avg_detail,
                rating_count = EXCLUDED.rating_count
            "#,
        )
        .bind(resource_id)
        .execute(pool)
        .await?;

        Ok(())
    }
}
