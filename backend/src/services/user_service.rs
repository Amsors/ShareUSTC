use crate::models::{
    UpdateProfileRequest, User, UserInfo, UserProfileResponse, VerificationRequest,
};
use sqlx::PgPool;
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
            "SELECT id, username, password_hash, email, role, bio,
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
            "SELECT id, username, password_hash, email, role, bio,
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
    pub async fn update_profile(
        pool: &PgPool,
        user_id: Uuid,
        req: UpdateProfileRequest,
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
        let bio = req.bio.or(current_user.bio);
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
            RETURNING id, username, password_hash, email, role, bio, social_links, real_info, is_verified, is_active, created_at, updated_at
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
            "SELECT id, username, password_hash, email, role, bio,
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

        // 更新用户为实名状态
        let updated_user: User = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET role = 'verified',
                is_verified = true,
                real_info = $1,
                updated_at = NOW()
            WHERE id = $2 AND is_active = true
            RETURNING id, username, password_hash, email, role, bio, social_links, real_info, is_verified, is_active, created_at, updated_at
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
}
