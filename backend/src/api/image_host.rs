use actix_multipart::Multipart;
use actix_web::{delete, get, post, web, HttpResponse};
use futures_util::StreamExt;
use uuid::Uuid;

use crate::db::AppState;
use crate::models::CurrentUser;
use crate::services::{ImageError, ImageService};
use crate::utils::{bad_request, created, no_content};

/// 上传图片
#[post("/images/upload")]
pub async fn upload_image(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    mut payload: Multipart,
) -> Result<HttpResponse, ImageError> {
    let mut file_data: Option<(String, Vec<u8>, Option<String>)> = None;

    // 解析multipart表单数据
    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(field) => field,
            Err(e) => {
                log::warn!("解析上传数据失败: {}", e);
                return Ok(bad_request("解析上传数据失败"));
            }
        };

        let content_disposition = field.content_disposition();
        let field_name = content_disposition.get_name().unwrap_or("unknown");

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
                        return Ok(bad_request("读取文件数据失败"));
                    }
                }
            }

            file_data = Some((filename, data, mime_type));
        }
    }

    // 检查是否有文件数据
    let Some((filename, data, mime_type)) = file_data else {
        return Ok(bad_request("请选择要上传的图片"));
    };

    // 调用服务上传图片（使用注入的配置生成图片 URL，不再每请求解析环境变量）
    let response = ImageService::upload_image(
        &state.pool,
        &user,
        &state.storage,
        &state.config,
        &filename,
        data,
        mime_type.as_deref(),
    )
    .await?;
    Ok(created(response))
}

/// 获取当前用户的图片列表
#[get("/images")]
pub async fn get_my_images(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    query: web::Query<ImageListQuery>,
) -> Result<HttpResponse, ImageError> {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20).min(100);

    let response = ImageService::get_user_images(
        &state.pool,
        user.id,
        page,
        per_page,
        &state.config.image_base_url,
    )
    .await?;
    Ok(HttpResponse::Ok().json(response))
}

/// 获取单张图片信息
#[get("/images/{image_id}")]
pub async fn get_image_info(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ImageError> {
    let image_id = path.into_inner();

    let response =
        ImageService::get_image_by_id(&state.pool, image_id, &state.config.image_base_url).await?;
    Ok(HttpResponse::Ok().json(response))
}

/// 删除图片
#[delete("/images/{image_id}")]
pub async fn delete_image(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ImageError> {
    let image_id = path.into_inner();

    ImageService::delete_image(&state.pool, &user, &state.storage, image_id).await?;
    Ok(no_content())
}

/// 图片列表查询参数
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
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
