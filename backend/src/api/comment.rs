use actix_web::{delete, web, HttpResponse, Responder};
use uuid::Uuid;

use crate::db::AppState;
use crate::models::{CurrentUser, UserRole};
use crate::services::{CommentService, ResourceError};
use crate::utils::{forbidden, not_found, internal_error};

/// 删除评论
#[delete("/comments/{comment_id}")]
pub async fn delete_comment(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let comment_id = path.into_inner();
    let is_admin = user.role == UserRole::Admin;

    match CommentService::delete_comment(&state.pool, comment_id, user.id, is_admin).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => forbidden("无权删除该评论"),
        Err(ResourceError::NotFound(msg)) => not_found(&msg),
        Err(e) => {
            log::warn!("删除评论失败: {}", e);
            internal_error("删除失败")
        }
    }
}

/// 配置评论路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(delete_comment);
}
