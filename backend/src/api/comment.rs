use actix_web::{delete, web, HttpRequest, HttpResponse};
use uuid::Uuid;

use crate::db::AppState;
use crate::models::{CurrentUser, UserRole};
use crate::services::{AuditLogService, CommentService};
use crate::utils::forbidden;

/// 删除评论
#[delete("/comments/{comment_id}")]
pub async fn delete_comment(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse, crate::services::CommentError> {
    let comment_id = path.into_inner();
    let is_admin = user.role == UserRole::Admin;

    // 删除成功返回 true；评论不存在或无权限返回 false（返回 403）
    if !CommentService::delete_comment(&state.pool, comment_id, user.id, is_admin).await? {
        return Ok(forbidden("无权删除该评论"));
    }

    // 记录审计日志
    let ip_address = req.peer_addr().map(|addr| addr.ip().to_string());
    if let Err(e) = AuditLogService::log_delete_comment(
        &state.pool,
        user.id,
        comment_id,
        is_admin,
        ip_address.as_deref(),
    )
    .await
    {
        log::warn!(
            "[Audit] 记录删除评论日志失败 | comment_id={}, error={}",
            comment_id,
            e
        );
    }

    Ok(HttpResponse::NoContent().finish())
}

/// 配置评论路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(delete_comment);
}
