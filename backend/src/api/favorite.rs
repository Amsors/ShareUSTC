use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};
use uuid::Uuid;

use crate::config::Config;
use crate::db::AppState;
use crate::models::{
    AddToFavoriteRequest, CreateFavoriteRequest, CurrentUser, UpdateFavoriteRequest,
};
use crate::services::{AuditLogService, FavoriteService, ResourceError};
use crate::utils::{
    bad_request, build_content_disposition, conflict, forbidden, internal_error, not_found,
};

/// 创建收藏夹
#[post("/favorites")]
pub async fn create_favorite(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    request: web::Json<CreateFavoriteRequest>,
    req: HttpRequest,
) -> impl Responder {
    log::info!(
        "[Favorite] 创建收藏夹 | user_id={}, name={}",
        user.id,
        request.name
    );

    let favorite_name = request.name.clone();
    match FavoriteService::create_favorite(&state.pool, user.id, request.into_inner()).await {
        Ok(response) => {
            log::info!(
                "[Favorite] 收藏夹创建成功 | favorite_id={}, user_id={}",
                response.id,
                user.id
            );

            // 记录审计日志
            let ip_address = req.peer_addr().map(|addr| addr.ip().to_string());
            if let Err(e) = AuditLogService::log_create_favorite(
                &state.pool,
                user.id,
                response.id,
                &favorite_name,
                ip_address.as_deref(),
            )
            .await
            {
                log::warn!(
                    "[Audit] 记录创建收藏夹日志失败 | favorite_id={}, error={}",
                    response.id,
                    e
                );
            }

            HttpResponse::Created().json(response)
        }
        Err(e) => {
            log::warn!(
                "[Favorite] 创建收藏夹失败 | user_id={}, error={}",
                user.id,
                e
            );
            match e {
                ResourceError::ValidationError(msg) => bad_request(&msg),
                ResourceError::NotFound(msg) => not_found(&msg),
                ResourceError::Unauthorized(msg) => forbidden(&msg),
                _ => internal_error("创建失败"),
            }
        }
    }
}

/// 获取我的收藏夹列表
#[get("/favorites")]
pub async fn get_my_favorites(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
) -> impl Responder {
    log::debug!("[Favorite] 获取收藏夹列表 | user_id={}", user.id);

    match FavoriteService::get_user_favorites(&state.pool, user.id).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            log::warn!(
                "[Favorite] 获取收藏夹列表失败 | user_id={}, error={}",
                user.id,
                e
            );
            internal_error("获取收藏夹列表失败")
        }
    }
}

/// 获取收藏夹详情
#[get("/favorites/{favorite_id}")]
pub async fn get_favorite_detail(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let favorite_id = path.into_inner();

    log::debug!(
        "[Favorite] 获取收藏夹详情 | favorite_id={}, user_id={}",
        favorite_id,
        user.id
    );

    match FavoriteService::get_favorite_detail(&state.pool, favorite_id, user.id).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            log::warn!(
                "[Favorite] 获取收藏夹详情失败 | favorite_id={}, user_id={}, error={}",
                favorite_id,
                user.id,
                e
            );
            match e {
                ResourceError::NotFound(msg) => not_found(&msg),
                ResourceError::Unauthorized(msg) => forbidden(&msg),
                _ => internal_error("获取收藏夹详情失败"),
            }
        }
    }
}

/// 更新收藏夹
#[put("/favorites/{favorite_id}")]
pub async fn update_favorite(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    request: web::Json<UpdateFavoriteRequest>,
) -> impl Responder {
    let favorite_id = path.into_inner();

    log::info!(
        "[Favorite] 更新收藏夹 | favorite_id={}, user_id={}",
        favorite_id,
        user.id
    );

    match FavoriteService::update_favorite(&state.pool, favorite_id, user.id, request.into_inner())
        .await
    {
        Ok(_) => {
            log::info!(
                "[Favorite] 收藏夹更新成功 | favorite_id={}, user_id={}",
                favorite_id,
                user.id
            );
            HttpResponse::NoContent().finish()
        }
        Err(e) => {
            log::warn!(
                "[Favorite] 收藏夹更新失败 | favorite_id={}, user_id={}, error={}",
                favorite_id,
                user.id,
                e
            );
            match e {
                ResourceError::ValidationError(msg) => bad_request(&msg),
                ResourceError::NotFound(msg) => not_found(&msg),
                ResourceError::Unauthorized(msg) => forbidden(&msg),
                _ => internal_error("更新失败"),
            }
        }
    }
}

/// 删除收藏夹
#[delete("/favorites/{favorite_id}")]
pub async fn delete_favorite(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let favorite_id = path.into_inner();

    log::info!(
        "[Favorite] 删除收藏夹 | favorite_id={}, user_id={}",
        favorite_id,
        user.id
    );

    match FavoriteService::delete_favorite(&state.pool, favorite_id, user.id).await {
        Ok(_) => {
            log::info!(
                "[Favorite] 收藏夹删除成功 | favorite_id={}, user_id={}",
                favorite_id,
                user.id
            );
            HttpResponse::NoContent().finish()
        }
        Err(e) => {
            log::warn!(
                "[Favorite] 收藏夹删除失败 | favorite_id={}, user_id={}, error={}",
                favorite_id,
                user.id,
                e
            );
            match e {
                ResourceError::NotFound(msg) => not_found(&msg),
                ResourceError::Unauthorized(msg) => forbidden(&msg),
                _ => internal_error("删除失败"),
            }
        }
    }
}

/// 添加资源到收藏夹
#[post("/favorites/{favorite_id}/resources")]
pub async fn add_resource_to_favorite(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    request: web::Json<AddToFavoriteRequest>,
) -> impl Responder {
    let favorite_id = path.into_inner();

    log::info!(
        "[Favorite] 添加资源到收藏夹 | favorite_id={}, user_id={}, resource_id={}",
        favorite_id,
        user.id,
        request.resource_id
    );

    match FavoriteService::add_resource_to_favorite(
        &state.pool,
        favorite_id,
        user.id,
        request.into_inner(),
    )
    .await
    {
        Ok(_) => {
            log::info!(
                "[Favorite] 资源添加到收藏夹成功 | favorite_id={}, user_id={}",
                favorite_id,
                user.id
            );
            HttpResponse::Created().finish()
        }
        Err(e) => {
            log::warn!(
                "[Favorite] 添加资源到收藏夹失败 | favorite_id={}, user_id={}, error={}",
                favorite_id,
                user.id,
                e
            );
            match e {
                ResourceError::ValidationError(msg) => bad_request(&msg),
                ResourceError::NotFound(msg) => not_found(&msg),
                ResourceError::Unauthorized(msg) => forbidden(&msg),
                ResourceError::Conflict(msg) => conflict(&msg),
                _ => internal_error("添加失败"),
            }
        }
    }
}

/// 从收藏夹移除资源
#[delete("/favorites/{favorite_id}/resources/{resource_id}")]
pub async fn remove_resource_from_favorite(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<(Uuid, Uuid)>,
) -> impl Responder {
    let (favorite_id, resource_id) = path.into_inner();
    log::info!(
        "[Favorite] 从收藏夹移除资源 | favorite_id={}, resource_id={}, user_id={}",
        favorite_id,
        resource_id,
        user.id
    );

    match FavoriteService::remove_resource_from_favorite(
        &state.pool,
        favorite_id,
        resource_id,
        user.id,
    )
    .await
    {
        Ok(_) => {
            log::info!(
                "[Favorite] 资源从收藏夹移除成功 | favorite_id={}, resource_id={}, user_id={}",
                favorite_id,
                resource_id,
                user.id
            );
            HttpResponse::NoContent().finish()
        }
        Err(e) => {
            log::warn!("[Favorite] 从收藏夹移除资源失败 | favorite_id={}, resource_id={}, user_id={}, error={}",
                favorite_id, resource_id, user.id, e);
            match e {
                ResourceError::NotFound(msg) => not_found(&msg),
                ResourceError::Unauthorized(msg) => forbidden(&msg),
                _ => internal_error("移除失败"),
            }
        }
    }
}

/// 检查资源收藏状态
#[get("/favorites/check/{resource_id}")]
pub async fn check_resource_in_favorite(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let resource_id = path.into_inner();

    match FavoriteService::check_resource_in_favorites(&state.pool, user.id, resource_id).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => match e {
            ResourceError::NotFound(msg) => not_found(&msg),
            _ => internal_error("检查失败"),
        },
    }
}

/// 打包下载收藏夹
#[get("/favorites/{favorite_id}/download")]
pub async fn download_favorite(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> impl Responder {
    let favorite_id = path.into_inner();

    // 首先获取收藏夹详情
    let favorite_detail =
        match FavoriteService::get_favorite_detail(&state.pool, favorite_id, user.id).await {
            Ok(detail) => detail,
            Err(e) => {
                return match e {
                    ResourceError::NotFound(msg) => not_found(&msg),
                    ResourceError::Unauthorized(msg) => forbidden(&msg),
                    _ => internal_error("获取收藏夹信息失败"),
                };
            }
        };

    let favorite_name = favorite_detail.name.clone();
    let resource_count = favorite_detail.resource_count as usize;

    // 打包下载
    // 加载配置用于创建存储后端（支持混合存储）
    let config = Config::from_env();
    match FavoriteService::pack_favorite_resources(
        &state.pool,
        &state.storage,
        &config,
        favorite_id,
        user.id,
        &favorite_name,
    )
    .await
    {
        Ok((zip_data, filename)) => {
            let download_size = zip_data.len() as i64;

            // 记录审计日志
            let ip_address = req.peer_addr().map(|addr| addr.ip().to_string());
            if let Err(e) = AuditLogService::log_pack_download(
                &state.pool,
                user.id,
                favorite_id,
                &favorite_name,
                download_size,
                resource_count,
                ip_address.as_deref(),
            )
            .await
            {
                log::warn!(
                    "[Audit] 记录打包下载日志失败 | favorite_id={}, error={}",
                    favorite_id,
                    e
                );
            }

            // 构建 Content-Disposition 头，支持中文文件名
            let content_disposition = build_content_disposition(&filename);

            HttpResponse::Ok()
                .content_type("application/zip")
                .append_header(("Content-Disposition", content_disposition))
                .body(zip_data)
        }
        Err(e) => match e {
            ResourceError::ValidationError(msg) => bad_request(&msg),
            ResourceError::NotFound(msg) => not_found(&msg),
            ResourceError::Unauthorized(msg) => forbidden(&msg),
            ResourceError::FileError(msg) => internal_error(&msg),
            _ => internal_error("打包下载失败"),
        },
    }
}

/// 配置收藏夹路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(create_favorite)
        .service(get_my_favorites)
        .service(get_favorite_detail)
        .service(update_favorite)
        .service(delete_favorite)
        .service(add_resource_to_favorite)
        .service(remove_resource_from_favorite)
        .service(check_resource_in_favorite)
        .service(download_favorite);
}
