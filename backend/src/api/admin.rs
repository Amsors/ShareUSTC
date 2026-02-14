use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;

use crate::db::AppState;
use crate::models::CurrentUser;
use crate::services::{
    AdminService, AdminError, AuditResourceRequest, UpdateUserStatusRequest,
    AuditLogQuery,
};
use crate::utils::{bad_request, forbidden, not_found, internal_error, no_content};

/// 检查用户是否是管理员
fn check_admin(current_user: &CurrentUser) -> Result<(), AdminError> {
    if !matches!(current_user.role, crate::models::UserRole::Admin) {
        return Err(AdminError::Forbidden("需要管理员权限".to_string()));
    }
    Ok(())
}

/// 将AdminError转换为HttpResponse
/// 使用正确的 HTTP 状态码
fn handle_admin_error(err: AdminError) -> HttpResponse {
    match err {
        AdminError::NotFound(msg) => not_found(&msg),
        AdminError::ValidationError(msg) => bad_request(&msg),
        AdminError::Forbidden(msg) => forbidden(&msg),
        AdminError::DatabaseError(msg) => {
            log::error!("[Admin] 数据库错误 | error={}", msg);
            internal_error("服务器内部错误")
        }
    }
}

/// 获取仪表盘统计数据
#[get("/admin/dashboard")]
async fn get_dashboard(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
) -> impl Responder {
    let user = current_user.into_inner();
    log::info!("[Admin] 获取仪表盘数据 | admin_id={}", user.id);

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    match AdminService::get_dashboard_stats(&data.pool).await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => handle_admin_error(e),
    }
}

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
) -> impl Responder {
    let user = current_user.into_inner();

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    let user_id = path.into_inner();
    log::info!("[Admin] 更新用户状态 | admin_id={}, target_user_id={}, is_active={}",
        user.id, user_id, req.is_active);

    // 禁止禁用自己
    if user_id == user.id {
        log::warn!("[Admin] 管理员尝试禁用自己 | admin_id={}", user.id);
        return bad_request("不能禁用自己的账号");
    }

    match AdminService::update_user_status(&data.pool, user_id, req.is_active
    ).await {
        Ok(_) => {
            log::info!("[Admin] 用户状态更新成功 | admin_id={}, target_user_id={}", user.id, user_id);
            HttpResponse::Ok().json(serde_json::json!({
                "message": "用户状态已更新"
            }))
        }
        Err(e) => handle_admin_error(e),
    }
}

/// 获取待审核资源列表
#[get("/admin/resources/pending")]
async fn get_pending_resources(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let user = current_user.into_inner();
    log::info!("[Admin] 获取待审核资源列表 | admin_id={}", user.id);

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

    match AdminService::get_pending_resources(&data.pool, page, per_page
    ).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => handle_admin_error(e),
    }
}

/// 审核资源
#[put("/admin/resources/{resource_id}/audit")]
async fn audit_resource(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    req: web::Json<AuditResourceRequest>,
) -> impl Responder {
    let user = current_user.into_inner();

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    let resource_id = path.into_inner();
    log::info!("[Admin] 审核资源 | admin_id={}, resource_id={}, status={}",
        user.id, resource_id, req.status);

    match AdminService::audit_resource(
        &data.pool,
        resource_id,
        req.status.clone(),
        req.reason.clone(),
    ).await {
        Ok(_) => {
            log::info!("[Admin] 资源审核完成 | admin_id={}, resource_id={}", user.id, resource_id);
            HttpResponse::Ok().json(serde_json::json!({
                "message": "资源审核完成"
            }))
        }
        Err(e) => handle_admin_error(e),
    }
}

/// 获取评论列表
#[get("/admin/comments")]
async fn get_comment_list(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let user = current_user.into_inner();
    log::info!("[Admin] 获取评论列表 | admin_id={}", user.id);

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
    let audit_status = query.get("auditStatus").cloned();

    match AdminService::get_comment_list(
        &data.pool, page, per_page, audit_status
    ).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => handle_admin_error(e),
    }
}

/// 删除评论
#[delete("/admin/comments/{comment_id}")]
async fn delete_comment(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let user = current_user.into_inner();

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    let comment_id = path.into_inner();
    log::info!("[Admin] 删除评论 | admin_id={}, comment_id={}", user.id, comment_id);

    match AdminService::delete_comment(&data.pool, comment_id).await {
        Ok(_) => {
            log::info!("[Admin] 评论删除成功 | admin_id={}, comment_id={}", user.id, comment_id);
            no_content()
        }
        Err(e) => handle_admin_error(e),
    }
}

/// 审核评论
#[put("/admin/comments/{comment_id}/audit")]
async fn audit_comment(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    req: web::Json<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let user = current_user.into_inner();

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    let comment_id = path.into_inner();
    let status = req.get("status").cloned().unwrap_or_default();
    log::info!("[Admin] 审核评论 | admin_id={}, comment_id={}, status={}",
        user.id, comment_id, status);

    match AdminService::audit_comment(
        &data.pool, comment_id, status
    ).await {
        Ok(_) => {
            log::info!("[Admin] 评论审核完成 | admin_id={}, comment_id={}", user.id, comment_id);
            HttpResponse::Ok().json(serde_json::json!({
                "message": "评论审核完成"
            }))
        }
        Err(e) => handle_admin_error(e),
    }
}

/// 发送系统通知
#[post("/admin/notifications")]
async fn send_notification(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    req: web::Json<crate::services::SendNotificationRequest>,
) -> impl Responder {
    let user = current_user.into_inner();
    log::info!("[Admin] 发送系统通知 | admin_id={}, title={}", user.id, req.title);

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    match AdminService::send_notification(&data.pool, req.into_inner()).await {
        Ok(_) => {
            log::info!("[Admin] 系统通知发送成功 | admin_id={}", user.id);
            HttpResponse::Created().json(serde_json::json!({
                "message": "通知发送成功"
            }))
        }
        Err(e) => handle_admin_error(e),
    }
}

/// 获取详细统计数据
#[get("/admin/stats/detailed")]
async fn get_detailed_stats(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
) -> impl Responder {
    let user = current_user.into_inner();
    log::info!("[Admin] 获取详细统计数据 | admin_id={}", user.id);

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    match AdminService::get_detailed_stats(&data.pool).await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => handle_admin_error(e),
    }
}

/// 获取操作日志列表
#[get("/admin/audit-logs")]
async fn get_audit_logs(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    query: web::Query<AuditLogQuery>,
) -> impl Responder {
    let user = current_user.into_inner();
    log::info!("[Admin] 获取审计日志 | admin_id={}", user.id);

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    let query_params = AuditLogQuery {
        page: query.page,
        per_page: query.per_page,
        action: query.action.clone(),
        user_id: query.user_id,
        start_date: query.start_date.clone(),
        end_date: query.end_date.clone(),
    };

    match AdminService::get_audit_logs(&data.pool, query_params).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => handle_admin_error(e),
    }
}

/// 配置管理后台路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_dashboard)
        .service(get_user_list)
        .service(update_user_status)
        .service(get_pending_resources)
        .service(audit_resource)
        .service(get_comment_list)
        .service(delete_comment)
        .service(audit_comment)
        .service(send_notification)
        .service(get_detailed_stats)
        .service(get_audit_logs);
}
