use actix_web::{delete, get, web, HttpRequest, HttpResponse, Responder};
use uuid::Uuid;

use crate::db::AppState;
use crate::models::CurrentUser;
use crate::services::{AdminService, AuditLogService, FavoriteService};

use super::{
    check_admin,
    utils::{handle_admin_error, handle_resource_error},
};

/// 获取管理员的收藏夹列表
#[get("/admin/favorites")]
async fn get_admin_favorites(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
) -> impl Responder {
    let user = current_user.into_inner();
    log::info!("[Admin] 获取收藏夹列表 | admin_id={}", user.id);

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    match FavoriteService::get_user_favorites(&data.pool, user.id).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => handle_resource_error(e),
    }
}

/// 删除收藏夹内的所有资源
#[delete("/admin/favorites/{favorite_id}/resources")]
async fn delete_all_favorite_resources(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> impl Responder {
    let user = current_user.into_inner();

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    let favorite_id = path.into_inner();
    log::info!(
        "[Admin] 删除收藏夹内所有资源 | admin_id={}, favorite_id={}",
        user.id,
        favorite_id
    );

    match AdminService::delete_all_favorite_resources(&data.pool, &user, &data.storage, favorite_id)
        .await
    {
        Ok(result) => {
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

            HttpResponse::Ok().json(result)
        }
        Err(e) => handle_admin_error(e),
    }
}

/// 配置路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_admin_favorites)
        .service(delete_all_favorite_resources);
}
