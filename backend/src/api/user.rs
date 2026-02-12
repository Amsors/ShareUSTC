use actix_web::{get, put, post, web, HttpResponse, Responder};
use crate::db::AppState;
use crate::models::{CurrentUser, UpdateProfileRequest, VerificationRequest, AuthResponse, TokenResponse, UserRole, UserHomepageQuery};
use crate::services::{UserError, UserService};
use crate::utils::{generate_access_token, generate_refresh_token};
use uuid::Uuid;

/// 获取当前用户信息
#[get("/users/me")]
pub async fn get_current_user(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
) -> impl Responder {
    log::info!("获取当前用户信息: user_id={}, username={}, role={}", user.id, user.username, user.role.to_string());

    match UserService::get_current_user(&state.pool, user.id).await {
        Ok(user_info) => {
            log::info!("返回用户信息: username={}, role={}", user_info.username, user_info.role);
            HttpResponse::Ok().json(serde_json::json!({
                "code": 200,
                "message": "获取成功",
                "data": user_info
            }))
        }
        Err(e) => {
            log::warn!("获取当前用户失败: {}", e);
            let (code, message) = match e {
                UserError::UserNotFound(_) => (404, e.to_string()),
                _ => (500, "获取用户信息失败".to_string()),
            };
            HttpResponse::Ok().json(serde_json::json!({
                "code": code,
                "message": message,
                "data": null
            }))
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
        return HttpResponse::Ok().json(serde_json::json!({
            "code": 403,
            "message": "实名认证后才可修改个人简介",
            "data": null
        }));
    }

    match UserService::update_profile(&state.pool, user.id, req.into_inner(), is_verified).await {
        Ok(user_info) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "更新成功",
            "data": user_info
        })),
        Err(e) => {
            log::warn!("更新用户资料失败: {}", e);
            let (code, message) = match e {
                UserError::UserNotFound(_) => (404, e.to_string()),
                UserError::UserExists(_) => (409, e.to_string()),
                UserError::ValidationError(_) => (400, e.to_string()),
                _ => (500, "更新失败".to_string()),
            };
            HttpResponse::Ok().json(serde_json::json!({
                "code": code,
                "message": message,
                "data": null
            }))
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
        return HttpResponse::Ok().json(serde_json::json!({
            "code": 400,
            "message": "用户已完成实名认证",
            "data": null
        }));
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
                    log::error!("生成访问令牌失败: {}", e);
                    return HttpResponse::Ok().json(serde_json::json!({
                        "code": 500,
                        "message": "认证成功但生成令牌失败，请重新登录",
                        "data": null
                    }));
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
                    log::error!("生成刷新令牌失败: {}", e);
                    return HttpResponse::Ok().json(serde_json::json!({
                        "code": 500,
                        "message": "认证成功但生成令牌失败，请重新登录",
                        "data": null
                    }));
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

            HttpResponse::Ok().json(serde_json::json!({
                "code": 200,
                "message": "实名认证成功",
                "data": auth_response
            }))
        }
        Err(e) => {
            log::warn!("实名认证失败: {}", e);
            let (code, message) = match e {
                UserError::UserNotFound(_) => (404, e.to_string()),
                UserError::ValidationError(_) => (400, e.to_string()),
                _ => (500, "认证失败".to_string()),
            };
            HttpResponse::Ok().json(serde_json::json!({
                "code": code,
                "message": message,
                "data": null
            }))
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
        Ok(profile) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "获取成功",
            "data": profile
        })),
        Err(e) => {
            log::warn!("获取用户资料失败: {}", e);
            let (code, message) = match e {
                UserError::UserNotFound(_) => (404, e.to_string()),
                _ => (500, "获取用户资料失败".to_string()),
            };
            HttpResponse::Ok().json(serde_json::json!({
                "code": code,
                "message": message,
                "data": null
            }))
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
        Ok(homepage) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "获取成功",
            "data": homepage
        })),
        Err(e) => {
            log::warn!("获取用户主页失败: {}", e);
            let (code, message) = match e {
                UserError::UserNotFound(_) => (404, e.to_string()),
                _ => (500, "获取用户主页失败".to_string()),
            };
            HttpResponse::Ok().json(serde_json::json!({
                "code": code,
                "message": message,
                "data": null
            }))
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
