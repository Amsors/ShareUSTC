use actix_web::{get, put, post, web, HttpResponse, Responder};
use crate::db::AppState;
use crate::models::{CurrentUser, UpdateProfileRequest, VerificationRequest, AuthResponse, TokenResponse, UserRole, UserHomepageQuery};
use crate::services::{UserError, UserService};
use crate::utils::{generate_access_token, generate_refresh_token, bad_request, forbidden, not_found, internal_error};
use uuid::Uuid;

/// 获取当前用户信息
#[get("/users/me")]
pub async fn get_current_user(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
) -> impl Responder {
    log::debug!("[User] 获取当前用户信息 | user_id={}", user.id);

    match UserService::get_current_user(&state.pool, user.id).await {
        Ok(user_info) => {
            log::info!("[User] 获取当前用户信息成功 | user_id={}, username={}", user.id, user_info.username);
            HttpResponse::Ok().json(user_info)
        }
        Err(e) => {
            log::warn!("[User] 获取当前用户信息失败 | user_id={}, error={}", user.id, e);
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
) -> impl Responder {
    // 检查是否为实名用户或管理员
    let is_verified = user.role == crate::models::UserRole::Verified
        || user.role == crate::models::UserRole::Admin;

    // 未实名用户尝试修改个人简介时，返回错误
    if !is_verified && req.bio.is_some() {
        return forbidden("实名认证后才可修改个人简介");
    }

    log::info!("[User] 更新用户资料 | user_id={}", user.id);

    match UserService::update_profile(&state.pool, user.id, req.into_inner(), is_verified).await {
        Ok(user_info) => {
            log::info!("[User] 用户资料更新成功 | user_id={}", user.id);
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
                    log::error!("[Auth] 生成访问令牌失败 | user_id={}, error={}", user_info.id, e);
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
                    log::error!("[Auth] 生成刷新令牌失败 | user_id={}, error={}", user_info.id, e);
                    return internal_error("认证成功但生成令牌失败，请重新登录");
                }
            };

            let auth_response = AuthResponse {
                user: user_info,
                tokens: TokenResponse {
                    access_token,
                    refresh_token,
                    token_type: "Bearer".to_string(),
                    expires_in: 15 * 60, // 15分钟
                },
            };

            HttpResponse::Ok().json(auth_response)
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
pub async fn get_user_profile(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> impl Responder {
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

/// 配置用户路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_current_user)
        .service(update_profile)
        .service(verify_user)
        .service(get_user_homepage)  // 必须在 get_user_profile 之前注册
        .service(get_user_profile);
}
