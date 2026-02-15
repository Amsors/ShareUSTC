use crate::models::{
    UpdateProfileRequest, User, UserInfo, UserProfileResponse, VerificationRequest,
    UserHomepageResponse, UserHomepageQuery,
};
use crate::models::resource::{ResourceListItem, ResourceStatsResponse};
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// 用户服务错误类型
#[derive(Debug)]
pub enum UserError {
    UserNotFound(String),
    UserExists(String),
    DatabaseError(String),
    ValidationError(String),
    #[allow(dead_code)]
    Forbidden(String),
}

impl std::fmt::Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserError::UserNotFound(msg) => write!(f, "用户不存在: {}", msg),
            UserError::UserExists(msg) => write!(f, "用户已存在: {}", msg),
            UserError::DatabaseError(msg) => write!(f, "数据库错误: {}", msg),
            UserError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
            UserError::Forbidden(msg) => write!(f, "没有权限: {}", msg),
        }
    }
}

impl std::error::Error for UserError {}

/// 用户服务
pub struct UserService;

impl UserService {
    /// 获取当前用户信息
    pub async fn get_current_user(pool: &PgPool, user_id: Uuid) -> Result<UserInfo, UserError> {
        let user: User = sqlx::query_as::<_, User>(
            "SELECT id, sn, username, password_hash, email, role, bio,
                    CASE WHEN social_links = '{}'::jsonb THEN NULL ELSE social_links END as social_links,
                    CASE WHEN real_info = '{}'::jsonb THEN NULL ELSE real_info END as real_info,
                    is_verified, is_active, created_at, updated_at
             FROM users WHERE id = $1 AND is_active = true"
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?
        .ok_or_else(|| UserError::UserNotFound("用户不存在".to_string()))?;

        Ok(UserInfo::from(user))
    }

    /// 获取用户公开资料
    pub async fn get_user_profile(pool: &PgPool, user_id: Uuid) -> Result<UserProfileResponse, UserError> {
        // 获取用户基本信息
        let user: User = sqlx::query_as::<_, User>(
            "SELECT id, sn, username, password_hash, email, role, bio,
                    CASE WHEN social_links = '{}'::jsonb THEN NULL ELSE social_links END as social_links,
                    CASE WHEN real_info = '{}'::jsonb THEN NULL ELSE real_info END as real_info,
                    is_verified, is_active, created_at, updated_at
             FROM users WHERE id = $1 AND is_active = true"
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?
        .ok_or_else(|| UserError::UserNotFound("用户不存在".to_string()))?;

        // 获取用户上传的资源数量
        let uploads_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM resources WHERE uploader_id = $1"
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        // 获取总点赞数
        let total_likes: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(likes), 0) FROM resource_stats rs
             JOIN resources r ON rs.resource_id = r.id
             WHERE r.uploader_id = $1"
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        // 获取总下载数
        let total_downloads: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(downloads), 0) FROM resource_stats rs
             JOIN resources r ON rs.resource_id = r.id
             WHERE r.uploader_id = $1"
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        Ok(UserProfileResponse {
            id: user.id,
            sn: user.sn,
            username: user.username,
            bio: user.bio,
            role: user.role,
            is_verified: user.is_verified,
            created_at: user.created_at,
            uploads_count,
            total_likes,
            total_downloads,
        })
    }

    /// 更新用户资料
    ///
    /// # Arguments
    /// * `pool` - 数据库连接池
    /// * `user_id` - 用户ID
    /// * `req` - 更新请求
    /// * `is_verified` - 是否已实名认证，未实名用户不能修改个人简介
    pub async fn update_profile(
        pool: &PgPool,
        user_id: Uuid,
        req: UpdateProfileRequest,
        is_verified: bool,
    ) -> Result<UserInfo, UserError> {
        // 验证请求
        req.validate()
            .map_err(|e| UserError::ValidationError(e))?;

        // 检查用户名是否已被使用
        if let Some(ref username) = req.username {
            let existing: Option<(Uuid,)> = sqlx::query_as(
                "SELECT id FROM users WHERE username = $1 AND id != $2 AND is_active = true"
            )
            .bind(username)
            .bind(user_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

            if existing.is_some() {
                return Err(UserError::UserExists(
                    "用户名已被使用".to_string()
                ));
            }
        }

        // 获取当前用户信息，然后更新
        let current_user = Self::get_current_user(pool, user_id).await?;

        // 构建更新查询 - 使用简单的字符串拼接
        let username = req.username.unwrap_or(current_user.username);
        // 未实名用户不能修改个人简介，保持原有值
        let bio = if is_verified {
            req.bio.or(current_user.bio)
        } else {
            current_user.bio
        };
        let email = req.email.or(current_user.email);
        let social_links = req.social_links;

        let updated_user: User = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET username = $1,
                bio = $2,
                email = $3,
                social_links = $4,
                updated_at = NOW()
            WHERE id = $5 AND is_active = true
            RETURNING id, sn, username, password_hash, email, role, bio, social_links, real_info, is_verified, is_active, created_at, updated_at
            "#
        )
        .bind(username)
        .bind(bio)
        .bind(email)
        .bind(social_links)
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| UserError::DatabaseError(format!("更新失败: {}", e)))?
        .ok_or_else(|| UserError::UserNotFound("用户不存在".to_string()))?;

        log::info!("用户资料已更新: {}", updated_user.username);

        Ok(UserInfo::from(updated_user))
    }

    /// 实名认证
    pub async fn verify_user(
        pool: &PgPool,
        user_id: Uuid,
        req: VerificationRequest,
    ) -> Result<UserInfo, UserError> {
        // 获取当前用户
        let user: User = sqlx::query_as::<_, User>(
            "SELECT id, sn, username, password_hash, email, role, bio,
                    CASE WHEN social_links = '{}'::jsonb THEN NULL ELSE social_links END as social_links,
                    CASE WHEN real_info = '{}'::jsonb THEN NULL ELSE real_info END as real_info,
                    is_verified, is_active, created_at, updated_at
             FROM users WHERE id = $1 AND is_active = true"
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?
        .ok_or_else(|| UserError::UserNotFound("用户不存在".to_string()))?;

        // 检查是否已经是实名用户
        if user.is_verified {
            return Err(UserError::ValidationError("用户已完成实名认证".to_string()));
        }

        // 构建实名信息 JSON
        let real_info = serde_json::json!({
            "real_name": req.real_name,
            "student_id": req.student_id,
            "major": req.major,
            "grade": req.grade,
        });

        // 更新用户为实名状态（保持原有角色，只更新 is_verified）
        let updated_user: User = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET is_verified = true,
                real_info = $1,
                updated_at = NOW()
            WHERE id = $2 AND is_active = true
            RETURNING id, sn, username, password_hash, email, role, bio, social_links, real_info, is_verified, is_active, created_at, updated_at
            "#
        )
        .bind(real_info)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| UserError::DatabaseError(format!("认证失败: {}", e)))?;

        log::info!("用户完成实名认证: {}", updated_user.username);

        Ok(UserInfo::from(updated_user))
    }

    /// 获取用户主页数据（公开接口）
    /// 包含用户基本信息、统计数据和已通过审核的资源列表
    pub async fn get_user_homepage(
        pool: &PgPool,
        user_id: Uuid,
        query: &UserHomepageQuery,
    ) -> Result<UserHomepageResponse, UserError> {
        // 获取用户基本信息
        let user: User = sqlx::query_as::<_, User>(
            "SELECT id, sn, username, password_hash, email, role, bio,
                    CASE WHEN social_links = '{}'::jsonb THEN NULL ELSE social_links END as social_links,
                    CASE WHEN real_info = '{}'::jsonb THEN NULL ELSE real_info END as real_info,
                    is_verified, is_active, created_at, updated_at
             FROM users WHERE id = $1 AND is_active = true"
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?
        .ok_or_else(|| UserError::UserNotFound("用户不存在".to_string()))?;

        // 获取用户上传的已通过审核的资源数量
        let uploads_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM resources WHERE uploader_id = $1 AND audit_status = 'approved'"
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        // 获取总点赞数
        let total_likes: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(likes), 0) FROM resource_stats rs
             JOIN resources r ON rs.resource_id = r.id
             WHERE r.uploader_id = $1 AND r.audit_status = 'approved'"
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        // 获取总下载数
        let total_downloads: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(downloads), 0) FROM resource_stats rs
             JOIN resources r ON rs.resource_id = r.id
             WHERE r.uploader_id = $1 AND r.audit_status = 'approved'"
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        // 获取分页参数
        let page = query.get_page();
        let per_page = query.get_per_page();
        let offset = (page - 1) * per_page;

        // 获取用户上传的已通过审核资源列表
        let rows = sqlx::query(
            r#"
            SELECT r.*, rs.views, rs.downloads, rs.likes,
                   rs.difficulty_total, rs.difficulty_count,
                   rs.overall_quality_total, rs.overall_quality_count,
                   rs.answer_quality_total, rs.answer_quality_count,
                   rs.format_quality_total, rs.format_quality_count,
                   rs.detail_level_total, rs.detail_level_count,
                   u.username as uploader_name
            FROM resources r
            LEFT JOIN resource_stats rs ON r.id = rs.resource_id
            LEFT JOIN users u ON r.uploader_id = u.id
            WHERE r.uploader_id = $1 AND r.audit_status = 'approved'
            ORDER BY r.created_at DESC
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(user_id)
        .bind(per_page as i64)
        .bind(offset as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        // 映射资源列表
        let mut resources = Vec::new();
        for row in rows {
            let tags_json: Option<serde_json::Value> = row.try_get("tags").ok();
            let tags: Option<Vec<String>> = tags_json.and_then(|t| {
                serde_json::from_value::<Vec<String>>(t).ok()
            });

            // 计算各维度的平均分
            let calc_avg = |total: Option<i32>, count: Option<i32>| -> Option<f64> {
                match (total, count) {
                    (Some(t), Some(c)) if c > 0 => Some(t as f64 / c as f64),
                    _ => None,
                }
            };

            let avg_difficulty = calc_avg(
                row.try_get::<i32, _>("difficulty_total").ok(),
                row.try_get::<i32, _>("difficulty_count").ok(),
            );
            let avg_overall_quality = calc_avg(
                row.try_get::<i32, _>("overall_quality_total").ok(),
                row.try_get::<i32, _>("overall_quality_count").ok(),
            );
            let avg_answer_quality = calc_avg(
                row.try_get::<i32, _>("answer_quality_total").ok(),
                row.try_get::<i32, _>("answer_quality_count").ok(),
            );
            let avg_format_quality = calc_avg(
                row.try_get::<i32, _>("format_quality_total").ok(),
                row.try_get::<i32, _>("format_quality_count").ok(),
            );
            let avg_detail_level = calc_avg(
                row.try_get::<i32, _>("detail_level_total").ok(),
                row.try_get::<i32, _>("detail_level_count").ok(),
            );

            // 评分人数取各维度中的最大值
            let rating_count: i32 = [
                row.try_get::<i32, _>("difficulty_count").unwrap_or(0),
                row.try_get::<i32, _>("overall_quality_count").unwrap_or(0),
                row.try_get::<i32, _>("answer_quality_count").unwrap_or(0),
                row.try_get::<i32, _>("format_quality_count").unwrap_or(0),
                row.try_get::<i32, _>("detail_level_count").unwrap_or(0),
            ].iter().max().copied().unwrap_or(0);

            resources.push(ResourceListItem {
                id: row.try_get("id").map_err(|e| UserError::DatabaseError(e.to_string()))?,
                title: row.try_get("title").map_err(|e| UserError::DatabaseError(e.to_string()))?,
                course_name: row.try_get("course_name").ok(),
                resource_type: row.try_get("resource_type").map_err(|e| UserError::DatabaseError(e.to_string()))?,
                category: row.try_get("category").map_err(|e| UserError::DatabaseError(e.to_string()))?,
                tags,
                audit_status: row.try_get("audit_status").map_err(|e| UserError::DatabaseError(e.to_string()))?,
                created_at: row.try_get("created_at").map_err(|e| UserError::DatabaseError(e.to_string()))?,
                stats: ResourceStatsResponse {
                    views: row.try_get::<i32, _>("views").unwrap_or(0),
                    downloads: row.try_get::<i32, _>("downloads").unwrap_or(0),
                    likes: row.try_get::<i32, _>("likes").unwrap_or(0),
                    avg_difficulty,
                    avg_overall_quality,
                    avg_answer_quality,
                    avg_format_quality,
                    avg_detail_level,
                    rating_count,
                },
                uploader_name: row.try_get("uploader_name").ok(),
            });
        }

        Ok(UserHomepageResponse {
            id: user.id,
            sn: user.sn,
            username: user.username,
            bio: user.bio,
            email: user.email,
            role: user.role,
            is_verified: user.is_verified,
            created_at: user.created_at,
            uploads_count,
            total_likes,
            total_downloads,
            resources,
            resources_total: uploads_count,
        })
    }
}
