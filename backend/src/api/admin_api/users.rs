use actix_web::{get, put, web, HttpRequest, HttpResponse};
use uuid::Uuid;

use crate::db::AppState;
use crate::models::CurrentUser;
use crate::services::{
    AdminPaginationQuery, AdminService, AuditLogService, UpdateUserStatusRequest,
};
use crate::utils::bad_request;

use super::check_admin;

/// 获取用户列表
#[get("/admin/users")]
async fn get_user_list(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    query: web::Query<AdminPaginationQuery>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let user = current_user.into_inner();
    log::info!("[Admin] 获取用户列表 | admin_id={}", user.id);

    check_admin(&user)?;

    let response =
        AdminService::get_user_list(&data.pool, query.get_page(), query.get_per_page()).await?;
    Ok(HttpResponse::Ok().json(response))
}

/// 更新用户状态（禁用/启用）
#[put("/admin/users/{user_id}/status")]
async fn update_user_status(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    req: web::Json<UpdateUserStatusRequest>,
    http_req: HttpRequest,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let user = current_user.into_inner();

    check_admin(&user)?;

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
        return Ok(bad_request("不能禁用自己的账号"));
    }

    AdminService::update_user_status(&data.pool, user_id, req.is_active).await?;

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

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "用户状态已更新"
    })))
}

/// 获取用户实名信息
#[get("/admin/users/{user_id}/real-info")]
async fn get_user_real_info(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let user = current_user.into_inner();

    check_admin(&user)?;

    let user_id = path.into_inner();
    log::info!(
        "[Admin] 管理员获取用户实名信息 | admin_id={}, target_user_id={}",
        user.id,
        user_id
    );

    let real_info = AdminService::get_user_real_info(&data.pool, user_id).await?;
    log::info!(
        "[Admin] 获取用户实名信息成功 | admin_id={}, target_user_id={}",
        user.id,
        user_id
    );
    Ok(HttpResponse::Ok().json(real_info))
}

/// 配置路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_user_list)
        .service(update_user_status)
        .service(get_user_real_info);
}
