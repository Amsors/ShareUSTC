use crate::models::{Claims, CurrentUser, UserRole};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use uuid::Uuid;

/// Access Token 有效期：60分钟（1小时）
const ACCESS_TOKEN_EXPIRE_MINUTES: i64 = 60;
/// Refresh Token 有效期：7天
const REFRESH_TOKEN_EXPIRE_DAYS: i64 = 7;

/// 生成 Access Token
///
/// # Arguments
/// * `user_id` - 用户ID
/// * `username` - 用户名
/// * `role` - 用户角色
/// * `is_verified` - 是否实名认证
/// * `secret` - JWT密钥
///
/// # Returns
/// * `Ok(String)` - JWT Token
/// * `Err(String)` - 错误信息
pub fn generate_access_token(
    user_id: Uuid,
    username: String,
    role: UserRole,
    is_verified: bool,
    secret: &str,
) -> Result<String, String> {
    let now = Utc::now();
    let exp = now + Duration::minutes(ACCESS_TOKEN_EXPIRE_MINUTES);

    let claims = Claims {
        sub: user_id.to_string(),
        username,
        role: role.to_string(),
        is_verified,
        exp: exp.timestamp(),
        iat: now.timestamp(),
        token_type: "access".to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| format!("生成Access Token失败: {}", e))
}

/// 生成 Refresh Token
///
/// # Arguments
/// * `user_id` - 用户ID
/// * `username` - 用户名
/// * `role` - 用户角色
/// * `is_verified` - 是否实名认证
/// * `secret` - JWT密钥
///
/// # Returns
/// * `Ok(String)` - JWT Token
/// * `Err(String)` - 错误信息
pub fn generate_refresh_token(
    user_id: Uuid,
    username: String,
    role: UserRole,
    is_verified: bool,
    secret: &str,
) -> Result<String, String> {
    let now = Utc::now();
    let exp = now + Duration::days(REFRESH_TOKEN_EXPIRE_DAYS);

    let claims = Claims {
        sub: user_id.to_string(),
        username,
        role: role.to_string(),
        is_verified,
        exp: exp.timestamp(),
        iat: now.timestamp(),
        token_type: "refresh".to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| format!("生成Refresh Token失败: {}", e))
}

/// 验证 Token
///
/// # Arguments
/// * `token` - JWT Token
/// * `secret` - JWT密钥
/// * `token_type` - 期望的token类型 ("access" 或 "refresh")
///
/// # Returns
/// * `Ok(Claims)` - Token Claims
/// * `Err(String)` - 错误信息
pub fn verify_token(token: &str, secret: &str, token_type: Option<&str>) -> Result<Claims, String> {
    let mut validation = Validation::default();
    validation.validate_exp = true;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|e| format!("Token验证失败: {}", e))?;

    // 如果指定了token类型，验证类型是否匹配
    if let Some(expected_type) = token_type {
        if token_data.claims.token_type != expected_type {
            return Err(format!(
                "Token类型不匹配: 期望 {}, 实际 {}",
                expected_type, token_data.claims.token_type
            ));
        }
    }

    Ok(token_data.claims)
}

/// 从 Claims 中提取当前用户信息
///
/// # Arguments
/// * `claims` - JWT Claims
///
/// # Returns
/// * `Ok(CurrentUser)` - 当前用户信息
/// * `Err(String)` - 错误信息
pub fn extract_current_user(claims: Claims) -> Result<CurrentUser, String> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| "无效的用户ID")?;

    let role = match claims.role.as_str() {
        "admin" => UserRole::Admin,
        "verified" => UserRole::Verified,
        "user" => UserRole::User,
        _ => UserRole::Guest,
    };

    Ok(CurrentUser {
        id: user_id,
        username: claims.username,
        role,
        is_verified: claims.is_verified,
    })
}

/// 获取 Access Token 有效期（秒）（预留接口）
#[allow(dead_code)]
pub fn get_access_token_expire_seconds() -> i64 {
    ACCESS_TOKEN_EXPIRE_MINUTES * 60
}

/// 获取 Refresh Token 有效期（秒）（预留接口）
#[allow(dead_code)]
pub fn get_refresh_token_expire_seconds() -> i64 {
    REFRESH_TOKEN_EXPIRE_DAYS * 24 * 60 * 60
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &str = "test_secret_key_for_jwt_testing";

    #[test]
    fn test_generate_and_verify_access_token() {
        let user_id = Uuid::new_v4();
        let username = "testuser".to_string();
        let role = UserRole::User;

        let token =
            generate_access_token(user_id, username.clone(), role.clone(), false, TEST_SECRET)
                .expect("生成Token失败");

        // 验证Token
        let claims = verify_token(&token, TEST_SECRET, Some("access")).expect("验证Token失败");

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.username, username);
        assert_eq!(claims.role, "user");
        assert_eq!(claims.is_verified, false);
        assert_eq!(claims.token_type, "access");
    }

    #[test]
    fn test_generate_and_verify_refresh_token() {
        let user_id = Uuid::new_v4();
        let username = "testuser".to_string();
        let role = UserRole::User;

        let token =
            generate_refresh_token(user_id, username.clone(), role.clone(), false, TEST_SECRET)
                .expect("生成Token失败");

        // 验证Token
        let claims = verify_token(&token, TEST_SECRET, Some("refresh")).expect("验证Token失败");

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.username, username);
        assert_eq!(claims.token_type, "refresh");
    }

    #[test]
    fn test_token_type_mismatch() {
        let user_id = Uuid::new_v4();
        let username = "testuser".to_string();
        let role = UserRole::User;

        // 生成Access Token
        let access_token =
            generate_access_token(user_id, username, role, false, TEST_SECRET).expect("生成Token失败");

        // 尝试用refresh类型验证access token
        let result = verify_token(&access_token, TEST_SECRET, Some("refresh"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Token类型不匹配"));
    }

    #[test]
    fn test_invalid_token() {
        let result = verify_token("invalid.token.here", TEST_SECRET, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_current_user() {
        let user_id = Uuid::new_v4();
        let claims = Claims {
            sub: user_id.to_string(),
            username: "testuser".to_string(),
            role: "admin".to_string(),
            is_verified: true,
            exp: Utc::now().timestamp() + 3600,
            iat: Utc::now().timestamp(),
            token_type: "access".to_string(),
        };

        let current_user = extract_current_user(claims).expect("提取用户信息失败");

        assert_eq!(current_user.id, user_id);
        assert_eq!(current_user.username, "testuser");
        assert_eq!(current_user.role, UserRole::Admin);
        assert_eq!(current_user.is_verified, true);
    }
}
