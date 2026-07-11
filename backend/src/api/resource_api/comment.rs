use actix_web::{get, post, web, HttpRequest, HttpResponse};
use uuid::Uuid;

use crate::db::AppState;
use crate::models::{CommentListQuery, CreateCommentRequest, CurrentUser};
use crate::services::{AuditLogService, CommentService, ResourceService};

/// 获取评论列表（公开接口，不需要登录）
#[get("/resources/{resource_id}/comments")]
pub async fn get_comments(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    query: web::Query<CommentListQuery>,
) -> Result<HttpResponse, crate::services::CommentError> {
    let resource_id = path.into_inner();

    let comments =
        CommentService::get_comments(&state.pool, resource_id, query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(comments))
}

/// 发表评论
#[post("/resources/{resource_id}/comments")]
pub async fn create_comment(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    request: web::Json<CreateCommentRequest>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let resource_id = path.into_inner();

    // 获取资源信息（用于审计日志）；资源不存在时经 ResourceError 冒泡为 404
    let resource_detail = ResourceService::get_resource_detail(&state.pool, resource_id).await?;

    let comment =
        CommentService::create_comment(&state.pool, resource_id, user.id, request.into_inner())
            .await?;

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

    Ok(HttpResponse::Created().json(comment))
}
