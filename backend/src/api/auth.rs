use crate::db::AppState;
use crate::models::{LoginRequest, RegisterRequest};
use crate::services::{AuditLogService, AuthError, AuthService};
use crate::utils::{
    build_auth_cookie, clear_auth_cookie, unauthorized, ACCESS_TOKEN_COOKIE, REFRESH_TOKEN_COOKIE,
};
use actix_web::{post, web, HttpRequest, HttpResponse, Responder};

/// 注册
#[post("/auth/register")]
pub async fn register(
    state: web::Data<AppState>,
    req: web::Json<RegisterRequest>,
    http_req: HttpRequest,
) -> Result<HttpResponse, AuthError> {
    let username = req.username.clone();
    log::info!("[Auth] 用户注册请求 | username={}", username);

    let response = AuthService::register(
        &state.pool,
        &state.jwt_secret,
        req.into_inner(),
        state.require_email_on_register,
        &state.config.admin_usernames,
    )
    .await?;

    log::info!(
        "[Auth] 用户注册成功 | user_id={}, username={}",
        response.user.id,
        response.user.username
    );

    // 获取 IP 地址
    let ip_address = http_req.peer_addr().map(|addr| addr.ip().to_string());

    // 记录审计日志
    let _ = AuditLogService::log_register(
        &state.pool,
        response.user.id,
        &response.user.username,
        ip_address.as_deref(),
    )
    .await;

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
    Ok(HttpResponse::Created()
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .json(response.user))
}

/// 登录
#[post("/auth/login")]
pub async fn login(
    state: web::Data<AppState>,
    req: web::Json<LoginRequest>,
    http_req: HttpRequest,
) -> Result<HttpResponse, AuthError> {
    let username = req.username.clone();
    log::info!("[Auth] 用户登录请求 | username={}", username);

    // 登录失败经 AuthError 的 ResponseError 生成响应：凭证错误 401 + InvalidCredentials，
    // 校验错误 400，其余 500（见 api_design.md 2.1）
    let response = AuthService::login(&state.pool, &state.jwt_secret, req.into_inner()).await?;

    log::info!(
        "[Auth] 用户登录成功 | user_id={}, username={}",
        response.user.id,
        response.user.username
    );

    // 获取 IP 地址
    let ip_address = http_req.peer_addr().map(|addr| addr.ip().to_string());

    // 记录审计日志
    let _ = AuditLogService::log_login(
        &state.pool,
        response.user.id,
        &response.user.username,
        ip_address.as_deref(),
    )
    .await;

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
    Ok(HttpResponse::Ok()
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .json(response.user))
}

/// 刷新 Token
#[post("/auth/refresh")]
pub async fn refresh(
    state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<HttpResponse, AuthError> {
    log::info!("[Auth] Token刷新请求");

    // 从 Cookie 中获取 refresh token；缺失时直接返回 401，避免 unwrap 造成 panic
    let Some(refresh_token) = req
        .cookie(REFRESH_TOKEN_COOKIE)
        .map(|c| c.value().to_string())
    else {
        log::warn!("[Auth] Token刷新失败 | 缺少refresh_token cookie");
        return Ok(unauthorized("缺少认证信息"));
    };

    let tokens = AuthService::refresh_token(&state.pool, &state.jwt_secret, refresh_token).await?;
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

    Ok(HttpResponse::Ok()
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .json(serde_json::json!({
            "message": "Token刷新成功"
        })))
}

/// 登出
#[post("/auth/logout")]
pub async fn logout(state: web::Data<AppState>) -> impl Responder {
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
