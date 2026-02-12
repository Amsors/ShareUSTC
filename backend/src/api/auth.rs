use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use crate::db::AppState;
use crate::models::{LoginRequest, RefreshTokenRequest, RegisterRequest};
use crate::services::{AuthService, AuditLogService};

/// 注册
#[post("/auth/register")]
pub async fn register(
    state: web::Data<AppState>,
    req: web::Json<RegisterRequest>,
    http_req: HttpRequest,
) -> impl Responder {
    log::debug!("收到注册请求: username={}", req.username);

    match AuthService::register(&state.pool, &state.jwt_secret, req.into_inner()).await {
        Ok(response) => {
            log::info!("用户注册成功: {}", response.user.username);

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

            HttpResponse::Ok().json(serde_json::json!({
                "code": 200,
                "message": "注册成功",
                "data": response
            }))
        }
        Err(e) => {
            log::warn!("注册失败: {}", e);
            let (code, message) = match e {
                crate::services::AuthError::UserExists(_) => (409, e.to_string()),
                crate::services::AuthError::ValidationError(_) => (400, e.to_string()),
                _ => (500, "注册失败".to_string()),
            };
            HttpResponse::Ok().json(serde_json::json!({
                "code": code,
                "message": message,
                "data": null
            }))
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
    log::debug!("收到登录请求: username={}", req.username);

    match AuthService::login(&state.pool, &state.jwt_secret, req.into_inner()).await {
        Ok(response) => {
            log::info!("用户登录成功: {}", response.user.username);

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

            HttpResponse::Ok().json(serde_json::json!({
                "code": 200,
                "message": "登录成功",
                "data": response
            }))
        }
        Err(e) => {
            log::warn!("登录失败: {}", e);
            let (code, message) = match e {
                crate::services::AuthError::InvalidCredentials(_) => (401, e.to_string()),
                crate::services::AuthError::ValidationError(_) => (400, e.to_string()),
                _ => (500, "登录失败".to_string()),
            };
            HttpResponse::Ok().json(serde_json::json!({
                "code": code,
                "message": message,
                "data": null
            }))
        }
    }
}

/// 刷新 Token
#[post("/auth/refresh")]
pub async fn refresh(
    state: web::Data<AppState>,
    req: web::Json<RefreshTokenRequest>,
) -> impl Responder {
    log::debug!("收到刷新Token请求");

    match AuthService::refresh_token(&state.pool, &state.jwt_secret, req.into_inner()).await {
        Ok(tokens) => {
            log::info!("Token刷新成功");
            HttpResponse::Ok().json(serde_json::json!({
                "code": 200,
                "message": "刷新成功",
                "data": tokens
            }))
        }
        Err(e) => {
            log::warn!("Token刷新失败: {}", e);
            let (code, message) = match e {
                crate::services::AuthError::TokenInvalid(_) => (401, e.to_string()),
                _ => (500, "刷新失败".to_string()),
            };
            HttpResponse::Ok().json(serde_json::json!({
                "code": code,
                "message": message,
                "data": null
            }))
        }
    }
}

/// 登出（此处仅记录，实际Token失效需要在前端处理或使用黑名单）
#[post("/auth/logout")]
pub async fn logout() -> impl Responder {
    log::info!("用户登出");
    HttpResponse::Ok().json(serde_json::json!({
        "code": 200,
        "message": "登出成功",
        "data": null
    }))
}

/// 配置认证路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(register)
        .service(login)
        .service(refresh)
        .service(logout);
}
