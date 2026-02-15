use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    CreateRatingRequest, Rating, RatingResponse, RatingSummary, ResourceRatingInfo, RatingDimension,
};
use crate::services::NotificationService;

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
        if let Err(msg) = request.validate() {
            return Err(sqlx::Error::Protocol(msg.into()));
        }

        // 插入或更新评分
        let rating = sqlx::query_as::<_, Rating>(
            r#"
            INSERT INTO ratings (
                resource_id, user_id,
                difficulty, overall_quality, answer_quality, format_quality, detail_level
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (resource_id, user_id)
            DO UPDATE SET
                difficulty = EXCLUDED.difficulty,
                overall_quality = EXCLUDED.overall_quality,
                answer_quality = EXCLUDED.answer_quality,
                format_quality = EXCLUDED.format_quality,
                detail_level = EXCLUDED.detail_level,
                updated_at = CURRENT_TIMESTAMP
            RETURNING *
            "#,
        )
        .bind(resource_id)
        .bind(user_id)
        .bind(request.difficulty)
        .bind(request.overall_quality)
        .bind(request.answer_quality)
        .bind(request.format_quality)
        .bind(request.detail_level)
        .fetch_one(pool)
        .await?;

        // 更新资源统计
        Self::update_resource_stats(pool, resource_id).await?;

        // 发送通知给资源上传者（如果不是评分自己的资源）
        Self::notify_uploader_on_rating(pool, resource_id, user_id).await;

        Ok(rating.into())
    }

    /// 评分时通知资源上传者
    async fn notify_uploader_on_rating(
        pool: &PgPool,
        resource_id: Uuid,
        rater_id: Uuid,
    ) {
        // 获取资源上传者信息和评分者用户名
        let resource_result = sqlx::query_as::<_, (Uuid, String, Option<Uuid>)>(
            "SELECT uploader_id, title, author_id FROM resources WHERE id = $1"
        )
        .bind(resource_id)
        .fetch_optional(pool)
        .await;

        let rater_result = sqlx::query_scalar::<_, String>(
            "SELECT username FROM users WHERE id = $1"
        )
        .bind(rater_id)
        .fetch_optional(pool)
        .await;

        if let (Ok(Some((uploader_id, resource_title, author_id))), Ok(Some(rater_name))) = (resource_result, rater_result) {
            // 优先通知作者（如果存在），否则通知上传者
            let notify_user_id = author_id.unwrap_or(uploader_id);

            // 不给自己发通知
            if notify_user_id != rater_id {
                if let Err(e) = NotificationService::create_rating_notification(
                    pool,
                    resource_id,
                    &resource_title,
                    notify_user_id,
                    &rater_name,
                ).await {
                    log::warn!("[RatingService] 发送评分通知失败: {}", e);
                }
            }
        }
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
                COALESCE(SUM(difficulty), 0) as difficulty_total,
                COUNT(difficulty) as difficulty_count,
                COALESCE(SUM(overall_quality), 0) as overall_quality_total,
                COUNT(overall_quality) as overall_quality_count,
                COALESCE(SUM(answer_quality), 0) as answer_quality_total,
                COUNT(answer_quality) as answer_quality_count,
                COALESCE(SUM(format_quality), 0) as format_quality_total,
                COUNT(format_quality) as format_quality_count,
                COALESCE(SUM(detail_level), 0) as detail_level_total,
                COUNT(detail_level) as detail_level_count
            FROM ratings
            WHERE resource_id = $1
            "#,
        )
        .bind(resource_id)
        .fetch_one(pool)
        .await?;

        Ok(summary)
    }

    /// 获取资源评分信息（用于资源详情页）
    pub async fn get_resource_rating_info(
        pool: &PgPool,
        resource_id: Uuid,
        user_id: Option<Uuid>,
    ) -> Result<ResourceRatingInfo, sqlx::Error> {
        // 获取评分汇总
        let summary = Self::get_rating_summary(pool, resource_id).await?;

        // 构建维度信息
        let dimensions = vec![
            RatingDimension {
                key: "difficulty".to_string(),
                name: "难度".to_string(),
                description: "资料的难易程度".to_string(),
                avg_score: summary.avg_difficulty(),
            },
            RatingDimension {
                key: "overall_quality".to_string(),
                name: "总体质量".to_string(),
                description: "资料的整体质量".to_string(),
                avg_score: summary.avg_overall_quality(),
            },
            RatingDimension {
                key: "answer_quality".to_string(),
                name: "参考答案质量".to_string(),
                description: "参考答案的准确性和完整性".to_string(),
                avg_score: summary.avg_answer_quality(),
            },
            RatingDimension {
                key: "format_quality".to_string(),
                name: "格式质量".to_string(),
                description: "排版是否清晰美观".to_string(),
                avg_score: summary.avg_format_quality(),
            },
            RatingDimension {
                key: "detail_level".to_string(),
                name: "知识点详细程度".to_string(),
                description: "对于复习提纲等资料的详细程度".to_string(),
                avg_score: summary.avg_detail_level(),
            },
        ];

        // 获取当前用户的评分
        let user_rating = if let Some(uid) = user_id {
            Self::get_user_rating(pool, resource_id, uid).await?
        } else {
            None
        };

        Ok(ResourceRatingInfo {
            resource_id,
            rating_count: summary.rating_count(),
            dimensions,
            user_rating,
        })
    }

    /// 更新资源统计表中的评分数据
    async fn update_resource_stats(
        pool: &PgPool,
        resource_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO resource_stats (
                resource_id,
                difficulty_total, difficulty_count,
                overall_quality_total, overall_quality_count,
                answer_quality_total, answer_quality_count,
                format_quality_total, format_quality_count,
                detail_level_total, detail_level_count
            )
            SELECT
                $1,
                COALESCE(SUM(difficulty), 0),
                COUNT(difficulty),
                COALESCE(SUM(overall_quality), 0),
                COUNT(overall_quality),
                COALESCE(SUM(answer_quality), 0),
                COUNT(answer_quality),
                COALESCE(SUM(format_quality), 0),
                COUNT(format_quality),
                COALESCE(SUM(detail_level), 0),
                COUNT(detail_level)
            FROM ratings
            WHERE resource_id = $1
            ON CONFLICT (resource_id)
            DO UPDATE SET
                difficulty_total = EXCLUDED.difficulty_total,
                difficulty_count = EXCLUDED.difficulty_count,
                overall_quality_total = EXCLUDED.overall_quality_total,
                overall_quality_count = EXCLUDED.overall_quality_count,
                answer_quality_total = EXCLUDED.answer_quality_total,
                answer_quality_count = EXCLUDED.answer_quality_count,
                format_quality_total = EXCLUDED.format_quality_total,
                format_quality_count = EXCLUDED.format_quality_count,
                detail_level_total = EXCLUDED.detail_level_total,
                detail_level_count = EXCLUDED.detail_level_count
            "#,
        )
        .bind(resource_id)
        .execute(pool)
        .await?;

        Ok(())
    }
}
