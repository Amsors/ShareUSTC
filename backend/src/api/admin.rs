use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;

use crate::db::AppState;
use crate::models::CurrentUser;
use crate::services::{
    AdminService, AdminError, AuditResourceRequest, UpdateUserStatusRequest,
    AuditLogQuery,
};

/// 检查用户是否是管理员
fn check_admin(current_user: &CurrentUser) -> Result<(), AdminError> {
    if !matches!(current_user.role, crate::models::UserRole::Admin) {
        return Err(AdminError::Forbidden("需要管理员权限".to_string()));
    }
    Ok(())
}

/// 将AdminError转换为统一格式的HttpResponse
/// 统一返回 HTTP 200，业务错误码通过 JSON 中的 code 字段传递
fn handle_admin_error(err: AdminError) -> HttpResponse {
    let (code, message) = match err {
        AdminError::NotFound(msg) => (404, msg),
        AdminError::ValidationError(msg) => (400, msg),
        AdminError::Forbidden(msg) => (403, msg),
        AdminError::DatabaseError(msg) => {
            log::error!("数据库错误: {}", msg);
            (500, "服务器内部错误".to_string())
        }
    };

    HttpResponse::Ok().json(serde_json::json!({
        "code": code,
        "message": message,
        "data": null
    }))
}

/// 获取仪表盘统计数据
#[get("/admin/dashboard")]
async fn get_dashboard(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
) -> impl Responder {
    let user = current_user.into_inner();

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    match AdminService::get_dashboard_stats(&data.pool).await {
        Ok(stats) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "success",
            "data": stats
        })),
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
        Ok(response) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "success",
            "data": response
        })),
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

    // 禁止禁用自己
    if user_id == user.id {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "code": 400,
            "message": "不能禁用自己的账号",
            "data": null
        }));
    }

    match AdminService::update_user_status(&data.pool, user_id, req.is_active
    ).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "用户状态已更新",
            "data": null
        })),
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
        Ok(response) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "success",
            "data": response
        })),
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

    match AdminService::audit_resource(
        &data.pool,
        resource_id,
        req.status.clone(),
        req.reason.clone(),
    ).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "资源审核完成",
            "data": null
        })),
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
        Ok(response) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "success",
            "data": response
        })),
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

    match AdminService::delete_comment(&data.pool, comment_id).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "评论已删除",
            "data": null
        })),
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

    match AdminService::audit_comment(&data.pool, comment_id, status
    ).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "评论审核完成",
            "data": null
        })),
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

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    match AdminService::send_notification(&data.pool, req.into_inner()).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "通知发送成功",
            "data": null
        })),
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

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    match AdminService::get_detailed_stats(&data.pool).await {
        Ok(stats) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "success",
            "data": stats
        })),
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
        Ok(response) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "success",
            "data": response
        })),
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
