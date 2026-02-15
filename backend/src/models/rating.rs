use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 评分实体 - 包含5个评分维度
#[derive(Debug, FromRow, Serialize)]
pub struct Rating {
    pub id: Uuid,
    pub resource_id: Uuid,
    pub user_id: Uuid,
    /// 难度评分 (1-10)
    pub difficulty: Option<i32>,
    /// 总体质量评分 (1-10)
    pub overall_quality: Option<i32>,
    /// 参考答案质量 (1-10)
    pub answer_quality: Option<i32>,
    /// 格式质量/排版清晰度 (1-10)
    pub format_quality: Option<i32>,
    /// 知识点详细程度 (1-10)
    pub detail_level: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// 创建评分请求 - 5个维度全部必填
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRatingRequest {
    /// 难度评分 (1-10)
    pub difficulty: i32,
    /// 总体质量评分 (1-10)
    pub overall_quality: i32,
    /// 参考答案质量 (1-10)
    pub answer_quality: i32,
    /// 格式质量/排版清晰度 (1-10)
    pub format_quality: i32,
    /// 知识点详细程度 (1-10)
    pub detail_level: i32,
}

impl CreateRatingRequest {
    /// 验证所有评分在有效范围内
    pub fn validate(&self) -> Result<(), String> {
        let dimensions = [
            ("difficulty", self.difficulty),
            ("overall_quality", self.overall_quality),
            ("answer_quality", self.answer_quality),
            ("format_quality", self.format_quality),
            ("detail_level", self.detail_level),
        ];

        for (name, value) in dimensions {
            if value < 1 || value > 10 {
                return Err(format!("{} must be between 1 and 10, got {}", name, value));
            }
        }

        Ok(())
    }
}

/// 评分响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RatingResponse {
    pub id: Uuid,
    pub resource_id: Uuid,
    pub user_id: Uuid,
    /// 难度评分 (1-10)
    pub difficulty: i32,
    /// 总体质量评分 (1-10)
    pub overall_quality: i32,
    /// 参考答案质量 (1-10)
    pub answer_quality: i32,
    /// 格式质量/排版清晰度 (1-10)
    pub format_quality: i32,
    /// 知识点详细程度 (1-10)
    pub detail_level: i32,
    pub created_at: NaiveDateTime,
}

impl From<Rating> for RatingResponse {
    fn from(rating: Rating) -> Self {
        Self {
            id: rating.id,
            resource_id: rating.resource_id,
            user_id: rating.user_id,
            difficulty: rating.difficulty.unwrap_or(0),
            overall_quality: rating.overall_quality.unwrap_or(0),
            answer_quality: rating.answer_quality.unwrap_or(0),
            format_quality: rating.format_quality.unwrap_or(0),
            detail_level: rating.detail_level.unwrap_or(0),
            created_at: rating.created_at,
        }
    }
}

/// 评分统计 - 包含每个维度的总分和评分次数
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RatingSummary {
    /// 难度总得分 (PostgreSQL SUM/COUNT 返回 BIGINT)
    pub difficulty_total: Option<i64>,
    /// 难度评分次数
    pub difficulty_count: Option<i64>,
    /// 总体质量总得分
    pub overall_quality_total: Option<i64>,
    /// 总体质量评分次数
    pub overall_quality_count: Option<i64>,
    /// 参考答案质量总得分
    pub answer_quality_total: Option<i64>,
    /// 参考答案质量评分次数
    pub answer_quality_count: Option<i64>,
    /// 格式质量总得分
    pub format_quality_total: Option<i64>,
    /// 格式质量评分次数
    pub format_quality_count: Option<i64>,
    /// 知识点详细程度总得分
    pub detail_level_total: Option<i64>,
    /// 知识点详细程度评分次数
    pub detail_level_count: Option<i64>,
}

impl RatingSummary {
    /// 计算难度平均分
    pub fn avg_difficulty(&self) -> Option<f64> {
        match (self.difficulty_total, self.difficulty_count) {
            (Some(total), Some(count)) if count > 0 => Some(total as f64 / count as f64),
            _ => None,
        }
    }

    /// 计算总体质量平均分
    pub fn avg_overall_quality(&self) -> Option<f64> {
        match (self.overall_quality_total, self.overall_quality_count) {
            (Some(total), Some(count)) if count > 0 => Some(total as f64 / count as f64),
            _ => None,
        }
    }

    /// 计算参考答案质量平均分
    pub fn avg_answer_quality(&self) -> Option<f64> {
        match (self.answer_quality_total, self.answer_quality_count) {
            (Some(total), Some(count)) if count > 0 => Some(total as f64 / count as f64),
            _ => None,
        }
    }

    /// 计算格式质量平均分
    pub fn avg_format_quality(&self) -> Option<f64> {
        match (self.format_quality_total, self.format_quality_count) {
            (Some(total), Some(count)) if count > 0 => Some(total as f64 / count as f64),
            _ => None,
        }
    }

    /// 计算知识点详细程度平均分
    pub fn avg_detail_level(&self) -> Option<f64> {
        match (self.detail_level_total, self.detail_level_count) {
            (Some(total), Some(count)) if count > 0 => Some(total as f64 / count as f64),
            _ => None,
        }
    }

    /// 获取总评分人数（取各维度中的最大值，因为每个维度都是一起评分的）
    pub fn rating_count(&self) -> i64 {
        let counts = [
            self.difficulty_count.unwrap_or(0),
            self.overall_quality_count.unwrap_or(0),
            self.answer_quality_count.unwrap_or(0),
            self.format_quality_count.unwrap_or(0),
            self.detail_level_count.unwrap_or(0),
        ];
        counts.iter().max().copied().unwrap_or(0)
    }
}

/// 评分维度信息（用于前端展示）
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RatingDimension {
    pub key: String,
    pub name: String,
    pub description: String,
    pub avg_score: Option<f64>,
}

/// 资源评分信息响应（用于资源详情页）
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceRatingInfo {
    pub resource_id: Uuid,
    pub rating_count: i64,
    pub dimensions: Vec<RatingDimension>,
    pub user_rating: Option<RatingResponse>,
}
