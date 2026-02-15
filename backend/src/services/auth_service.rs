use crate::models::{
    AuthResponse, LoginRequest, RegisterRequest, TokenResponse,
    UserInfo, UserRole, User,
};
use crate::utils::{generate_access_token, generate_refresh_token, hash_password, verify_password, verify_token};
use sqlx::PgPool;
use uuid::Uuid;

/// 认证错误类型
#[derive(Debug)]
pub enum AuthError {
    InvalidCredentials(String),
    UserExists(String),
    #[allow(dead_code)]
    UserNotFound(String),
    TokenInvalid(String),
    DatabaseError(String),
    ValidationError(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::InvalidCredentials(msg) => write!(f, "认证失败: {}", msg),
            AuthError::UserExists(msg) => write!(f, "用户已存在: {}", msg),
            AuthError::UserNotFound(msg) => write!(f, "用户不存在: {}", msg),
            AuthError::TokenInvalid(msg) => write!(f, "Token无效: {}", msg),
            AuthError::DatabaseError(msg) => write!(f, "数据库错误: {}", msg),
            AuthError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
        }
    }
}

impl std::error::Error for AuthError {}

/// 认证服务
pub struct AuthService;

impl AuthService {
    /// 用户注册
    pub async fn register(
        pool: &PgPool,
        jwt_secret: &str,
        req: RegisterRequest,
    ) -> Result<AuthResponse, AuthError> {
        // 验证请求
        req.validate()
            .map_err(|e| AuthError::ValidationError(e))?;

        // 检查用户名是否已存在
        let existing_user: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM users WHERE username = $1 AND is_active = true"
        )
        .bind(&req.username)
        .fetch_optional(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        if existing_user.is_some() {
            return Err(AuthError::UserExists(
                "用户名已被使用".to_string()
            ));
        }

        // 哈希密码
        let password_hash = hash_password(&req.password)
            .map_err(|e| AuthError::ValidationError(e))?;

        // 创建用户
        let user_id = Uuid::new_v4();
        // 检查用户名是否在ADMIN_USERNAMES环境变量列表中，如果是则赋予admin角色
        let admin_usernames = std::env::var("ADMIN_USERNAMES").unwrap_or_default();
        let admin_list: Vec<String> = admin_usernames
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let role = if admin_list.contains(&req.username) {
            "admin"
        } else {
            "user"
        };

        // 获取下一个 sn 值
        let sn: i64 = sqlx::query_scalar("SELECT nextval('user_sn_seq')")
            .fetch_one(pool)
            .await
            .map_err(|e| AuthError::DatabaseError(format!("获取用户编号失败: {}", e)))?;

        sqlx::query(
            r#"
            INSERT INTO users (id, sn, username, password_hash, email, role, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, true, NOW(), NOW())
            "#
        )
        .bind(user_id)
        .bind(sn)
        .bind(&req.username)
        .bind(&password_hash)
        .bind(&req.email)
        .bind(role)
        .execute(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(format!("创建用户失败: {}", e)))?;

        log::info!("用户注册成功: {}, 角色: {}", req.username, role);

        // 解析角色用于Token生成
        let user_role = match role {
            "admin" => UserRole::Admin,
            "verified" => UserRole::Verified,
            "user" => UserRole::User,
            _ => UserRole::Guest,
        };

        // 生成 Token
        let access_token = generate_access_token(
            user_id,
            req.username.clone(),
            user_role.clone(),
            false, // 新注册用户默认未实名认证
            jwt_secret,
        )
        .map_err(|e| AuthError::TokenInvalid(e))?;

        let refresh_token = generate_refresh_token(
            user_id,
            req.username.clone(),
            user_role,
            false,
            jwt_secret,
        )
        .map_err(|e| AuthError::TokenInvalid(e))?;

        Ok(AuthResponse {
            user: UserInfo {
                id: user_id,
                sn: Some(sn),
                username: req.username,
                email: req.email,
                role: role.to_string(),
                bio: None,
                is_verified: false,
                created_at: chrono::Local::now().naive_local(),
            },
            tokens: TokenResponse {
                access_token,
                refresh_token,
                token_type: "Bearer".to_string(),
                expires_in: 15 * 60, // 15分钟
            },
        })
    }

    /// 用户登录
    pub async fn login(
        pool: &PgPool,
        jwt_secret: &str,
        req: LoginRequest,
    ) -> Result<AuthResponse, AuthError> {
        // 验证请求
        req.validate()
            .map_err(|e| AuthError::ValidationError(e))?;

        // 查询用户
        let user: User = sqlx::query_as::<_, User>(
            "SELECT id, sn, username, password_hash, email, role, bio,
                    CASE WHEN social_links = '{}'::jsonb THEN NULL ELSE social_links END as social_links,
                    CASE WHEN real_info = '{}'::jsonb THEN NULL ELSE real_info END as real_info,
                    is_verified, is_active, created_at, updated_at
             FROM users WHERE username = $1 AND is_active = true"
        )
        .bind(&req.username)
        .fetch_optional(pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AuthError::InvalidCredentials("用户名或密码错误".to_string()))?;

        // 验证密码
        let valid = verify_password(&req.password, &user.password_hash)
            .map_err(|_| AuthError::InvalidCredentials("用户名或密码错误".to_string()))?;

        if !valid {
            log::warn!("登录失败，密码错误: {}", req.username);
            return Err(AuthError::InvalidCredentials(
                "用户名或密码错误".to_string()
            ));
        }

        log::info!("用户登录成功: {}, 角色: {}", req.username, user.role);

        // 解析角色
        let role = match user.role.as_str() {
            "admin" => {
                log::info!("用户 {} 以管理员身份登录", req.username);
                UserRole::Admin
            }
            "verified" => UserRole::Verified,
            "user" => UserRole::User,
            _ => UserRole::Guest,
        };

        // 生成 Token
        let access_token = generate_access_token(
            user.id,
            user.username.clone(),
            role.clone(),
            user.is_verified,
            jwt_secret,
        )
        .map_err(|e| AuthError::TokenInvalid(e))?;

        let refresh_token = generate_refresh_token(
            user.id,
            user.username.clone(),
            role.clone(),
            user.is_verified,
            jwt_secret,
        )
        .map_err(|e| AuthError::TokenInvalid(e))?;

        Ok(AuthResponse {
            user: UserInfo {
                id: user.id,
                sn: user.sn,
                username: user.username,
                email: user.email,
                role: user.role,
                bio: user.bio,
                is_verified: user.is_verified,
                created_at: user.created_at,
            },
            tokens: TokenResponse {
                access_token,
                refresh_token,
                token_type: "Bearer".to_string(),
                expires_in: 15 * 60, // 15分钟
            },
        })
    }

    /// 刷新 Token
    pub async fn refresh_token(
        _pool: &PgPool,
        jwt_secret: &str,
        refresh_token: String,
    ) -> Result<TokenResponse, AuthError> {
        // 验证 Refresh Token
        let claims = verify_token(&refresh_token, jwt_secret, Some("refresh"))
            .map_err(|e| AuthError::TokenInvalid(e))?;

        // 提取用户信息
        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AuthError::TokenInvalid("无效的用户ID".to_string()))?;

        let role = match claims.role.as_str() {
            "admin" => UserRole::Admin,
            "verified" => UserRole::Verified,
            "user" => UserRole::User,
            _ => UserRole::Guest,
        };

        log::info!("刷新 Token: {}", claims.username);

        // 生成新的 Token 对
        let access_token = generate_access_token(
            user_id,
            claims.username.clone(),
            role.clone(),
            claims.is_verified,
            jwt_secret,
        )
        .map_err(|e| AuthError::TokenInvalid(e))?;

        let refresh_token = generate_refresh_token(
            user_id,
            claims.username,
            role,
            claims.is_verified,
            jwt_secret,
        )
        .map_err(|e| AuthError::TokenInvalid(e))?;

        Ok(TokenResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 15 * 60, // 15分钟
        })
    }
}
