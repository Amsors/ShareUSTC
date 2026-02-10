use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use uuid::Uuid;

use crate::db::AppState;
use crate::models::{
    AddToFavoriteRequest, CreateFavoriteRequest, CurrentUser, UpdateFavoriteRequest,
};
use crate::services::FavoriteService;

/// 创建收藏夹
#[post("/favorites")]
pub async fn create_favorite(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    request: web::Json<CreateFavoriteRequest>,
) -> impl Responder {
    match FavoriteService::create_favorite(&state.pool, user.id, request.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "创建成功",
            "data": response
        })),
        Err(e) => {
            let (code, message) = match e {
                crate::services::ResourceError::ValidationError(msg) => (400, msg),
                crate::services::ResourceError::NotFound(msg) => (404, msg),
                _ => (500, "创建失败".to_string()),
            };
            HttpResponse::Ok().json(serde_json::json!({
                "code": code,
                "message": message,
                "data": null
            }))
        }
    }
}

/// 获取我的收藏夹列表
#[get("/favorites")]
pub async fn get_my_favorites(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
) -> impl Responder {
    match FavoriteService::get_user_favorites(&state.pool, user.id).await {
        Ok(response) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "success",
            "data": response
        })),
        Err(e) => {
            log::warn!("获取收藏夹列表失败: {}", e);
            HttpResponse::Ok().json(serde_json::json!({
                "code": 500,
                "message": "获取收藏夹列表失败",
                "data": null
            }))
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

    match FavoriteService::get_favorite_detail(&state.pool, favorite_id, user.id).await {
        Ok(response) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "success",
            "data": response
        })),
        Err(e) => {
            let (code, message) = match e {
                crate::services::ResourceError::NotFound(msg) => (404, msg),
                _ => (500, "获取收藏夹详情失败".to_string()),
            };
            HttpResponse::Ok().json(serde_json::json!({
                "code": code,
                "message": message,
                "data": null
            }))
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

    match FavoriteService::update_favorite(&state.pool, favorite_id, user.id, request.into_inner()).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "更新成功",
            "data": null
        })),
        Err(e) => {
            let (code, message) = match e {
                crate::services::ResourceError::ValidationError(msg) => (400, msg),
                crate::services::ResourceError::NotFound(msg) => (404, msg),
                _ => (500, "更新失败".to_string()),
            };
            HttpResponse::Ok().json(serde_json::json!({
                "code": code,
                "message": message,
                "data": null
            }))
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

    match FavoriteService::delete_favorite(&state.pool, favorite_id, user.id).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "删除成功",
            "data": null
        })),
        Err(e) => {
            let (code, message) = match e {
                crate::services::ResourceError::NotFound(msg) => (404, msg),
                _ => (500, "删除失败".to_string()),
            };
            HttpResponse::Ok().json(serde_json::json!({
                "code": code,
                "message": message,
                "data": null
            }))
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

    match FavoriteService::add_resource_to_favorite(&state.pool, favorite_id, user.id, request.into_inner()).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "添加成功",
            "data": null
        })),
        Err(e) => {
            let (code, message) = match e {
                crate::services::ResourceError::ValidationError(msg) => (400, msg),
                crate::services::ResourceError::NotFound(msg) => (404, msg),
                _ => (500, "添加失败".to_string()),
            };
            HttpResponse::Ok().json(serde_json::json!({
                "code": code,
                "message": message,
                "data": null
            }))
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

    match FavoriteService::remove_resource_from_favorite(&state.pool, favorite_id, resource_id, user.id).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "移除成功",
            "data": null
        })),
        Err(e) => {
            let (code, message) = match e {
                crate::services::ResourceError::NotFound(msg) => (404, msg),
                _ => (500, "移除失败".to_string()),
            };
            HttpResponse::Ok().json(serde_json::json!({
                "code": code,
                "message": message,
                "data": null
            }))
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
        Ok(response) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "success",
            "data": response
        })),
        Err(e) => {
            let (code, message) = match e {
                crate::services::ResourceError::NotFound(msg) => (404, msg),
                _ => (500, "检查失败".to_string()),
            };
            HttpResponse::Ok().json(serde_json::json!({
                "code": code,
                "message": message,
                "data": null
            }))
        }
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
        .service(check_resource_in_favorite);
}
