use actix_web::{delete, get, put, web, HttpRequest, HttpResponse};
use uuid::Uuid;

use crate::db::AppState;
use crate::models::CurrentUser;
use crate::services::{AdminService, AuditLogService};
use crate::utils::no_content;

use super::check_admin;

/// 获取评论列表
#[get("/admin/comments")]
async fn get_comment_list(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let user = current_user.into_inner();
    log::info!("[Admin] 获取评论列表 | admin_id={}", user.id);

    check_admin(&user)?;

    let page = query
        .get("page")
        .and_then(|p| p.parse::<i32>().ok())
        .unwrap_or(1);
    let per_page = query
        .get("perPage")
        .and_then(|p| p.parse::<i32>().ok())
        .unwrap_or(20);
    let audit_status = query.get("auditStatus").cloned();

    let response = AdminService::get_comment_list(&data.pool, page, per_page, audit_status).await?;
    Ok(HttpResponse::Ok().json(response))
}

/// 删除评论
#[delete("/admin/comments/{comment_id}")]
async fn delete_comment(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let user = current_user.into_inner();

    check_admin(&user)?;

    let comment_id = path.into_inner();
    log::info!(
        "[Admin] 删除评论 | admin_id={}, comment_id={}",
        user.id,
        comment_id
    );

    AdminService::delete_comment(&data.pool, comment_id).await?;

    log::info!(
        "[Admin] 评论删除成功 | admin_id={}, comment_id={}",
        user.id,
        comment_id
    );

    // 记录审计日志
    let ip_address = req.peer_addr().map(|addr| addr.ip().to_string());
    if let Err(e) = AuditLogService::log_delete_comment(
        &data.pool,
        user.id,
        comment_id,
        true, // is_admin
        ip_address.as_deref(),
    )
    .await
    {
        log::warn!(
            "[Audit] 记录删除评论日志失败 | admin_id={}, comment_id={}, error={}",
            user.id,
            comment_id,
            e
        );
    }

    Ok(no_content())
}

/// 审核评论
#[put("/admin/comments/{comment_id}/audit")]
async fn audit_comment(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    req: web::Json<std::collections::HashMap<String, String>>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let user = current_user.into_inner();

    check_admin(&user)?;

    let comment_id = path.into_inner();
    let status = req.get("status").cloned().unwrap_or_default();
    log::info!(
        "[Admin] 审核评论 | admin_id={}, comment_id={}, status={}",
        user.id,
        comment_id,
        status
    );

    AdminService::audit_comment(&data.pool, comment_id, status).await?;

    log::info!(
        "[Admin] 评论审核完成 | admin_id={}, comment_id={}",
        user.id,
        comment_id
    );
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "评论审核完成"
    })))
}

/// 配置路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_comment_list)
        .service(delete_comment)
        .service(audit_comment);
}
