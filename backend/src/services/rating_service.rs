use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::models::{
    CreateRatingRequest, Rating, RatingDimension, RatingResponse, RatingSummary, ResourceRatingInfo,
};
use crate::services::NotificationService;

/// 评分服务错误类型
#[derive(Debug, thiserror::Error)]
pub enum RatingError {
    #[error("验证错误: {0}")]
    Validation(String),
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),
}

impl actix_web::ResponseError for RatingError {
    fn error_response(&self) -> actix_web::HttpResponse {
        match self {
            RatingError::Validation(msg) => crate::utils::bad_request(msg),
            RatingError::Database(e) => {
                log::error!("[Rating] 数据库错误 | error={}", e);
                crate::utils::internal_error("服务器内部错误")
            }
        }
    }
}

pub struct RatingService;

impl RatingService {
    /// 创建或更新评分
    pub async fn create_or_update_rating(
        pool: &PgPool,
        resource_id: Uuid,
        user_id: Uuid,
        request: CreateRatingRequest,
    ) -> Result<RatingResponse, RatingError> {
        // 验证评分范围
        if let Err(msg) = request.validate() {
            return Err(RatingError::Validation(msg));
        }

        // 开启事务
        let mut tx = pool.begin().await?;

        // 插入或更新评分
        // 各评分维度列在库中可空，但业务保证一次写入五项，故用 `!` 断言非空以匹配 Rating 结构体
        let rating = sqlx::query_as!(
            Rating,
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
            RETURNING id, resource_id, user_id,
                      difficulty AS "difficulty!", overall_quality AS "overall_quality!",
                      answer_quality AS "answer_quality!", format_quality AS "format_quality!",
                      detail_level AS "detail_level!",
                      created_at AS "created_at!", updated_at AS "updated_at!"
            "#,
            resource_id,
            user_id,
            request.difficulty,
            request.overall_quality,
            request.answer_quality,
            request.format_quality,
            request.detail_level
        )
        .fetch_one(&mut *tx)
        .await?;

        // 更新资源统计（在事务中）
        Self::update_resource_stats_in_tx(&mut tx, resource_id).await?;

        // 提交事务
        tx.commit().await?;

        // 发送通知给资源上传者（如果不是评分自己的资源）- 在事务外执行
        Self::notify_uploader_on_rating(pool, resource_id, user_id).await;

        Ok(rating.into())
    }

    /// 评分时通知资源上传者
    async fn notify_uploader_on_rating(pool: &PgPool, resource_id: Uuid, rater_id: Uuid) {
        // 使用单个JOIN查询获取资源信息和评分者用户名（避免N+1查询）
        let result = sqlx::query!(
            r#"
            SELECT r.uploader_id, r.title, r.author_id, u.username
            FROM resources r
            LEFT JOIN users u ON u.id = $2
            WHERE r.id = $1
            "#,
            resource_id,
            rater_id
        )
        .fetch_optional(pool)
        .await;

        if let Ok(Some(row)) = result {
            // 优先通知作者（如果存在），否则通知上传者
            let notify_user_id = row.author_id.unwrap_or(row.uploader_id);

            // 不给自己发通知
            if notify_user_id != rater_id {
                if let Err(e) = NotificationService::create_rating_notification(
                    pool,
                    resource_id,
                    &row.title,
                    notify_user_id,
                    &row.username,
                )
                .await
                {
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
    ) -> Result<Option<RatingResponse>, RatingError> {
        let rating = sqlx::query_as!(
            Rating,
            r#"
            SELECT id, resource_id, user_id,
                   difficulty AS "difficulty!", overall_quality AS "overall_quality!",
                   answer_quality AS "answer_quality!", format_quality AS "format_quality!",
                   detail_level AS "detail_level!",
                   created_at AS "created_at!", updated_at AS "updated_at!"
            FROM ratings WHERE resource_id = $1 AND user_id = $2
            "#,
            resource_id,
            user_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(rating.map(|r| r.into()))
    }

    /// 删除评分
    pub async fn delete_rating(
        pool: &PgPool,
        resource_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), RatingError> {
        // 开启事务
        let mut tx = pool.begin().await?;

        // 删除评分
        sqlx::query!(
            "DELETE FROM ratings WHERE resource_id = $1 AND user_id = $2",
            resource_id,
            user_id
        )
        .execute(&mut *tx)
        .await?;

        // 更新资源统计（在事务中）
        Self::update_resource_stats_in_tx(&mut tx, resource_id).await?;

        // 提交事务
        tx.commit().await?;

        Ok(())
    }

    /// 获取评分汇总
    pub async fn get_rating_summary(
        pool: &PgPool,
        resource_id: Uuid,
    ) -> Result<RatingSummary, RatingError> {
        // 各汇总列用 `?` 强制为可空 i64，匹配 RatingSummary 的 Option<i64> 字段
        let summary = sqlx::query_as!(
            RatingSummary,
            r#"
            SELECT
                COALESCE(SUM(difficulty), 0) AS "difficulty_total?",
                COUNT(difficulty) AS "difficulty_count?",
                COALESCE(SUM(overall_quality), 0) AS "overall_quality_total?",
                COUNT(overall_quality) AS "overall_quality_count?",
                COALESCE(SUM(answer_quality), 0) AS "answer_quality_total?",
                COUNT(answer_quality) AS "answer_quality_count?",
                COALESCE(SUM(format_quality), 0) AS "format_quality_total?",
                COUNT(format_quality) AS "format_quality_count?",
                COALESCE(SUM(detail_level), 0) AS "detail_level_total?",
                COUNT(detail_level) AS "detail_level_count?"
            FROM ratings
            WHERE resource_id = $1
            "#,
            resource_id
        )
        .fetch_one(pool)
        .await?;

        Ok(summary)
    }

    /// 获取资源评分信息（用于资源详情页）
    pub async fn get_resource_rating_info(
        pool: &PgPool,
        resource_id: Uuid,
        user_id: Option<Uuid>,
    ) -> Result<ResourceRatingInfo, RatingError> {
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

    /// 更新资源统计表中的评分数据（在事务中）
    async fn update_resource_stats_in_tx(
        tx: &mut Transaction<'_, Postgres>,
        resource_id: Uuid,
    ) -> Result<(), RatingError> {
        sqlx::query!(
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
            resource_id
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// 更新资源统计表中的评分数据（独立操作，用于非事务场景）
    #[allow(dead_code)]
    async fn update_resource_stats(pool: &PgPool, resource_id: Uuid) -> Result<(), RatingError> {
        let mut tx = pool.begin().await?;
        Self::update_resource_stats_in_tx(&mut tx, resource_id).await?;
        tx.commit().await?;
        Ok(())
    }
}
