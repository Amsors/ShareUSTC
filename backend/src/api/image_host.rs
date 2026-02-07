use actix_multipart::Multipart;
use actix_web::{delete, get, post, web, HttpResponse, Responder};
use futures_util::StreamExt;
use uuid::Uuid;

use crate::db::AppState;
use crate::models::CurrentUser;
use crate::services::{ImageError, ImageService};

/// 上传图片
#[post("/images/upload")]
pub async fn upload_image(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    mut payload: Multipart,
) -> impl Responder {
    let mut file_data: Option<(String, Vec<u8>, Option<String>)> = None;

    // 解析multipart表单数据
    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(field) => field,
            Err(e) => {
                log::warn!("解析上传数据失败: {}", e);
                return HttpResponse::Ok().json(serde_json::json!({
                    "code": 400,
                    "message": "解析上传数据失败",
                    "data": null
                }));
            }
        };

        let content_disposition = field.content_disposition();
        let field_name = content_disposition
            .get_name()
            .unwrap_or("unknown");

        if field_name == "image" {
            // 获取文件名
            let filename = content_disposition
                .get_filename()
                .unwrap_or("unnamed.jpg")
                .to_string();

            // 获取MIME类型
            let mime_type = field.content_type().map(|m| m.to_string());

            // 读取文件数据
            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                match chunk {
                    Ok(bytes) => data.extend_from_slice(&bytes),
                    Err(e) => {
                        log::warn!("读取文件数据失败: {}", e);
                        return HttpResponse::Ok().json(serde_json::json!({
                            "code": 400,
                            "message": "读取文件数据失败",
                            "data": null
                        }));
                    }
                }
            }

            file_data = Some((filename, data, mime_type));
        }
    }

    // 检查是否有文件数据
    let (filename, data, mime_type) = match file_data {
        Some(data) => data,
        None => {
            return HttpResponse::Ok().json(serde_json::json!({
                "code": 400,
                "message": "请选择要上传的图片",
                "data": null
            }));
        }
    };

    // 调用服务上传图片
    match ImageService::upload_image(
        &state.pool,
        &user,
        &filename,
        data,
        mime_type.as_deref(),
    )
    .await
    {
        Ok(response) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "上传成功",
            "data": response
        })),
        Err(e) => {
            log::warn!("上传图片失败: {}", e);
            let (code, message) = match e {
                ImageError::ValidationError(msg) => (400, msg),
                ImageError::FileError(msg) => (500, msg),
                ImageError::DatabaseError(msg) => (500, msg),
                _ => (500, "上传失败".to_string()),
            };
            HttpResponse::Ok().json(serde_json::json!({
                "code": code,
                "message": message,
                "data": null
            }))
        }
    }
}

/// 获取当前用户的图片列表
#[get("/images")]
pub async fn get_my_images(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    query: web::Query<ImageListQuery>,
) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20).min(100);

    match ImageService::get_user_images(&state.pool, user.id, page, per_page).await {
        Ok(response) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "获取成功",
            "data": response
        })),
        Err(e) => {
            log::warn!("获取图片列表失败: {}", e);
            HttpResponse::Ok().json(serde_json::json!({
                "code": 500,
                "message": "获取图片列表失败",
                "data": null
            }))
        }
    }
}

/// 获取单张图片信息
#[get("/images/{image_id}")]
pub async fn get_image_info(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let image_id = path.into_inner();

    match ImageService::get_image_by_id(&state.pool, image_id).await {
        Ok(response) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "获取成功",
            "data": response
        })),
        Err(e) => {
            log::warn!("获取图片信息失败: {}", e);
            let (code, message) = match e {
                ImageError::NotFound(msg) => (404, msg),
                _ => (500, "获取图片信息失败".to_string()),
            };
            HttpResponse::Ok().json(serde_json::json!({
                "code": code,
                "message": message,
                "data": null
            }))
        }
    }
}

/// 删除图片
#[delete("/images/{image_id}")]
pub async fn delete_image(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let image_id = path.into_inner();

    match ImageService::delete_image(&state.pool, &user, image_id).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "删除成功",
            "data": null
        })),
        Err(e) => {
            log::warn!("删除图片失败: {}", e);
            let (code, message) = match e {
                ImageError::NotFound(msg) => (404, msg),
                ImageError::Unauthorized(msg) => (403, msg),
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

/// 图片列表查询参数
#[derive(Debug, serde::Deserialize)]
pub struct ImageListQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

/// 配置图床路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(upload_image)
        .service(get_my_images)
        .service(get_image_info)
        .service(delete_image);
}
