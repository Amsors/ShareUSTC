use actix_web::{delete, get, web, HttpRequest, HttpResponse};
use uuid::Uuid;

use crate::db::AppState;
use crate::models::CurrentUser;
use crate::services::{AdminService, AuditLogService, FavoriteService};

use super::check_admin;

/// 获取管理员的收藏夹列表
#[get("/admin/favorites")]
async fn get_admin_favorites(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let user = current_user.into_inner();
    log::info!("[Admin] 获取收藏夹列表 | admin_id={}", user.id);

    check_admin(&user)?;

    let response = FavoriteService::get_user_favorites(&data.pool, user.id).await?;
    Ok(HttpResponse::Ok().json(response))
}

/// 删除收藏夹内的所有资源
#[delete("/admin/favorites/{favorite_id}/resources")]
async fn delete_all_favorite_resources(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let user = current_user.into_inner();

    check_admin(&user)?;

    let favorite_id = path.into_inner();
    log::info!(
        "[Admin] 删除收藏夹内所有资源 | admin_id={}, favorite_id={}",
        user.id,
        favorite_id
    );

    let result =
        AdminService::delete_all_favorite_resources(&data.pool, &user, &data.storage, favorite_id)
            .await?;
    log::info!(
        "[Admin] 收藏夹内资源删除成功 | admin_id={}, favorite_id={}, deleted_count={}",
        user.id,
        favorite_id,
        result.deleted_count
    );

    // 记录审计日志
    let ip_address = req.peer_addr().map(|addr| addr.ip().to_string());
    if let Err(e) = AuditLogService::log_action(
        &data.pool,
        user.id,
        "delete_favorite_resources",
        Some("favorite"),
        Some(favorite_id),
        Some(serde_json::json!({
            "deleted_count": result.deleted_count,
            "favorite_name": result.favorite_name
        })),
        ip_address.as_deref(),
    )
    .await
    {
        log::warn!(
            "[Audit] 记录删除收藏夹资源日志失败 | admin_id={}, favorite_id={}, error={}",
            user.id,
            favorite_id,
            e
        );
    }

    Ok(HttpResponse::Ok().json(result))
}

/// 配置路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_admin_favorites)
        .service(delete_all_favorite_resources);
}
