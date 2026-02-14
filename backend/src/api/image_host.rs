use actix_multipart::Multipart;
use actix_web::{delete, get, post, web, HttpResponse, Responder};
use futures_util::StreamExt;
use uuid::Uuid;

use crate::db::AppState;
use crate::models::CurrentUser;
use crate::services::{ImageError, ImageService};
use crate::utils::{bad_request, created, forbidden, internal_error, no_content, not_found};

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
                return bad_request("解析上传数据失败");
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
                        return bad_request("读取文件数据失败");
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
            return bad_request("请选择要上传的图片");
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
        Ok(response) => created(response),
        Err(e) => {
            log::warn!("上传图片失败: {}", e);
            match e {
                ImageError::ValidationError(msg) => bad_request(&msg),
                ImageError::FileError(msg) => internal_error(&msg),
                ImageError::DatabaseError(msg) => internal_error(&msg),
                _ => internal_error("上传失败"),
            }
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
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            log::warn!("获取图片列表失败: {}", e);
            internal_error("获取图片列表失败")
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
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            log::warn!("获取图片信息失败: {}", e);
            match e {
                ImageError::NotFound(msg) => not_found(&msg),
                _ => internal_error("获取图片信息失败"),
            }
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
        Ok(_) => no_content(),
        Err(e) => {
            log::warn!("删除图片失败: {}", e);
            match e {
                ImageError::NotFound(msg) => not_found(&msg),
                ImageError::Unauthorized(msg) => forbidden(&msg),
                _ => internal_error("删除失败"),
            }
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
