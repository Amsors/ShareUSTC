use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};
use uuid::Uuid;

use crate::config::Config;
use crate::db::AppState;
use crate::models::{resource::ResourceType, resource::UploadResourceRequest, CurrentUser};
use crate::services::{
    AuditLogService, FileService, ImageError, ImageService, ResourceError, ResourceService,
    StorageBackendType, StorageFileMetadata,
};
use crate::utils::{bad_request, created, forbidden, internal_error};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OssStatusResponse {
    storage_backend: String,
    sts_enabled: bool,
    signed_url_expiry: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OssStsTokenRequest {
    file_type: String,
    file_name: String,
    file_size: Option<u64>,
    content_type: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OssStsTokenResponse {
    upload_mode: String,
    upload_key: String,
    expires_in: u64,
    storage_backend: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    upload_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    access_key_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    access_key_secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    security_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expiration: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bucket: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    endpoint: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResourceUploadCallbackRequest {
    oss_key: String,
    title: String,
    course_name: Option<String>,
    resource_type: Option<ResourceType>,
    category: crate::models::resource::ResourceCategory,
    tags: Option<Vec<String>>,
    description: Option<String>,
    teacher_sns: Option<Vec<i64>>,
    course_sns: Option<Vec<i64>>,
    related_resource_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImageUploadCallbackRequest {
    oss_key: String,
    original_name: Option<String>,
}

#[get("/oss/status")]
async fn get_oss_status(state: web::Data<AppState>) -> impl Responder {
    let storage = state.storage.clone();
    HttpResponse::Ok().json(OssStatusResponse {
        storage_backend: storage.backend_type().as_str().to_string(),
        sts_enabled: storage.backend_type() == StorageBackendType::Oss && storage.supports_sts(),
        signed_url_expiry: storage.default_signed_url_expiry(),
    })
}

#[post("/oss/sts-token")]
async fn get_sts_token(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    payload: web::Json<OssStsTokenRequest>,
) -> impl Responder {
    if state.storage.backend_type() != StorageBackendType::Oss {
        return bad_request("当前不是 OSS 存储模式，无法申请直传凭证");
    }

    let (folder, max_size) = match payload.file_type.as_str() {
        "resource" => ("resources", FileService::MAX_FILE_SIZE as u64),
        "image" => ("images", 5 * 1024 * 1024),
        _ => return bad_request("fileType 仅支持 resource 或 image"),
    };

    if payload.file_name.trim().is_empty() {
        return bad_request("fileName 不能为空");
    }
    if let Some(size) = payload.file_size {
        if size == 0 {
            return bad_request("文件不能为空");
        }
        if size > max_size {
            return bad_request("文件大小超过限制");
        }
    }

    let extension = pick_extension(folder, &payload.file_name, payload.content_type.as_deref())
        .unwrap_or_else(|| {
            if folder == "images" {
                "png".to_string()
            } else {
                "bin".to_string()
            }
        });
    let upload_key = format!("{}/{}.{}", folder, Uuid::new_v4(), extension);
    if state.storage.supports_sts() {
        match state.storage.get_sts_token(&upload_key, 0).await {
            Ok(credentials) => {
                log::info!(
                    "[OSS] 生成 STS 上传凭证成功 | user_id={}, file_type={}, key={}",
                    user.id,
                    payload.file_type,
                    credentials.upload_key
                );
                return HttpResponse::Ok().json(OssStsTokenResponse {
                    upload_mode: "sts".to_string(),
                    upload_key: credentials.upload_key,
                    expires_in: credentials.expires_in,
                    storage_backend: "oss".to_string(),
                    upload_url: None,
                    access_key_id: Some(credentials.access_key_id),
                    access_key_secret: Some(credentials.access_key_secret),
                    security_token: Some(credentials.security_token),
                    expiration: Some(credentials.expiration),
                    bucket: Some(credentials.bucket),
                    region: Some(credentials.region),
                    endpoint: Some(credentials.endpoint),
                });
            }
            Err(e) => {
                log::warn!(
                    "[OSS] 生成 STS 上传凭证失败，回退 signed_url | user_id={}, key={}, error={}",
                    user.id,
                    upload_key,
                    e
                );
            }
        }
    } else {
        log::warn!(
            "[OSS] STS 未启用，回退 signed_url 上传 | user_id={}, key={}",
            user.id,
            upload_key
        );
    }

    let expires_in = state.storage.default_signed_url_expiry();
    let upload_url = match state
        .storage
        .get_upload_url(&upload_key, expires_in, payload.content_type.as_deref())
        .await
    {
        Ok(url) => url,
        Err(e) => {
            log::error!(
                "[OSS] 生成直传 URL 失败 | user_id={}, key={}, error={}",
                user.id,
                upload_key,
                e
            );
            return internal_error("生成上传凭证失败");
        }
    };

    log::info!(
        "[OSS] 生成 signed_url 上传凭证成功 | user_id={}, file_type={}, key={}",
        user.id,
        payload.file_type,
        upload_key
    );

    HttpResponse::Ok().json(OssStsTokenResponse {
        upload_mode: "signed_url".to_string(),
        upload_key,
        expires_in,
        storage_backend: "oss".to_string(),
        upload_url: Some(upload_url),
        access_key_id: None,
        access_key_secret: None,
        security_token: None,
        expiration: None,
        bucket: None,
        region: None,
        endpoint: None,
    })
}

#[post("/oss/callback/resource")]
async fn resource_upload_callback(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    req: HttpRequest,
    payload: web::Json<ResourceUploadCallbackRequest>,
) -> Result<HttpResponse, ResourceError> {
    if state.storage.backend_type() != StorageBackendType::Oss {
        return Ok(internal_error("当前未启用OSS存储模式"));
    }

    if !key_in_scope(&payload.oss_key, "resources") {
        return Ok(forbidden("ossKey 不在允许路径范围（resources）"));
    }

    let metadata = match head_file_with_retry(&state, &payload.oss_key, user.id, "资源").await {
        Ok(meta) => meta,
        Err(msg) => return Ok(bad_request(&format!("上传文件不存在或不可访问: {}", msg))),
    };

    let upload_request = UploadResourceRequest {
        title: payload.title.clone(),
        course_name: payload.course_name.clone(),
        resource_type: payload.resource_type.clone().unwrap_or(ResourceType::Other),
        category: payload.category.clone(),
        tags: payload.tags.clone(),
        description: payload.description.clone(),
        teacher_sns: payload.teacher_sns.clone(),
        course_sns: payload.course_sns.clone(),
        related_resource_ids: payload.related_resource_ids.clone(),
    };

    let response = ResourceService::create_resource_from_oss_callback(
        &state.pool,
        &user,
        &state.storage,
        upload_request,
        &payload.oss_key,
        metadata,
    )
    .await?;

    let ip_address = req.peer_addr().map(|addr| addr.ip().to_string());
    let _ = AuditLogService::log_upload_resource(
        &state.pool,
        user.id,
        response.id,
        &response.title,
        &response.resource_type,
        ip_address.as_deref(),
    )
    .await;
    Ok(created(response))
}

#[post("/oss/callback/image")]
async fn image_upload_callback(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    payload: web::Json<ImageUploadCallbackRequest>,
) -> Result<HttpResponse, ImageError> {
    if state.storage.backend_type() != StorageBackendType::Oss {
        return Ok(internal_error("当前未启用OSS存储模式"));
    }

    if !key_in_scope(&payload.oss_key, "images") {
        return Ok(forbidden("ossKey 不在允许路径范围（images）"));
    }

    let metadata: StorageFileMetadata =
        match head_file_with_retry(&state, &payload.oss_key, user.id, "图片").await {
            Ok(meta) => meta,
            Err(msg) => return Ok(bad_request(&format!("上传图片不存在或不可访问: {}", msg))),
        };

    // 加载配置用于生成图片 URL
    let config = Config::from_env();
    let response = ImageService::create_image_from_oss_callback(
        &state.pool,
        &user,
        &state.storage,
        &config,
        &payload.oss_key,
        payload.original_name.as_deref(),
        metadata,
    )
    .await?;
    Ok(created(response))
}

fn key_in_scope(key: &str, scope: &str) -> bool {
    // 使用 Path 规范化进行更稳健的路径验证
    let normalized = key.trim_start_matches('/');

    // 检查是否包含路径遍历字符序列
    if normalized.contains("..") || normalized.contains("//") {
        return false;
    }

    // 使用 std::path::Path 进行规范化处理
    let path = std::path::Path::new(normalized);

    // 获取路径的第一个组件
    if let Some(first_component) = path.components().next() {
        if let std::path::Component::Normal(first) = first_component {
            if let Some(first_str) = first.to_str() {
                return first_str == scope;
            }
        }
    }

    false
}

fn pick_extension(folder: &str, file_name: &str, content_type: Option<&str>) -> Option<String> {
    let ext = std::path::Path::new(file_name)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.trim().to_lowercase())
        .filter(|ext| !ext.is_empty());
    if ext.is_some() {
        return ext;
    }

    match (folder, content_type) {
        ("images", Some("image/jpeg")) | ("images", Some("image/jpg")) => Some("jpg".to_string()),
        ("images", Some("image/png")) => Some("png".to_string()),
        ("resources", Some("application/pdf")) => Some("pdf".to_string()),
        ("resources", Some("text/markdown")) => Some("md".to_string()),
        ("resources", Some("text/plain")) => Some("txt".to_string()),
        ("resources", Some("application/zip")) => Some("zip".to_string()),
        _ => None,
    }
}

async fn head_file_with_retry(
    state: &web::Data<AppState>,
    oss_key: &str,
    user_id: Uuid,
    scope: &str,
) -> Result<StorageFileMetadata, String> {
    let retry_delays_ms = [0_u64, 200, 500];
    let mut last_error = String::new();

    for (idx, delay_ms) in retry_delays_ms.iter().enumerate() {
        if *delay_ms > 0 {
            sleep(Duration::from_millis(*delay_ms)).await;
        }

        match state.storage.head_file(oss_key).await {
            Ok(meta) => return Ok(meta),
            Err(e) => {
                last_error = e.to_string();
                log::warn!(
                    "[OSS] {}回调读取文件元信息失败 | user_id={}, key={}, attempt={}/{}, error={}",
                    scope,
                    user_id,
                    oss_key,
                    idx + 1,
                    retry_delays_ms.len(),
                    e
                );
            }
        }
    }

    log::warn!(
        "[OSS] {}回调读取文件元信息最终失败 | user_id={}, key={}, error={}",
        scope,
        user_id,
        oss_key,
        last_error
    );
    Err(last_error)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_oss_status)
        .service(get_sts_token)
        .service(resource_upload_callback)
        .service(image_upload_callback);
}
