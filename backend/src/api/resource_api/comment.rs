use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use uuid::Uuid;

use crate::db::AppState;
use crate::models::{CommentListQuery, CreateCommentRequest, CurrentUser};
use crate::services::{AuditLogService, CommentService, ResourceError, ResourceService};
use crate::utils::{bad_request, internal_error, not_found};

/// 获取评论列表（公开接口，不需要登录）
#[get("/resources/{resource_id}/comments")]
pub async fn get_comments(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    query: web::Query<CommentListQuery>,
) -> impl Responder {
    let resource_id = path.into_inner();

    match CommentService::get_comments(&state.pool, resource_id, query.into_inner()).await {
        Ok(comments) => HttpResponse::Ok().json(comments),
        Err(e) => {
            log::warn!(
                "[Resource] 获取评论失败 | resource_id={}, error={}",
                resource_id,
                e
            );
            match e {
                ResourceError::NotFound(msg) => not_found(&msg),
                _ => internal_error("获取评论失败"),
            }
        }
    }
}

/// 发表评论
#[post("/resources/{resource_id}/comments")]
pub async fn create_comment(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    request: web::Json<CreateCommentRequest>,
    req: HttpRequest,
) -> impl Responder {
    let resource_id = path.into_inner();

    // 获取资源信息（用于审计日志）
    let resource_detail = match ResourceService::get_resource_detail(&state.pool, resource_id).await
    {
        Ok(detail) => detail,
        Err(e) => {
            log::warn!(
                "[Resource] 获取资源详情失败 | resource_id={}, user_id={}, error={}",
                resource_id,
                user.id,
                e
            );
            return match e {
                ResourceError::NotFound(msg) => not_found(&msg),
                _ => internal_error("获取资源详情失败"),
            };
        }
    };

    match CommentService::create_comment(&state.pool, resource_id, user.id, request.into_inner())
        .await
    {
        Ok(comment) => {
            // 记录审计日志
            let ip_address = req.peer_addr().map(|addr| addr.ip().to_string());
            if let Err(e) = AuditLogService::log_create_comment(
                &state.pool,
                user.id,
                resource_id,
                comment.id,
                &resource_detail.title,
                ip_address.as_deref(),
            )
            .await
            {
                log::warn!(
                    "[Audit] 记录发表评论日志失败 | comment_id={}, error={}",
                    comment.id,
                    e
                );
            }

            HttpResponse::Created().json(comment)
        }
        Err(e) => {
            log::warn!(
                "[Resource] 发表评论失败 | resource_id={}, user_id={}, error={}",
                resource_id,
                user.id,
                e
            );
            match e {
                ResourceError::ValidationError(msg) => bad_request(&msg),
                ResourceError::NotFound(msg) => not_found(&msg),
                _ => internal_error("评论失败"),
            }
        }
    }
}
