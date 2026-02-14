use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use crate::db::AppState;
use crate::models::{LoginRequest, RefreshTokenRequest, RegisterRequest};
use crate::services::{AuthService, AuditLogService, AuthError};
use crate::utils::{bad_request, unauthorized, conflict, internal_error, created};

/// 注册
#[post("/auth/register")]
pub async fn register(
    state: web::Data<AppState>,
    req: web::Json<RegisterRequest>,
    http_req: HttpRequest,
) -> impl Responder {
    let username = req.username.clone();
    log::info!("[Auth] 用户注册 | username={}", username);

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

            created(response)
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
    log::info!("[Auth] 用户登录 | username={}", username);

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

            HttpResponse::Ok().json(response)
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
    req: web::Json<RefreshTokenRequest>,
) -> impl Responder {
    log::info!("[Auth] Token刷新请求");

    match AuthService::refresh_token(&state.pool, &state.jwt_secret, req.into_inner()).await {
        Ok(tokens) => {
            log::info!("[Auth] Token刷新成功");
            HttpResponse::Ok().json(tokens)
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

/// 登出（此处仅记录，实际Token失效需要在前端处理或使用黑名单）
#[post("/auth/logout")]
pub async fn logout() -> impl Responder {
    log::info!("[Auth] 用户登出");
    HttpResponse::Ok().json(serde_json::json!({
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
