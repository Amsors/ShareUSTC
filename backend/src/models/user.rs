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
    pub sn: Option<i64>,
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
    pub sn: Option<i64>,
    pub username: String,
    pub email: Option<String>,
    pub role: String,
    pub bio: Option<String>,
    pub is_verified: bool,
    pub created_at: NaiveDateTime,
}

impl From<User> for UserInfo {
    fn from(user: User) -> Self {
        UserInfo {
            id: user.id,
            sn: user.sn,
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
        if !self
            .username
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
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

/// 修改密码请求 DTO
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

impl ChangePasswordRequest {
    /// 验证修改密码请求
    pub fn validate(&self) -> Result<(), String> {
        // 验证旧密码不为空
        if self.old_password.is_empty() {
            return Err("原密码不能为空".to_string());
        }

        // 验证新密码长度
        if self.new_password.len() < 6 {
            return Err("新密码长度至少为6个字符".to_string());
        }

        // 验证新密码不能与旧密码相同
        if self.old_password == self.new_password {
            return Err("新密码不能与原密码相同".to_string());
        }

        Ok(())
    }
}

/// 用户资料响应（公开信息）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfileResponse {
    pub id: Uuid,
    pub sn: Option<i64>,
    pub username: String,
    pub bio: Option<String>,
    pub role: String,
    pub is_verified: bool,
    pub created_at: NaiveDateTime,
    pub uploads_count: i64,
    pub total_likes: i64,
    pub total_downloads: i64,
}

/// 用户主页响应（包含资源列表和统计数据）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserHomepageResponse {
    pub id: Uuid,
    pub sn: Option<i64>,
    pub username: String,
    pub bio: Option<String>,
    pub email: Option<String>,
    pub role: String,
    pub is_verified: bool,
    pub created_at: NaiveDateTime,
    pub uploads_count: i64,
    pub total_likes: i64,
    pub total_downloads: i64,
    pub resources: Vec<crate::models::resource::ResourceListItem>,
    pub resources_total: i64,
}

/// 用户主页查询参数
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserHomepageQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

impl UserHomepageQuery {
    pub fn get_page(&self) -> i32 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn get_per_page(&self) -> i32 {
        self.per_page.unwrap_or(10).min(50).max(1)
    }
}

/// 贡献榜单查询参数
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardQuery {
    pub limit: Option<i32>,
}

impl LeaderboardQuery {
    pub fn get_limit(&self) -> i32 {
        self.limit.unwrap_or(50).min(100).max(1)
    }
}

/// 贡献榜单用户信息
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardUser {
    pub id: Uuid,
    pub sn: Option<i64>,
    pub username: String,
    pub bio: Option<String>,
    pub role: String,
    pub is_verified: bool,
    pub uploads_count: i64,
    pub total_likes: i64,
    pub total_downloads: i64,
}

/// 贡献榜单响应
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardResponse {
    pub users: Vec<LeaderboardUser>,
    pub total: i64,
}

impl UpdateProfileRequest {
    /// 验证更新请求
    pub fn validate(&self) -> Result<(), String> {
        // 如果提供了用户名，验证格式
        if let Some(username) = &self.username {
            if username.len() < 3 || username.len() > 50 {
                return Err("用户名长度必须在3-50个字符之间".to_string());
            }
            if !username
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
            {
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

#[cfg(test)]
mod tests {
    use super::*;

    mod register_request_tests {
        use super::*;

        fn create_register_request(
            username: &str,
            password: &str,
            email: Option<&str>,
        ) -> RegisterRequest {
            RegisterRequest {
                username: username.to_string(),
                password: password.to_string(),
                email: email.map(|s| s.to_string()),
            }
        }

        #[test]
        fn test_valid_register_request() {
            let req = create_register_request("test_user", "password123", Some("test@example.com"));
            assert!(req.validate().is_ok());
        }

        #[test]
        fn test_valid_register_request_without_email() {
            let req = create_register_request("test_user", "password123", None);
            assert!(req.validate().is_ok());
        }

        #[test]
        fn test_username_too_short() {
            let req = create_register_request("ab", "password123", None);
            assert!(req.validate().is_err());
            assert!(req.validate().unwrap_err().contains("用户名长度"));
        }

        #[test]
        fn test_username_too_long() {
            let req = create_register_request("a".repeat(51).as_str(), "password123", None);
            assert!(req.validate().is_err());
            assert!(req.validate().unwrap_err().contains("用户名长度"));
        }

        #[test]
        fn test_username_with_invalid_chars() {
            let req = create_register_request("test-user", "password123", None);
            assert!(req.validate().is_err());
            assert!(req.validate().unwrap_err().contains("用户名只能包含"));
        }

        #[test]
        fn test_username_with_chinese() {
            let req = create_register_request("测试用户", "password123", None);
            assert!(req.validate().is_err());
            assert!(req.validate().unwrap_err().contains("用户名只能包含"));
        }

        #[test]
        fn test_password_too_short() {
            let req = create_register_request("test_user", "12345", None);
            assert!(req.validate().is_err());
            assert!(req.validate().unwrap_err().contains("密码长度"));
        }

        #[test]
        fn test_invalid_email() {
            let req = create_register_request("test_user", "password123", Some("invalid-email"));
            assert!(req.validate().is_err());
            assert!(req.validate().unwrap_err().contains("邮箱格式"));
        }

        #[test]
        fn test_valid_username_with_underscore() {
            let req = create_register_request("test_user_123", "password123", None);
            assert!(req.validate().is_ok());
        }

        #[test]
        fn test_valid_username_alphanumeric() {
            let req = create_register_request("user123", "password123", None);
            assert!(req.validate().is_ok());
        }
    }

    mod login_request_tests {
        use super::*;

        fn create_login_request(username: &str, password: &str) -> LoginRequest {
            LoginRequest {
                username: username.to_string(),
                password: password.to_string(),
            }
        }

        #[test]
        fn test_valid_login_request() {
            let req = create_login_request("test_user", "password123");
            assert!(req.validate().is_ok());
        }

        #[test]
        fn test_empty_username() {
            let req = create_login_request("", "password123");
            assert!(req.validate().is_err());
            assert!(req.validate().unwrap_err().contains("用户名不能为空"));
        }

        #[test]
        fn test_empty_password() {
            let req = create_login_request("test_user", "");
            assert!(req.validate().is_err());
            assert!(req.validate().unwrap_err().contains("密码不能为空"));
        }

        #[test]
        fn test_both_empty() {
            let req = create_login_request("", "");
            assert!(req.validate().is_err());
        }
    }

    mod user_role_tests {
        use super::*;

        #[test]
        fn test_user_role_default() {
            let role = UserRole::default();
            assert_eq!(role, UserRole::User);
        }

        #[test]
        fn test_user_role_to_string() {
            assert_eq!(UserRole::Guest.to_string(), "guest");
            assert_eq!(UserRole::User.to_string(), "user");
            assert_eq!(UserRole::Verified.to_string(), "verified");
            assert_eq!(UserRole::Admin.to_string(), "admin");
        }

        #[test]
        fn test_user_role_equality() {
            assert_eq!(UserRole::Admin, UserRole::Admin);
            assert_ne!(UserRole::Admin, UserRole::User);
        }
    }

    mod update_profile_request_tests {
        use super::*;
        use serde_json::json;

        #[test]
        fn test_valid_update_profile_request() {
            let req = UpdateProfileRequest {
                username: Some("new_username".to_string()),
                bio: Some("This is my bio".to_string()),
                email: Some("new@example.com".to_string()),
                social_links: Some(json!({"twitter": "@test"})),
            };
            assert!(req.validate().is_ok());
        }

        #[test]
        fn test_empty_update_profile_request() {
            let req = UpdateProfileRequest {
                username: None,
                bio: None,
                email: None,
                social_links: None,
            };
            assert!(req.validate().is_ok());
        }

        #[test]
        fn test_username_too_short_in_update() {
            let req = UpdateProfileRequest {
                username: Some("ab".to_string()),
                bio: None,
                email: None,
                social_links: None,
            };
            assert!(req.validate().is_err());
        }

        #[test]
        fn test_username_with_invalid_chars_in_update() {
            let req = UpdateProfileRequest {
                username: Some("test-user".to_string()),
                bio: None,
                email: None,
                social_links: None,
            };
            assert!(req.validate().is_err());
        }

        #[test]
        fn test_invalid_email_in_update() {
            let req = UpdateProfileRequest {
                username: None,
                bio: None,
                email: Some("invalid-email".to_string()),
                social_links: None,
            };
            assert!(req.validate().is_err());
        }

        #[test]
        fn test_empty_email_is_valid() {
            let req = UpdateProfileRequest {
                username: None,
                bio: None,
                email: Some("".to_string()),
                social_links: None,
            };
            assert!(req.validate().is_ok());
        }

        #[test]
        fn test_bio_too_long() {
            let req = UpdateProfileRequest {
                username: None,
                bio: Some("a".repeat(501)),
                email: None,
                social_links: None,
            };
            assert!(req.validate().is_err());
            assert!(req.validate().unwrap_err().contains("签名长度"));
        }

        #[test]
        fn test_bio_max_length() {
            let req = UpdateProfileRequest {
                username: None,
                bio: Some("a".repeat(500)),
                email: None,
                social_links: None,
            };
            assert!(req.validate().is_ok());
        }
    }

    mod user_homepage_query_tests {
        use super::*;

        #[test]
        fn test_default_page() {
            let query = UserHomepageQuery {
                page: None,
                per_page: None,
            };
            assert_eq!(query.get_page(), 1);
        }

        #[test]
        fn test_custom_page() {
            let query = UserHomepageQuery {
                page: Some(5),
                per_page: None,
            };
            assert_eq!(query.get_page(), 5);
        }

        #[test]
        fn test_page_less_than_one() {
            let query = UserHomepageQuery {
                page: Some(0),
                per_page: None,
            };
            assert_eq!(query.get_page(), 1);

            let query = UserHomepageQuery {
                page: Some(-5),
                per_page: None,
            };
            assert_eq!(query.get_page(), 1);
        }

        #[test]
        fn test_default_per_page() {
            let query = UserHomepageQuery {
                page: None,
                per_page: None,
            };
            assert_eq!(query.get_per_page(), 10);
        }

        #[test]
        fn test_custom_per_page() {
            let query = UserHomepageQuery {
                page: None,
                per_page: Some(20),
            };
            assert_eq!(query.get_per_page(), 20);
        }

        #[test]
        fn test_per_page_too_high() {
            let query = UserHomepageQuery {
                page: None,
                per_page: Some(100),
            };
            assert_eq!(query.get_per_page(), 50);
        }

        #[test]
        fn test_per_page_less_than_one() {
            let query = UserHomepageQuery {
                page: None,
                per_page: Some(0),
            };
            assert_eq!(query.get_per_page(), 1);

            let query = UserHomepageQuery {
                page: None,
                per_page: Some(-5),
            };
            assert_eq!(query.get_per_page(), 1);
        }
    }
}
