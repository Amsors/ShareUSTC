use actix_web::{delete, get, post, web, HttpRequest, HttpResponse, Responder};
use uuid::Uuid;

use crate::db::AppState;
use crate::models::{CreateRatingRequest, CurrentUser};
use crate::services::{AuditLogService, RatingService, ResourceError, ResourceService};
use crate::utils::{bad_request, internal_error, not_found};

/// 提交评分
#[post("/resources/{resource_id}/rate")]
pub async fn rate_resource(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    request: web::Json<CreateRatingRequest>,
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

    let overall_quality = request.overall_quality;

    match RatingService::create_or_update_rating(
        &state.pool,
        resource_id,
        user.id,
        request.into_inner(),
    )
    .await
    {
        Ok(rating) => {
            // 记录审计日志
            let ip_address = req.peer_addr().map(|addr| addr.ip().to_string());
            if let Err(e) = AuditLogService::log_rate_resource(
                &state.pool,
                user.id,
                resource_id,
                &resource_detail.title,
                overall_quality,
                ip_address.as_deref(),
            )
            .await
            {
                log::warn!(
                    "[Audit] 记录评分日志失败 | resource_id={}, error={}",
                    resource_id,
                    e
                );
            }

            HttpResponse::Ok().json(rating)
        }
        Err(e) => {
            log::warn!(
                "[Resource] 评分失败 | resource_id={}, user_id={}, error={}",
                resource_id,
                user.id,
                e
            );
            bad_request("评分失败")
        }
    }
}

/// 获取当前用户的评分
#[get("/resources/{resource_id}/rate")]
pub async fn get_my_rating(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let resource_id = path.into_inner();

    match RatingService::get_user_rating(&state.pool, resource_id, user.id).await {
        Ok(rating) => HttpResponse::Ok().json(rating),
        Err(e) => {
            log::warn!(
                "[Resource] 获取评分失败 | resource_id={}, user_id={}, error={}",
                resource_id,
                user.id,
                e
            );
            internal_error("获取失败")
        }
    }
}

/// 获取资源评分信息（包含所有维度的平均分，支持未登录用户）
#[get("/resources/{resource_id}/ratings")]
pub async fn get_resource_ratings(
    state: web::Data<AppState>,
    user: Option<web::ReqData<CurrentUser>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let resource_id = path.into_inner();
    let user_id = user.map(|u| u.id);

    match RatingService::get_resource_rating_info(&state.pool, resource_id, user_id).await {
        Ok(info) => HttpResponse::Ok().json(info),
        Err(e) => {
            log::warn!(
                "[Resource] 获取资源评分信息失败 | resource_id={}, error={}",
                resource_id,
                e
            );
            internal_error("获取评分信息失败")
        }
    }
}

/// 删除评分
#[delete("/resources/{resource_id}/rate")]
pub async fn delete_rating(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let resource_id = path.into_inner();

    match RatingService::delete_rating(&state.pool, resource_id, user.id).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => {
            log::warn!(
                "[Resource] 删除评分失败 | resource_id={}, user_id={}, error={}",
                resource_id,
                user.id,
                e
            );
            internal_error("删除失败")
        }
    }
}
