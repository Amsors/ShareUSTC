use actix_web::{get, put, web, HttpRequest, HttpResponse, Responder};
use uuid::Uuid;

use crate::db::AppState;
use crate::models::CurrentUser;
use crate::services::{AdminService, AuditLogService, UpdateUserStatusRequest};
use crate::utils::bad_request;

use super::{check_admin, utils::handle_admin_error};

/// 获取用户列表
#[get("/admin/users")]
async fn get_user_list(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let user = current_user.into_inner();
    log::info!("[Admin] 获取用户列表 | admin_id={}", user.id);

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    let page = query
        .get("page")
        .and_then(|p| p.parse::<i32>().ok())
        .unwrap_or(1);
    let per_page = query
        .get("perPage")
        .and_then(|p| p.parse::<i32>().ok())
        .unwrap_or(20);

    match AdminService::get_user_list(&data.pool, page, per_page).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => handle_admin_error(e),
    }
}

/// 更新用户状态（禁用/启用）
#[put("/admin/users/{user_id}/status")]
async fn update_user_status(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    req: web::Json<UpdateUserStatusRequest>,
    http_req: HttpRequest,
) -> impl Responder {
    let user = current_user.into_inner();

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    let user_id = path.into_inner();
    log::info!(
        "[Admin] 更新用户状态 | admin_id={}, target_user_id={}, is_active={}",
        user.id,
        user_id,
        req.is_active
    );

    // 禁止禁用自己
    if user_id == user.id {
        log::warn!("[Admin] 管理员尝试禁用自己 | admin_id={}", user.id);
        return bad_request("不能禁用自己的账号");
    }

    match AdminService::update_user_status(&data.pool, user_id, req.is_active).await {
        Ok(_) => {
            log::info!(
                "[Admin] 用户状态更新成功 | admin_id={}, target_user_id={}",
                user.id,
                user_id
            );

            // 记录审计日志
            let ip_address = http_req.peer_addr().map(|addr| addr.ip().to_string());
            if let Err(e) = AuditLogService::log_update_user_status(
                &data.pool,
                user.id,
                user_id,
                req.is_active,
                ip_address.as_deref(),
            )
            .await
            {
                log::warn!(
                    "[Audit] 记录更新用户状态日志失败 | admin_id={}, target_user_id={}, error={}",
                    user.id,
                    user_id,
                    e
                );
            }

            HttpResponse::Ok().json(serde_json::json!({
                "message": "用户状态已更新"
            }))
        }
        Err(e) => handle_admin_error(e),
    }
}

/// 获取用户实名信息
#[get("/admin/users/{user_id}/real-info")]
async fn get_user_real_info(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let user = current_user.into_inner();

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    let user_id = path.into_inner();
    log::info!(
        "[Admin] 管理员获取用户实名信息 | admin_id={}, target_user_id={}",
        user.id,
        user_id
    );

    match AdminService::get_user_real_info(&data.pool, user_id).await {
        Ok(real_info) => {
            log::info!(
                "[Admin] 获取用户实名信息成功 | admin_id={}, target_user_id={}",
                user.id,
                user_id
            );
            HttpResponse::Ok().json(real_info)
        }
        Err(e) => {
            log::warn!(
                "[Admin] 获取用户实名信息失败 | admin_id={}, target_user_id={}, error={}",
                user.id,
                user_id,
                e
            );
            handle_admin_error(e)
        }
    }
}

/// 配置路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_user_list)
        .service(update_user_status)
        .service(get_user_real_info);
}
