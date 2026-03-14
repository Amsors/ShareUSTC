use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use uuid::Uuid;

use crate::db::AppState;
use crate::models::CurrentUser;
use crate::services::{AuditLogService, LikeService, ResourceError, ResourceService};
use crate::utils::internal_error;

/// 点赞/取消点赞
#[post("/resources/{resource_id}/like")]
pub async fn toggle_like(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
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
                ResourceError::NotFound(msg) => crate::utils::not_found(&msg),
                _ => internal_error("获取资源详情失败"),
            };
        }
    };

    match LikeService::toggle_like(&state.pool, resource_id, user.id).await {
        Ok(result) => {
            // 记录审计日志
            let ip_address = req.peer_addr().map(|addr| addr.ip().to_string());
            if let Err(e) = AuditLogService::log_like_resource(
                &state.pool,
                user.id,
                resource_id,
                &resource_detail.title,
                result.is_liked,
                ip_address.as_deref(),
            )
            .await
            {
                log::warn!(
                    "[Audit] 记录点赞日志失败 | resource_id={}, error={}",
                    resource_id,
                    e
                );
            }

            // 转换为 camelCase 的响应结构
            let response_data = crate::models::LikeToggleResponse {
                is_liked: result.is_liked,
                like_count: result.like_count,
                message: result.message.clone(),
            };
            HttpResponse::Ok().json(response_data)
        }
        Err(e) => {
            log::warn!(
                "[Resource] 点赞操作失败 | resource_id={}, user_id={}, error={}",
                resource_id,
                user.id,
                e
            );
            internal_error("操作失败")
        }
    }
}

/// 获取点赞状态
#[get("/resources/{resource_id}/like")]
pub async fn get_like_status(
    state: web::Data<AppState>,
    user: Option<web::ReqData<CurrentUser>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let resource_id = path.into_inner();

    // 如果有用户登录，检查该用户的点赞状态；否则返回未点赞
    let (is_liked, like_count) = if let Some(user) = user {
        match LikeService::check_like_status(&state.pool, resource_id, user.id).await {
            Ok(status) => (status.is_liked, status.like_count),
            Err(e) => {
                log::warn!(
                    "[Resource] 获取点赞状态失败 | resource_id={}, user_id={}, error={}",
                    resource_id,
                    user.id,
                    e
                );
                (false, 0)
            }
        }
    } else {
        // 未登录用户，获取点赞数但不显示已点赞
        match LikeService::get_like_count(&state.pool, resource_id).await {
            Ok(count) => (false, count),
            Err(e) => {
                log::warn!(
                    "[Resource] 获取点赞数失败 | resource_id={}, error={}",
                    resource_id,
                    e
                );
                (false, 0)
            }
        }
    };

    // 使用 LikeStatusResponse 结构体，确保字段名使用 camelCase
    let response_data = crate::models::LikeStatusResponse {
        is_liked,
        like_count,
    };

    HttpResponse::Ok().json(response_data)
}
