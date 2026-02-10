use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 用户角色枚举
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    /// 游客（未登录）
    Guest,
    /// 注册用户
    User,
    /// 实名用户
    Verified,
    /// 管理员
    Admin,
}

impl Default for UserRole {
    fn default() -> Self {
        UserRole::User
    }
}

impl ToString for UserRole {
    fn to_string(&self) -> String {
        match self {
            UserRole::Guest => "guest".to_string(),
            UserRole::User => "user".to_string(),
            UserRole::Verified => "verified".to_string(),
            UserRole::Admin => "admin".to_string(),
        }
    }
}

/// 用户结构体（对应数据库 users 表）
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub email: Option<String>,
    pub role: String,
    pub bio: Option<String>,
    pub social_links: Option<serde_json::Value>,
    pub real_info: Option<serde_json::Value>,
    pub is_verified: bool,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// 简化用户信息（用于返回给前端）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub role: String,
    pub bio: Option<String>,
    #[serde(rename = "isVerified")]
    pub is_verified: bool,
    #[serde(rename = "createdAt")]
    pub created_at: NaiveDateTime,
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        UserInfo {
            id: user.id,
            username: user.username,
            email: user.email,
            role: user.role,
            bio: user.bio,
            is_verified: user.is_verified,
            created_at: user.created_at,
        }
    }
}

/// 注册请求 DTO
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
}

/// 登录请求 DTO
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Token 响应 DTO
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// 认证响应 DTO
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    pub user: UserInfo,
    pub tokens: TokenResponse,
}

/// 刷新 Token 请求 DTO
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // 用户ID
    pub username: String,   // 用户名
    pub role: String,       // 角色
    pub is_verified: bool,  // 是否实名认证
    pub exp: i64,           // 过期时间
    pub iat: i64,           // 签发时间
    pub token_type: String, // token 类型: access | refresh
}

/// 当前用户信息（从 JWT 中提取）
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: Uuid,
    pub username: String,
    pub role: UserRole,
    pub is_verified: bool,
}

impl RegisterRequest {
    /// 验证注册请求
    pub fn validate(&self) -> Result<(), String> {
        // 用户名验证：3-50字符，只允许字母数字下划线
        if self.username.len() < 3 || self.username.len() > 50 {
            return Err("用户名长度必须在3-50个字符之间".to_string());
        }
        if !self.username.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err("用户名只能包含字母、数字和下划线".to_string());
        }

        // 密码验证：至少6个字符
        if self.password.len() < 6 {
            return Err("密码长度至少为6个字符".to_string());
        }

        // 邮箱验证（如果提供）
        if let Some(email) = &self.email {
            if !email.contains('@') {
                return Err("邮箱格式不正确".to_string());
            }
        }

        Ok(())
    }
}

impl LoginRequest {
    /// 验证登录请求
    pub fn validate(&self) -> Result<(), String> {
        if self.username.is_empty() {
            return Err("用户名不能为空".to_string());
        }
        if self.password.is_empty() {
            return Err("密码不能为空".to_string());
        }
        Ok(())
    }
}

/// 更新用户资料请求 DTO
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProfileRequest {
    pub username: Option<String>,
    pub bio: Option<String>,
    pub email: Option<String>,
    pub social_links: Option<serde_json::Value>,
}

/// 实名认证请求 DTO
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationRequest {
    pub real_name: Option<String>,
    pub student_id: Option<String>,
    pub major: Option<String>,
    pub grade: Option<String>,
}

/// 用户资料响应（公开信息）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfileResponse {
    pub id: Uuid,
    pub username: String,
    pub bio: Option<String>,
    pub role: String,
    pub is_verified: bool,
    pub created_at: NaiveDateTime,
    pub uploads_count: i64,
    pub total_likes: i64,
    pub total_downloads: i64,
}

impl UpdateProfileRequest {
    /// 验证更新请求
    pub fn validate(&self) -> Result<(), String> {
        // 如果提供了用户名，验证格式
        if let Some(username) = &self.username {
            if username.len() < 3 || username.len() > 50 {
                return Err("用户名长度必须在3-50个字符之间".to_string());
            }
            if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Err("用户名只能包含字母、数字和下划线".to_string());
            }
        }

        // 如果提供了邮箱，验证格式
        if let Some(email) = &self.email {
            if !email.is_empty() && !email.contains('@') {
                return Err("邮箱格式不正确".to_string());
            }
        }

        // 如果提供了签名，验证长度
        if let Some(bio) = &self.bio {
            if bio.len() > 500 {
                return Err("签名长度不能超过500个字符".to_string());
            }
        }

        Ok(())
    }
}
