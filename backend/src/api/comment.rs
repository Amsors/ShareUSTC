use actix_web::{delete, web, HttpResponse, Responder};
use uuid::Uuid;

use crate::db::AppState;
use crate::models::{CurrentUser, UserRole};
use crate::services::CommentService;

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
        Ok(true) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "删除成功",
            "data": null
        })),
        Ok(false) => HttpResponse::Ok().json(serde_json::json!({
            "code": 403,
            "message": "无权删除该评论",
            "data": null
        })),
        Err(e) => {
            log::warn!("删除评论失败: {}", e);
            let message = match e {
                crate::services::ResourceError::NotFound(msg) => msg,
                _ => "删除失败".to_string(),
            };
            HttpResponse::Ok().json(serde_json::json!({
                "code": 500,
                "message": message,
                "data": null
            }))
        }
    }
}

/// 配置评论路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(delete_comment);
}
