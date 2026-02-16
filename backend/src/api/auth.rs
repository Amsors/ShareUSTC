use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use actix_web::cookie::{Cookie, SameSite, time::Duration as CookieDuration};
use crate::db::AppState;
use crate::models::{LoginRequest, RegisterRequest};
use crate::services::{AuthService, AuditLogService, AuthError};
use crate::utils::{bad_request, unauthorized, conflict, internal_error};

/// Cookie 名称常量
const ACCESS_TOKEN_COOKIE: &str = "access_token";
const REFRESH_TOKEN_COOKIE: &str = "refresh_token";

/// 构建 HttpOnly Cookie
fn build_auth_cookie<'a>(name: &'a str, value: &'a str, max_age_days: i64, secure: bool) -> Cookie<'a> {
    Cookie::build(name, value)
        .http_only(true)
        .secure(secure)  // 从配置读取，生产环境设为 true (HTTPS)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(CookieDuration::days(max_age_days))
        .finish()
}

/// 清除认证 Cookie
fn clear_auth_cookie<'a>(name: &'a str, secure: bool) -> Cookie<'a> {
    Cookie::build(name, "")
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(CookieDuration::seconds(0))
        .finish()
}

/// 注册
#[post("/auth/register")]
pub async fn register(
    state: web::Data<AppState>,
    req: web::Json<RegisterRequest>,
    http_req: HttpRequest,
) -> impl Responder {
    let username = req.username.clone();
    log::info!("[Auth] 用户注册请求 | username={}", username);

    match AuthService::register(&state.pool, &state.jwt_secret, req.into_inner()).await {
        Ok(response) => {
            log::info!("[Auth] 用户注册成功 | user_id={}, username={}", response.user.id, response.user.username);

            // 获取 IP 地址
            let ip_address = http_req
                .peer_addr()
                .map(|addr| addr.ip().to_string());

            // 记录审计日志
            let _ = AuditLogService::log_register(
                &state.pool,
                response.user.id,
                &response.user.username,
                ip_address.as_deref(),
            ).await;

            // 设置 HttpOnly Cookies
            let access_cookie = build_auth_cookie(
                ACCESS_TOKEN_COOKIE,
                &response.tokens.access_token,
                1, // 1天
                state.cookie_secure,
            );
            let refresh_cookie = build_auth_cookie(
                REFRESH_TOKEN_COOKIE,
                &response.tokens.refresh_token,
                7, // 7天
                state.cookie_secure,
            );

            // 返回用户信息（不包含token），直接返回用户对象（符合API规范）
            HttpResponse::Created()
                .cookie(access_cookie)
                .cookie(refresh_cookie)
                .json(response.user)
        }
        Err(e) => {
            log::warn!("[Auth] 用户注册失败 | username={}, error={}", username, e);
            match e {
                AuthError::UserExists(msg) => conflict(&msg),
                AuthError::ValidationError(msg) => bad_request(&msg),
                _ => internal_error("注册失败"),
            }
        }
    }
}

/// 登录
#[post("/auth/login")]
pub async fn login(
    state: web::Data<AppState>,
    req: web::Json<LoginRequest>,
    http_req: HttpRequest,
) -> impl Responder {
    let username = req.username.clone();
    log::info!("[Auth] 用户登录请求 | username={}", username);

    match AuthService::login(&state.pool, &state.jwt_secret, req.into_inner()).await {
        Ok(response) => {
            log::info!("[Auth] 用户登录成功 | user_id={}, username={}", response.user.id, response.user.username);

            // 获取 IP 地址
            let ip_address = http_req
                .peer_addr()
                .map(|addr| addr.ip().to_string());

            // 记录审计日志
            let _ = AuditLogService::log_login(
                &state.pool,
                response.user.id,
                &response.user.username,
                ip_address.as_deref(),
            ).await;

            // 设置 HttpOnly Cookies
            let access_cookie = build_auth_cookie(
                ACCESS_TOKEN_COOKIE,
                &response.tokens.access_token,
                1, // 1天
                state.cookie_secure,
            );
            let refresh_cookie = build_auth_cookie(
                REFRESH_TOKEN_COOKIE,
                &response.tokens.refresh_token,
                7, // 7天
                state.cookie_secure,
            );

            // 返回用户信息（不包含token），直接返回用户对象（符合API规范）
            HttpResponse::Ok()
                .cookie(access_cookie)
                .cookie(refresh_cookie)
                .json(response.user)
        }
        Err(e) => {
            log::warn!("[Auth] 用户登录失败 | username={}, error={}", username, e);
            match e {
                AuthError::InvalidCredentials(msg) => unauthorized(&msg),
                AuthError::ValidationError(msg) => bad_request(&msg),
                _ => internal_error("登录失败"),
            }
        }
    }
}

/// 刷新 Token
#[post("/auth/refresh")]
pub async fn refresh(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> impl Responder {
    log::info!("[Auth] Token刷新请求");

    // 从 Cookie 中获取 refresh token
    let refresh_token = req.cookie(REFRESH_TOKEN_COOKIE)
        .map(|c| c.value().to_string());

    if refresh_token.is_none() {
        log::warn!("[Auth] Token刷新失败 | 缺少refresh_token cookie");
        return unauthorized("缺少认证信息");
    }

    let refresh_token = refresh_token.unwrap();

    match AuthService::refresh_token(&state.pool, &state.jwt_secret, refresh_token).await {
        Ok(tokens) => {
            log::info!("[Auth] Token刷新成功");

            // 设置新的 HttpOnly Cookies
            let access_cookie = build_auth_cookie(
                ACCESS_TOKEN_COOKIE,
                &tokens.access_token,
                1, // 1天
                state.cookie_secure,
            );
            let refresh_cookie = build_auth_cookie(
                REFRESH_TOKEN_COOKIE,
                &tokens.refresh_token,
                7, // 7天
                state.cookie_secure,
            );

            HttpResponse::Ok()
                .cookie(access_cookie)
                .cookie(refresh_cookie)
                .json(serde_json::json!({
                    "message": "Token刷新成功"
                }))
        }
        Err(e) => {
            log::warn!("[Auth] Token刷新失败 | error={}", e);
            match e {
                AuthError::TokenInvalid(msg) => unauthorized(&msg),
                _ => internal_error("刷新失败"),
            }
        }
    }
}

/// 登出
#[post("/auth/logout")]
pub async fn logout(
    state: web::Data<AppState>,
) -> impl Responder {
    log::info!("[Auth] 用户登出");

    // 清除 Cookies
    let access_cookie = clear_auth_cookie(ACCESS_TOKEN_COOKIE, state.cookie_secure);
    let refresh_cookie = clear_auth_cookie(REFRESH_TOKEN_COOKIE, state.cookie_secure);

    HttpResponse::Ok()
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .json(serde_json::json!({
            "message": "登出成功"
        }))
}

/// 配置认证路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(register)
        .service(login)
        .service(refresh)
        .service(logout);
}
