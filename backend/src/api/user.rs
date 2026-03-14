use crate::db::AppState;
use crate::models::{
    ChangePasswordRequest, CurrentUser, LeaderboardQuery, UpdateProfileRequest, UserHomepageQuery,
    UserRole, VerificationRequest,
};
use crate::services::{AuditLogService, UserError, UserService};
use crate::utils::{
    bad_request, forbidden, generate_access_token, generate_refresh_token, internal_error,
    not_found, unauthorized,
};
use actix_web::cookie::{time::Duration as CookieDuration, Cookie, SameSite};
use actix_web::{get, post, put, web, HttpRequest, HttpResponse, Responder};
use serde::Serialize;
use uuid::Uuid;

/// 站点公开配置响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteConfigResponse {
    /// 注册时是否强制要求邮箱
    pub require_email_on_register: bool,
    /// 是否允许用户修改用户名
    pub allow_username_change: bool,
    /// 是否允许用户修改邮箱
    pub allow_email_change: bool,
}

/// Cookie 名称常量
const ACCESS_TOKEN_COOKIE: &str = "access_token";
const REFRESH_TOKEN_COOKIE: &str = "refresh_token";

/// 构建 HttpOnly Cookie
fn build_auth_cookie<'a>(
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

/// 获取当前用户信息
#[get("/users/me")]
pub async fn get_current_user(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
) -> impl Responder {
    log::debug!("[User] 获取当前用户信息 | user_id={}", user.id);

    match UserService::get_current_user(&state.pool, user.id).await {
        Ok(user_info) => {
            log::info!(
                "[User] 获取当前用户信息成功 | user_id={}, username={}",
                user.id,
                user_info.username
            );
            HttpResponse::Ok().json(user_info)
        }
        Err(e) => {
            log::warn!(
                "[User] 获取当前用户信息失败 | user_id={}, error={}",
                user.id,
                e
            );
            match e {
                UserError::UserNotFound(msg) => not_found(&msg),
                _ => internal_error("获取用户信息失败"),
            }
        }
    }
}

/// 更新当前用户资料
#[put("/users/me")]
pub async fn update_profile(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    req: web::Json<UpdateProfileRequest>,
    http_req: HttpRequest,
) -> impl Responder {
    // 检查是否为实名用户或管理员
    let is_verified = user.role == crate::models::UserRole::Verified
        || user.role == crate::models::UserRole::Admin;

    // 未实名用户尝试修改个人简介时，返回错误
    // 检查 bio 是否为有效值（非空字符串且非空白）
    let bio_has_value = req.bio.as_ref().map_or(false, |b| !b.trim().is_empty());
    if !is_verified && bio_has_value {
        return forbidden("实名认证后才可修改个人简介");
    }

    log::info!("[User] 更新用户资料 | user_id={}", user.id);

    match UserService::update_profile(
        &state.pool,
        user.id,
        req.into_inner(),
        is_verified,
        state.allow_username_change,
        state.allow_email_change,
    )
    .await
    {
        Ok(user_info) => {
            log::info!("[User] 用户资料更新成功 | user_id={}", user.id);

            // 记录审计日志
            let ip_address = http_req.peer_addr().map(|addr| addr.ip().to_string());
            if let Err(e) = AuditLogService::log_update_profile(
                &state.pool,
                user.id,
                &user_info.username,
                ip_address.as_deref(),
            )
            .await
            {
                log::warn!(
                    "[Audit] 记录更新个人主页日志失败 | user_id={}, error={}",
                    user.id,
                    e
                );
            }

            HttpResponse::Ok().json(user_info)
        }
        Err(e) => {
            log::warn!("[User] 更新用户资料失败 | user_id={}, error={}", user.id, e);
            match e {
                UserError::UserNotFound(msg) => not_found(&msg),
                UserError::UserExists(msg) => HttpResponse::Conflict().json(serde_json::json!({
                    "error": "Conflict",
                    "message": msg
                })),
                UserError::ValidationError(msg) => bad_request(&msg),
                _ => internal_error("更新失败"),
            }
        }
    }
}

/// 实名认证
#[post("/users/verify")]
pub async fn verify_user(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    req: web::Json<VerificationRequest>,
) -> impl Responder {
    // 检查是否已经完成实名认证（通过 is_verified 字段判断）
    if user.is_verified {
        return bad_request("用户已完成实名认证");
    }

    match UserService::verify_user(&state.pool, user.id, req.into_inner()).await {
        Ok(user_info) => {
            // 实名认证成功，生成新的 Token（保持原有角色）
            let user_role = match user_info.role.as_str() {
                "admin" => UserRole::Admin,
                "verified" => UserRole::Verified,
                "user" => UserRole::User,
                _ => UserRole::Guest,
            };
            let access_token = match generate_access_token(
                user_info.id,
                user_info.username.clone(),
                user_role.clone(),
                user_info.is_verified,
                &state.jwt_secret,
            ) {
                Ok(token) => token,
                Err(e) => {
                    log::error!(
                        "[Auth] 生成访问令牌失败 | user_id={}, error={}",
                        user_info.id,
                        e
                    );
                    return internal_error("认证成功但生成令牌失败，请重新登录");
                }
            };

            let refresh_token = match generate_refresh_token(
                user_info.id,
                user_info.username.clone(),
                user_role,
                user_info.is_verified,
                &state.jwt_secret,
            ) {
                Ok(token) => token,
                Err(e) => {
                    log::error!(
                        "[Auth] 生成刷新令牌失败 | user_id={}, error={}",
                        user_info.id,
                        e
                    );
                    return internal_error("认证成功但生成令牌失败，请重新登录");
                }
            };

            // 设置 HttpOnly Cookies
            let access_cookie = build_auth_cookie(
                ACCESS_TOKEN_COOKIE,
                &access_token,
                1, // 1天
                state.cookie_secure,
            );
            let refresh_cookie = build_auth_cookie(
                REFRESH_TOKEN_COOKIE,
                &refresh_token,
                7, // 7天
                state.cookie_secure,
            );

            // 返回用户信息（不包含token），直接返回用户对象（符合API规范）
            HttpResponse::Ok()
                .cookie(access_cookie)
                .cookie(refresh_cookie)
                .json(user_info)
        }
        Err(e) => {
            log::warn!("[User] 实名认证失败 | user_id={}, error={}", user.id, e);
            match e {
                UserError::UserNotFound(msg) => not_found(&msg),
                UserError::ValidationError(msg) => bad_request(&msg),
                _ => internal_error("认证失败"),
            }
        }
    }
}

/// 获取用户公开资料（公开接口，任何人都可以访问）
#[get("/users/{user_id}")]
pub async fn get_user_profile(state: web::Data<AppState>, path: web::Path<Uuid>) -> impl Responder {
    let user_id = path.into_inner();

    match UserService::get_user_profile(&state.pool, user_id).await {
        Ok(profile) => HttpResponse::Ok().json(profile),
        Err(e) => {
            log::warn!("[User] 获取用户资料失败 | user_id={}, error={}", user_id, e);
            match e {
                UserError::UserNotFound(msg) => not_found(&msg),
                _ => internal_error("获取用户资料失败"),
            }
        }
    }
}

/// 获取用户主页数据（公开接口，任何人都可以访问）
/// 包含用户基本信息、统计数据和已通过审核的资源列表
#[get("/users/{user_id}/homepage")]
pub async fn get_user_homepage(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    query: web::Query<UserHomepageQuery>,
) -> impl Responder {
    let user_id = path.into_inner();

    match UserService::get_user_homepage(&state.pool, user_id, &query.into_inner()).await {
        Ok(homepage) => HttpResponse::Ok().json(homepage),
        Err(e) => {
            log::warn!("[User] 获取用户主页失败 | user_id={}, error={}", user_id, e);
            match e {
                UserError::UserNotFound(msg) => not_found(&msg),
                _ => internal_error("获取用户主页失败"),
            }
        }
    }
}

/// 获取站点公开配置（公开接口，无需认证）
#[get("/config")]
pub async fn get_site_config(state: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().json(SiteConfigResponse {
        require_email_on_register: state.require_email_on_register,
        allow_username_change: state.allow_username_change,
        allow_email_change: state.allow_email_change,
    })
}

/// 修改用户密码
#[put("/users/me/password")]
pub async fn change_password(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    req: web::Json<ChangePasswordRequest>,
    http_req: HttpRequest,
) -> impl Responder {
    // 验证请求
    if let Err(msg) = req.validate() {
        return bad_request(&msg);
    }

    log::info!("[User] 用户请求修改密码 | user_id={}", user.id);

    match UserService::change_password(&state.pool, user.id, &req.old_password, &req.new_password)
        .await
    {
        Ok(()) => {
            log::info!("[User] 用户密码修改成功 | user_id={}", user.id);

            // 记录审计日志
            let ip_address = http_req.peer_addr().map(|addr| addr.ip().to_string());
            if let Err(e) = AuditLogService::log_action(
                &state.pool,
                user.id,
                "change_password",
                Some("user"),
                Some(user.id),
                None,
                ip_address.as_deref(),
            )
            .await
            {
                log::warn!(
                    "[Audit] 记录修改密码日志失败 | user_id={}, error={}",
                    user.id,
                    e
                );
            }

            HttpResponse::Ok().json(serde_json::json!({
                "message": "密码修改成功"
            }))
        }
        Err(e) => {
            log::warn!("[User] 用户密码修改失败 | user_id={}, error={}", user.id, e);
            match e {
                UserError::UserNotFound(msg) => not_found(&msg),
                UserError::InvalidCredentials(msg) => unauthorized(&msg),
                UserError::ValidationError(msg) => bad_request(&msg),
                _ => internal_error("修改密码失败"),
            }
        }
    }
}

/// 获取贡献榜单（公开接口，无需认证）
#[get("/users/leaderboard")]
pub async fn get_leaderboard(
    state: web::Data<AppState>,
    query: web::Query<LeaderboardQuery>,
) -> impl Responder {
    match UserService::get_leaderboard(&state.pool, &query.into_inner()).await {
        Ok(leaderboard) => HttpResponse::Ok().json(leaderboard),
        Err(e) => {
            log::warn!("[User] 获取贡献榜单失败 | error={}", e);
            internal_error("获取贡献榜单失败")
        }
    }
}

/// 配置用户路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_current_user)
        .service(update_profile)
        .service(verify_user)
        .service(get_leaderboard) // 必须在 get_user_profile 之前注册，避免被解析为 user_id
        .service(get_user_homepage) // 必须在 get_user_profile 之前注册
        .service(get_user_profile)
        .service(get_site_config)
        .service(change_password);
}
