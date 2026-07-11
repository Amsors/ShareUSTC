// 认证 Cookie 工具：Cookie 名称常量与构建/清除逻辑的单一定义
// 供 api/auth.rs、api/user.rs、middleware/auth.rs 共用，避免重复实现。

use actix_web::cookie::{time::Duration as CookieDuration, Cookie, SameSite};

/// 访问令牌 Cookie 名称
pub const ACCESS_TOKEN_COOKIE: &str = "access_token";
/// 刷新令牌 Cookie 名称
pub const REFRESH_TOKEN_COOKIE: &str = "refresh_token";

/// 构建 HttpOnly 认证 Cookie
pub fn build_auth_cookie<'a>(
    name: &'a str,
    value: &'a str,
    max_age_days: i64,
    secure: bool,
) -> Cookie<'a> {
    Cookie::build(name, value)
        .http_only(true)
        .secure(secure) // 从配置读取，生产环境设为 true (HTTPS)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(CookieDuration::days(max_age_days))
        .finish()
}

/// 构建用于清除认证 Cookie 的空值 Cookie（max-age=0）
pub fn clear_auth_cookie<'a>(name: &'a str, secure: bool) -> Cookie<'a> {
    Cookie::build(name, "")
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(CookieDuration::seconds(0))
        .finish()
}
