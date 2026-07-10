use actix_web::{get, post, put, web, HttpRequest, HttpResponse};
use uuid::Uuid;

use crate::db::AppState;
use crate::models::CurrentUser;
use crate::services::{ResourceService, StorageBackendType, StorageError};
use crate::utils::{build_content_disposition, internal_error, not_found};

use super::{record_download_events, sanitize_filename};

/// 下载资源
/// 支持未登录用户（游客）下载
#[get("/resources/{resource_id}/download")]
pub async fn download_resource(
    state: web::Data<AppState>,
    user: Option<web::ReqData<CurrentUser>>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let resource_id = path.into_inner();
    let current_user = user.map(|u| u.into_inner());

    // 获取资源文件路径和存储类型（带权限检查）；不存在/无权限经 ResourceError 冒泡
    let (file_path, resource_type, title, storage_type) =
        ResourceService::get_resource_file_path(&state.pool, resource_id, current_user.as_ref())
            .await?;

    let user_id = current_user.map(|u| u.id);
    let content_type = crate::services::FileService::get_mime_type_by_type(&resource_type);
    let extension = crate::services::FileService::get_extension_by_type(&resource_type);
    let filename = format!("{}.{}", sanitize_filename(&title), extension);
    let content_disposition = build_content_disposition(&filename);

    // 根据资源实际的存储类型决定读取方式
    let is_oss = storage_type.as_deref() == Some("oss");

    if is_oss {
        // OSS 存储：生成签名下载 URL
        let expires_secs = state.storage.default_signed_url_expiry();
        match state
            .storage
            .get_download_url(&file_path, &filename, expires_secs)
            .await
        {
            Ok(download_url) => {
                record_download_events(&state, resource_id, user_id, &title, &req).await;
                Ok(HttpResponse::Found()
                    .insert_header(("Location", download_url))
                    .finish())
            }
            Err(e) => {
                log::warn!(
                    "[Resource] 生成 OSS 下载链接失败 | resource_id={}, path={}, error={}",
                    resource_id,
                    file_path,
                    e
                );
                Ok(internal_error("生成下载链接失败"))
            }
        }
    } else {
        // 本地存储：需要创建本地存储实例来读取文件
        let config = crate::config::Config::from_env();
        match crate::services::create_local_storage(&config) {
            Ok(local_storage) => match local_storage.read_file(&file_path).await {
                Ok(file_content) => {
                    record_download_events(&state, resource_id, user_id, &title, &req).await;

                    log::info!(
                        "[Resource] 资源下载成功 | resource_id={}, user_id={:?}, storage=local",
                        resource_id,
                        user_id
                    );

                    Ok(HttpResponse::Ok()
                        .content_type(content_type)
                        .insert_header(("Content-Disposition", content_disposition))
                        .body(file_content))
                }
                Err(StorageError::NotFound(_)) => {
                    log::warn!(
                        "[Resource] 下载文件不存在 | resource_id={}, path={}",
                        resource_id,
                        file_path
                    );
                    Ok(not_found("文件不存在"))
                }
                Err(e) => {
                    log::warn!(
                        "[Resource] 读取资源文件失败(下载) | resource_id={}, path={}, error={}",
                        resource_id,
                        file_path,
                        e
                    );
                    Ok(internal_error("文件读取失败"))
                }
            },
            Err(e) => {
                log::error!("[Resource] 创建本地存储失败 | error={}", e);
                Ok(internal_error("无法访问本地存储"))
            }
        }
    }
}

/// 获取资源预览 URL
/// 对于 OSS 存储的资源，返回带签名的直链 URL（避免服务器中转流量）
/// 对于本地存储的资源，返回 /content 代理路径
/// 支持未登录用户（游客）预览
#[get("/resources/{resource_id}/preview-url")]
pub async fn get_resource_preview_url(
    state: web::Data<AppState>,
    user: Option<web::ReqData<CurrentUser>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let resource_id = path.into_inner();
    let current_user = user.map(|u| u.into_inner());

    // 获取资源文件路径和存储类型（带权限检查）
    let (file_path, resource_type, storage_type, updated_at) =
        ResourceService::get_resource_file_path_for_preview(
            &state.pool,
            resource_id,
            current_user.as_ref(),
        )
        .await?;

    let is_oss = storage_type.as_deref() == Some("oss");

    // 将 updated_at 格式化为 ISO 8601 字符串
    let updated_at_str = updated_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

    if is_oss {
        // OSS 存储：生成签名 URL，前端直接从 OSS 获取
        let expires_secs = state.storage.default_signed_url_expiry();
        match state.storage.get_file_url(&file_path, expires_secs).await {
            Ok(preview_url) => {
                log::debug!(
                    "[Resource] 生成 OSS 预览 URL | resource_id={}, storage=oss",
                    resource_id
                );
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "previewUrl": preview_url,
                    "storageType": "oss",
                    "resourceType": resource_type,
                    "directAccess": true,
                    "updatedAt": updated_at_str
                })))
            }
            Err(e) => {
                log::warn!(
                    "[Resource] 生成 OSS 预览 URL 失败 | resource_id={}, error={}",
                    resource_id,
                    e
                );
                Ok(internal_error("生成预览链接失败"))
            }
        }
    } else {
        // 本地存储：返回相对路径，通过 /content 接口获取
        log::debug!(
            "[Resource] 本地存储预览 | resource_id={}, storage=local",
            resource_id
        );
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "previewUrl": format!("/api/resources/{}/content", resource_id),
            "storageType": "local",
            "resourceType": resource_type,
            "directAccess": false,
            "updatedAt": updated_at_str
        })))
    }
}

/// 获取资源文件内容（用于预览）
/// 使用后端代理模式读取文件，本地存储和OSS兜底场景使用
/// 支持未登录用户（游客）预览
#[get("/resources/{resource_id}/content")]
pub async fn get_resource_content(
    state: web::Data<AppState>,
    user: Option<web::ReqData<CurrentUser>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let resource_id = path.into_inner();
    let current_user = user.map(|u| u.into_inner());

    // 获取资源文件路径和存储类型（带权限检查）
    let (file_path, resource_type, storage_type, updated_at) =
        ResourceService::get_resource_file_path_for_preview(
            &state.pool,
            resource_id,
            current_user.as_ref(),
        )
        .await?;

    // 根据资源实际的存储类型选择正确的存储后端读取文件
    // 使用后端代理模式，避免浏览器直接访问 OSS 产生 CORS 问题
    let is_oss = storage_type.as_deref() == Some("oss");

    // 将 updated_at 格式化为 ISO 8601 字符串
    let updated_at_str = updated_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

    let read_result = if is_oss {
        // OSS 存储：使用主 storage（如果是 OSS 模式）或创建 OSS 存储实例
        if state.storage.backend_type() == StorageBackendType::Oss {
            state.storage.read_file(&file_path).await
        } else {
            // 当前是 local 模式，但需要读取 OSS 文件
            // 创建临时 OSS 存储实例
            let config = crate::config::Config::from_env();
            match crate::services::create_storage_backend(&config) {
                Ok(oss_storage) if oss_storage.backend_type() == StorageBackendType::Oss => {
                    oss_storage.read_file(&file_path).await
                }
                _ => {
                    log::warn!(
                        "[Resource] 无法创建 OSS 存储实例来读取资源 | resource_id={}",
                        resource_id
                    );
                    return Ok(internal_error("无法读取 OSS 资源"));
                }
            }
        }
    } else {
        // 本地存储：使用主 storage（如果是 Local 模式）或创建本地存储实例
        if state.storage.backend_type() == StorageBackendType::Local {
            state.storage.read_file(&file_path).await
        } else {
            // 当前是 OSS 模式，但需要读取本地文件
            let config = crate::config::Config::from_env();
            match crate::services::create_local_storage(&config) {
                Ok(local_storage) => local_storage.read_file(&file_path).await,
                Err(e) => {
                    log::error!("[Resource] 创建本地存储失败 | error={}", e);
                    return Ok(internal_error("无法访问本地存储"));
                }
            }
        }
    };

    match read_result {
        Ok(file_content) => {
            // 获取 MIME 类型 - 优先使用 resource_type，因为它更准确
            let content_type = crate::services::FileService::get_mime_type_by_type(&resource_type);

            log::debug!(
                "[Resource] 预览资源 | resource_id={}, path={}, type={}, mime={}, storage={}",
                resource_id,
                file_path,
                resource_type,
                content_type,
                if is_oss { "oss" } else { "local" }
            );

            // 返回文件内容（inline 显示，不是下载）
            Ok(HttpResponse::Ok()
                .content_type(content_type)
                .insert_header(("Cache-Control", "public, max-age=3600"))
                .insert_header(("X-Resource-Updated-At", updated_at_str.as_str()))
                .body(file_content))
        }
        Err(StorageError::NotFound(_)) => {
            log::warn!(
                "[Resource] 预览文件不存在 | resource_id={}, path={}",
                resource_id,
                file_path
            );
            Ok(not_found("文件不存在"))
        }
        Err(e) => {
            log::warn!(
                "[Resource] 读取资源文件失败(预览) | resource_id={}, path={}, error={}",
                resource_id,
                file_path,
                e
            );
            Ok(internal_error("文件读取失败"))
        }
    }
}

/// 获取资源原始内容（用于Markdown编辑）
#[get("/resources/{resource_id}/raw")]
pub async fn get_resource_raw_content(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, crate::services::ResourceError> {
    let resource_id = path.into_inner();

    let content =
        ResourceService::get_resource_content_raw(&state.pool, &state.storage, &user, resource_id)
            .await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "content": content
    })))
}

/// 更新资源内容（用于Markdown在线编辑）
#[put("/resources/{resource_id}/content")]
pub async fn update_resource_content(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    request: web::Json<crate::models::UpdateResourceContentRequest>,
    req: HttpRequest,
) -> Result<HttpResponse, crate::services::ResourceError> {
    use crate::services::AuditLogService;
    use crate::utils::bad_request;

    let resource_id = path.into_inner();

    // 验证请求
    if let Err(msg) = request.validate() {
        return Ok(bad_request(&msg));
    }

    // 获取资源信息（用于审计日志）
    let resource_detail = ResourceService::get_resource_detail(&state.pool, resource_id).await?;

    let response = ResourceService::update_resource_content(
        &state.pool,
        &user,
        &state.storage,
        resource_id,
        request.content.clone(),
    )
    .await?;

    // 记录审计日志
    let ip_address = req.peer_addr().map(|addr| addr.ip().to_string());
    if let Err(e) = AuditLogService::log_update_resource(
        &state.pool,
        user.id,
        resource_id,
        &resource_detail.title,
        ip_address.as_deref(),
    )
    .await
    {
        log::warn!(
            "[Audit] 记录资源更新日志失败 | resource_id={}, error={}",
            resource_id,
            e
        );
    }

    Ok(HttpResponse::Ok().json(response))
}

/// 记录下载（用于缓存下载和浏览器端打包下载场景）
/// 支持未登录用户（游客）
#[post("/resources/{resource_id}/track-download")]
pub async fn track_download(
    state: web::Data<AppState>,
    user: Option<web::ReqData<CurrentUser>>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse, crate::services::ResourceError> {
    let resource_id = path.into_inner();
    let current_user = user.map(|u| u.into_inner());
    let user_id = current_user.map(|u| u.id);

    // 获取资源信息（用于审计日志）
    let resource_detail = ResourceService::get_resource_detail(&state.pool, resource_id).await?;

    // 记录下载事件
    record_download_events(&state, resource_id, user_id, &resource_detail.title, &req).await;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "下载记录已保存",
        "resourceId": resource_id
    })))
}
