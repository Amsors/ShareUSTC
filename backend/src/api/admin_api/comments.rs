use actix_web::{delete, get, put, web, HttpRequest, HttpResponse, Responder};
use uuid::Uuid;

use crate::db::AppState;
use crate::models::CurrentUser;
use crate::services::{AdminService, AuditLogService};
use crate::utils::no_content;

use super::{check_admin, utils::handle_admin_error};

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

    match AdminService::get_comment_list(&data.pool, page, per_page, audit_status).await {
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
    req: HttpRequest,
) -> impl Responder {
    let user = current_user.into_inner();

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    let comment_id = path.into_inner();
    log::info!(
        "[Admin] 删除评论 | admin_id={}, comment_id={}",
        user.id,
        comment_id
    );

    match AdminService::delete_comment(&data.pool, comment_id).await {
        Ok(_) => {
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
    log::info!(
        "[Admin] 审核评论 | admin_id={}, comment_id={}, status={}",
        user.id,
        comment_id,
        status
    );

    match AdminService::audit_comment(&data.pool, comment_id, status).await {
        Ok(_) => {
            log::info!(
                "[Admin] 评论审核完成 | admin_id={}, comment_id={}",
                user.id,
                comment_id
            );
            HttpResponse::Ok().json(serde_json::json!({
                "message": "评论审核完成"
            }))
        }
        Err(e) => handle_admin_error(e),
    }
}

/// 配置路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_comment_list)
        .service(delete_comment)
        .service(audit_comment);
}
