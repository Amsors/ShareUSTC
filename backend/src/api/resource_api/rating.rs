use actix_web::{delete, get, post, web, HttpRequest, HttpResponse};
use uuid::Uuid;

use crate::db::AppState;
use crate::models::{CreateRatingRequest, CurrentUser};
use crate::services::{AuditLogService, RatingService, ResourceService};

/// 提交评分
#[post("/resources/{resource_id}/rate")]
pub async fn rate_resource(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    request: web::Json<CreateRatingRequest>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let resource_id = path.into_inner();

    // 获取资源信息（用于审计日志）；资源不存在时经 ResourceError 冒泡为 404
    let resource_detail = ResourceService::get_resource_detail(&state.pool, resource_id).await?;

    let overall_quality = request.overall_quality;

    // 评分校验错误 400、数据库错误 500，均经 RatingError 的 ResponseError 生成
    let rating = RatingService::create_or_update_rating(
        &state.pool,
        resource_id,
        user.id,
        request.into_inner(),
    )
    .await?;

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

    Ok(HttpResponse::Ok().json(rating))
}

/// 获取当前用户的评分
#[get("/resources/{resource_id}/rate")]
pub async fn get_my_rating(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, crate::services::RatingError> {
    let resource_id = path.into_inner();

    let rating = RatingService::get_user_rating(&state.pool, resource_id, user.id).await?;
    Ok(HttpResponse::Ok().json(rating))
}

/// 获取资源评分信息（包含所有维度的平均分，支持未登录用户）
#[get("/resources/{resource_id}/ratings")]
pub async fn get_resource_ratings(
    state: web::Data<AppState>,
    user: Option<web::ReqData<CurrentUser>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, crate::services::RatingError> {
    let resource_id = path.into_inner();
    let user_id = user.map(|u| u.id);

    let info = RatingService::get_resource_rating_info(&state.pool, resource_id, user_id).await?;
    Ok(HttpResponse::Ok().json(info))
}

/// 删除评分
#[delete("/resources/{resource_id}/rate")]
pub async fn delete_rating(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, crate::services::RatingError> {
    let resource_id = path.into_inner();

    RatingService::delete_rating(&state.pool, resource_id, user.id).await?;
    Ok(HttpResponse::NoContent().finish())
}
