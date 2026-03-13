use actix_multipart::Multipart;
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};
use futures_util::StreamExt;
use uuid::Uuid;

use crate::db::AppState;
use crate::models::{
    resource::*, CommentListQuery, CreateCommentRequest, CreateRatingRequest, CurrentUser,
    UpdateResourceContentRequest, UpdateResourceDescriptionRequest, UpdateResourceRelationsRequest,
};
use crate::services::{
    AuditLogService, CommentService, LikeService, RatingService, ResourceError, ResourceService,
    StorageBackendType, StorageError,
};
use crate::utils::{bad_request, conflict, forbidden, internal_error, not_found};

/// 上传资源
#[post("/resources")]
pub async fn upload_resource(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    mut payload: Multipart,
    req: HttpRequest,
) -> impl Responder {
    let mut metadata: Option<UploadResourceRequest> = None;
    let mut file_data: Option<(String, Vec<u8>, Option<String>)> = None;

    // 解析 multipart 表单数据
    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(field) => field,
            Err(e) => {
                log::warn!(
                    "[Resource] 解析上传数据失败 | user_id={}, error={}",
                    user.id,
                    e
                );
                return bad_request("解析上传数据失败");
            }
        };

        let content_disposition = field.content_disposition();
        let field_name = content_disposition.get_name().unwrap_or("unknown");

        match field_name {
            "metadata" => {
                // 读取元数据 JSON
                let mut data = Vec::new();
                while let Some(chunk) = field.next().await {
                    match chunk {
                        Ok(bytes) => data.extend_from_slice(&bytes),
                        Err(e) => {
                            log::warn!(
                                "[Resource] 读取元数据失败 | user_id={}, error={}",
                                user.id,
                                e
                            );
                            return bad_request("读取元数据失败");
                        }
                    }
                }

                // 解析 JSON
                match serde_json::from_slice::<UploadResourceRequest>(&data) {
                    Ok(req) => metadata = Some(req),
                    Err(e) => {
                        log::warn!(
                            "[Resource] 解析元数据 JSON 失败 | user_id={}, error={}",
                            user.id,
                            e
                        );
                        return bad_request(&format!("元数据格式错误: {}", e));
                    }
                }
            }
            "file" => {
                // 获取文件名
                let filename = content_disposition
                    .get_filename()
                    .unwrap_or("unnamed.bin")
                    .to_string();

                // 获取 MIME 类型
                let mime_type = field.content_type().map(|m| m.to_string());

                // 读取文件数据
                let mut data = Vec::new();
                while let Some(chunk) = field.next().await {
                    match chunk {
                        Ok(bytes) => data.extend_from_slice(&bytes),
                        Err(e) => {
                            log::warn!(
                                "[Resource] 读取文件数据失败 | user_id={}, error={}",
                                user.id,
                                e
                            );
                            return bad_request("读取文件数据失败");
                        }
                    }
                }

                file_data = Some((filename, data, mime_type));
            }
            _ => {
                // 忽略未知字段
                while let Some(_) = field.next().await {}
            }
        }
    }

    // 检查是否有元数据
    let metadata = match metadata {
        Some(m) => m,
        None => {
            return bad_request("缺少资源元数据");
        }
    };

    // 检查是否有文件数据
    let (filename, data, mime_type) = match file_data {
        Some(d) => d,
        None => {
            return bad_request("请选择要上传的文件");
        }
    };

    // 调用服务上传资源
    match ResourceService::upload_resource(
        &state.pool,
        &user,
        &state.storage,
        metadata,
        &filename,
        data,
        mime_type.as_deref(),
    )
    .await
    {
        Ok(response) => {
            // 记录审计日志
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

            log::info!(
                "[Resource] 资源上传成功 | resource_id={}, user_id={}, title={}",
                response.id,
                user.id,
                response.title
            );

            HttpResponse::Created().json(response)
        }
        Err(e) => {
            log::error!(
                "[Resource] 资源上传失败 | user_id={}, error={:?}",
                user.id,
                e
            );
            match e {
                ResourceError::ValidationError(msg) => bad_request(&msg),
                ResourceError::FileError(msg) => internal_error(&msg),
                ResourceError::DatabaseError(msg) => {
                    log::error!("数据库错误详情: {}", msg);
                    internal_error(&format!("数据库错误: {}", msg))
                }
                ResourceError::AiError(msg) => internal_error(&msg),
                ResourceError::NotFound(msg) => not_found(&msg),
                ResourceError::Unauthorized(msg) => forbidden(&msg),
                ResourceError::Conflict(msg) => conflict(&msg),
            }
        }
    }
}

/// 获取资源列表
#[get("/resources")]
pub async fn get_resource_list(
    state: web::Data<AppState>,
    query: web::Query<ResourceListQuery>,
) -> impl Responder {
    match ResourceService::get_resource_list(&state.pool, &query).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            log::warn!("[Resource] 获取资源列表失败 | error={}", e);
            internal_error("获取资源列表失败")
        }
    }
}

/// 搜索资源
#[get("/resources/search")]
pub async fn search_resources(
    state: web::Data<AppState>,
    query: web::Query<ResourceSearchQuery>,
) -> impl Responder {
    // 验证搜索关键词
    if query.q.trim().is_empty() {
        return bad_request("搜索关键词不能为空");
    }

    match ResourceService::search_resources(&state.pool, &query).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            log::warn!("[Resource] 搜索资源失败 | error={}", e);
            internal_error("搜索资源失败")
        }
    }
}

/// 获取资源详情
#[get("/resources/{resource_id}")]
pub async fn get_resource_detail(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let resource_id = path.into_inner();

    // 增加浏览量
    let _ = ResourceService::increment_views(&state.pool, resource_id).await;

    match ResourceService::get_resource_detail(&state.pool, resource_id).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            log::warn!(
                "[Resource] 获取资源详情失败 | resource_id={}, error={}",
                resource_id,
                e
            );
            match e {
                ResourceError::NotFound(msg) => not_found(&msg),
                _ => internal_error("获取资源详情失败"),
            }
        }
    }
}

/// 删除资源
#[delete("/resources/{resource_id}")]
pub async fn delete_resource(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> impl Responder {
    let resource_id = path.into_inner();

    log::info!(
        "[Resource] 删除资源 | resource_id={}, user_id={}",
        resource_id,
        user.id
    );

    match ResourceService::delete_resource(&state.pool, &user, &state.storage, resource_id).await {
        Ok(title) => {
            // 获取 IP 地址
            let ip_address = req.peer_addr().map(|addr| addr.ip().to_string());

            // 记录审计日志
            let _ = AuditLogService::log_delete_resource(
                &state.pool,
                user.id,
                resource_id,
                &title,
                ip_address.as_deref(),
            )
            .await;

            log::info!(
                "[Resource] 资源删除成功 | resource_id={}, user_id={}",
                resource_id,
                user.id
            );

            HttpResponse::NoContent().finish()
        }
        Err(e) => {
            log::warn!(
                "[Resource] 删除资源失败 | resource_id={}, user_id={}, error={}",
                resource_id,
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

/// 获取当前用户的资源列表
#[get("/resources/my")]
pub async fn get_my_resources(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    query: web::Query<ResourceListQuery>,
) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20).min(100);

    match ResourceService::get_user_resources(&state.pool, user.id, page, per_page).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            log::warn!(
                "[Resource] 获取我的资源列表失败 | user_id={}, error={}",
                user.id,
                e
            );
            internal_error("获取资源列表失败")
        }
    }
}

/// 下载资源
/// 支持未登录用户（游客）下载
#[get("/resources/{resource_id}/download")]
pub async fn download_resource(
    state: web::Data<AppState>,
    user: Option<web::ReqData<CurrentUser>>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> impl Responder {
    let resource_id = path.into_inner();
    let current_user = user.map(|u| u.into_inner());

    // 获取资源文件路径和存储类型（带权限检查）
    match ResourceService::get_resource_file_path(&state.pool, resource_id, current_user.as_ref()).await {
        Ok((file_path, resource_type, title, storage_type)) => {
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
                        record_download_events(&state, resource_id, user_id, &title, &req)
                            .await;
                        HttpResponse::Found()
                            .insert_header(("Location", download_url))
                            .finish()
                    }
                    Err(e) => {
                        log::warn!(
                            "[Resource] 生成 OSS 下载链接失败 | resource_id={}, path={}, error={}",
                            resource_id,
                            file_path,
                            e
                        );
                        internal_error("生成下载链接失败")
                    }
                }
            } else {
                // 本地存储：需要创建本地存储实例来读取文件
                let config = crate::config::Config::from_env();
                match crate::services::create_local_storage(&config) {
                    Ok(local_storage) => {
                        match local_storage.read_file(&file_path).await {
                            Ok(file_content) => {
                                record_download_events(&state, resource_id, user_id, &title, &req).await;

                                log::info!(
                                    "[Resource] 资源下载成功 | resource_id={}, user_id={:?}, storage=local",
                                    resource_id,
                                    user_id
                                );

                                HttpResponse::Ok()
                                    .content_type(content_type)
                                    .insert_header(("Content-Disposition", content_disposition))
                                    .body(file_content)
                            }
                            Err(StorageError::NotFound(_)) => {
                                log::warn!(
                                    "[Resource] 下载文件不存在 | resource_id={}, path={}",
                                    resource_id,
                                    file_path
                                );
                                not_found("文件不存在")
                            }
                            Err(e) => {
                                log::warn!(
                                    "[Resource] 读取资源文件失败(下载) | resource_id={}, path={}, error={}",
                                    resource_id,
                                    file_path,
                                    e
                                );
                                internal_error("文件读取失败")
                            }
                        }
                    }
                    Err(e) => {
                        log::error!(
                            "[Resource] 创建本地存储失败 | error={}",
                            e
                        );
                        internal_error("无法访问本地存储")
                    }
                }
            }
        }
        Err(e) => {
            log::warn!(
                "[Resource] 获取资源文件路径失败(下载) | resource_id={}, error={}",
                resource_id,
                e
            );
            match e {
                ResourceError::NotFound(msg) => not_found(&msg),
                ResourceError::Unauthorized(msg) => forbidden(&msg),
                _ => internal_error("获取资源失败"),
            }
        }
    }
}

async fn record_download_events(
    state: &web::Data<AppState>,
    resource_id: Uuid,
    user_id: Option<Uuid>,
    title: &str,
    req: &HttpRequest,
) {
    let _ = ResourceService::increment_downloads(&state.pool, resource_id).await;

    let ip_address = req
        .peer_addr()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "0.0.0.0".to_string());

    let _ = ResourceService::record_download(&state.pool, resource_id, user_id, &ip_address).await;

    let _ = AuditLogService::log_download_resource(
        &state.pool,
        user_id,
        resource_id,
        title,
        Some(&ip_address),
    )
    .await;
}

/// 清理文件名，移除不合法字符
fn sanitize_filename(filename: &str) -> String {
    // 移除或替换文件系统不支持的字符
    filename
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            c => c,
        })
        .collect()
}

/// 对文件名进行 RFC 5987 编码，用于支持中文等非 ASCII 字符
/// 参考: https://datatracker.ietf.org/doc/html/rfc5987
fn encode_rfc5987(filename: &str) -> String {
    let mut result = String::new();
    for c in filename.chars() {
        if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
            // ASCII 字母数字和常用符号直接保留
            result.push(c);
        } else {
            // 非 ASCII 字符进行 percent-encoding
            for byte in c.encode_utf8(&mut [0; 4]).bytes() {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

/// 检查文件名是否只包含 ASCII 字符
fn is_ascii_filename(filename: &str) -> bool {
    filename.chars().all(|c| c.is_ascii())
}

/// 构建 Content-Disposition 头部值
///
/// 策略：
/// 1. 对于纯 ASCII 文件名：直接使用 filename="xxx"
/// 2. 对于含中文的文件名：同时提供 filename 和 filename*
///    - filename：包含原始中文，HTTP 库会自动处理编码
///    - filename*：RFC 5987 编码，现代浏览器优先使用
fn build_content_disposition(filename: &str) -> String {
    if is_ascii_filename(filename) {
        // 纯 ASCII 文件名，直接使用
        format!("attachment; filename=\"{}\"", filename)
    } else {
        // 包含中文等非 ASCII 字符
        // RFC 5987 编码用于 filename*
        let encoded = encode_rfc5987(filename);

        // 同时提供 filename 和 filename*
        // filename* 优先被现代浏览器使用，能正确显示中文
        format!(
            "attachment; filename*=UTF-8''{}; filename=\"{}\"",
            encoded, filename
        )
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
) -> impl Responder {
    let resource_id = path.into_inner();
    let current_user = user.map(|u| u.into_inner());

    // 获取资源文件路径和存储类型（带权限检查）
    match ResourceService::get_resource_file_path_for_preview(&state.pool, resource_id, current_user.as_ref()).await {
        Ok((file_path, resource_type, storage_type, updated_at)) => {
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
                        HttpResponse::Ok().json(serde_json::json!({
                            "previewUrl": preview_url,
                            "storageType": "oss",
                            "resourceType": resource_type,
                            "directAccess": true,
                            "updatedAt": updated_at_str
                        }))
                    }
                    Err(e) => {
                        log::warn!(
                            "[Resource] 生成 OSS 预览 URL 失败 | resource_id={}, error={}",
                            resource_id,
                            e
                        );
                        internal_error("生成预览链接失败")
                    }
                }
            } else {
                // 本地存储：返回相对路径，通过 /content 接口获取
                log::debug!(
                    "[Resource] 本地存储预览 | resource_id={}, storage=local",
                    resource_id
                );
                HttpResponse::Ok().json(serde_json::json!({
                    "previewUrl": format!("/api/resources/{}/content", resource_id),
                    "storageType": "local",
                    "resourceType": resource_type,
                    "directAccess": false,
                    "updatedAt": updated_at_str
                }))
            }
        }
        Err(e) => {
            log::warn!(
                "[Resource] 获取预览 URL 失败 | resource_id={}, error={}",
                resource_id,
                e
            );
            match e {
                ResourceError::NotFound(msg) => not_found(&msg),
                ResourceError::Unauthorized(msg) => forbidden(&msg),
                _ => internal_error("获取预览链接失败"),
            }
        }
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
) -> impl Responder {
    let resource_id = path.into_inner();
    let current_user = user.map(|u| u.into_inner());

    // 获取资源文件路径和存储类型（带权限检查）
    match ResourceService::get_resource_file_path_for_preview(&state.pool, resource_id, current_user.as_ref()).await {
        Ok((file_path, resource_type, storage_type, updated_at)) => {
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
                            return internal_error("无法读取 OSS 资源");
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
                            return internal_error("无法访问本地存储");
                        }
                    }
                }
            };

            match read_result {
                Ok(file_content) => {
                    // 获取 MIME 类型 - 优先使用 resource_type，因为它更准确
                    let content_type =
                        crate::services::FileService::get_mime_type_by_type(&resource_type);

                    log::debug!(
                        "[Resource] 预览资源 | resource_id={}, path={}, type={}, mime={}, storage={}",
                        resource_id,
                        file_path,
                        resource_type,
                        content_type,
                        if is_oss { "oss" } else { "local" }
                    );

                    // 返回文件内容（inline 显示，不是下载）
                    HttpResponse::Ok()
                        .content_type(content_type)
                        .insert_header(("Cache-Control", "public, max-age=3600"))
                        .insert_header(("X-Resource-Updated-At", updated_at_str.as_str()))
                        .body(file_content)
                }
                Err(StorageError::NotFound(_)) => {
                    log::warn!(
                        "[Resource] 预览文件不存在 | resource_id={}, path={}",
                        resource_id,
                        file_path
                    );
                    not_found("文件不存在")
                }
                Err(e) => {
                    log::warn!(
                        "[Resource] 读取资源文件失败(预览) | resource_id={}, path={}, error={}",
                        resource_id,
                        file_path,
                        e
                    );
                    internal_error("文件读取失败")
                }
            }
        }
        Err(e) => {
            log::warn!(
                "[Resource] 获取资源文件路径失败(预览) | resource_id={}, error={}",
                resource_id,
                e
            );
            match e {
                ResourceError::NotFound(msg) => not_found(&msg),
                ResourceError::Unauthorized(msg) => forbidden(&msg),
                _ => internal_error("获取资源失败"),
            }
        }
    }
}

/// 获取资源原始内容（用于Markdown编辑）
#[get("/resources/{resource_id}/raw")]
pub async fn get_resource_raw_content(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let resource_id = path.into_inner();

    match ResourceService::get_resource_content_raw(&state.pool, &state.storage, &user, resource_id)
        .await
    {
        Ok(content) => HttpResponse::Ok().json(serde_json::json!({
            "content": content
        })),
        Err(e) => {
            log::warn!(
                "[Resource] 获取资源原始内容失败 | resource_id={}, user_id={}, error={}",
                resource_id,
                user.id,
                e
            );
            match e {
                ResourceError::NotFound(msg) => not_found(&msg),
                ResourceError::Unauthorized(msg) => forbidden(&msg),
                _ => internal_error("获取资源原始内容失败"),
            }
        }
    }
}

/// 更新资源内容（用于Markdown在线编辑）
#[put("/resources/{resource_id}/content")]
pub async fn update_resource_content(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    request: web::Json<UpdateResourceContentRequest>,
    req: HttpRequest,
) -> impl Responder {
    let resource_id = path.into_inner();

    // 验证请求
    if let Err(msg) = request.validate() {
        return bad_request(&msg);
    }

    // 获取资源信息（用于审计日志）
    let resource_detail = match ResourceService::get_resource_detail(&state.pool, resource_id).await {
        Ok(detail) => detail,
        Err(e) => {
            log::warn!(
                "[Resource] 获取资源详情失败 | resource_id={}, user_id={}, error={}",
                resource_id,
                user.id,
                e
            );
            return match e {
                ResourceError::NotFound(msg) => not_found(&msg),
                _ => internal_error("获取资源详情失败"),
            };
        }
    };

    match ResourceService::update_resource_content(
        &state.pool,
        &user,
        &state.storage,
        resource_id,
        request.content.clone(),
    )
    .await
    {
        Ok(response) => {
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
                log::warn!("[Audit] 记录资源更新日志失败 | resource_id={}, error={}", resource_id, e);
            }

            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            log::warn!(
                "[Resource] 更新资源内容失败 | resource_id={}, user_id={}, error={}",
                resource_id,
                user.id,
                e
            );
            match e {
                ResourceError::NotFound(msg) => not_found(&msg),
                ResourceError::Unauthorized(msg) => forbidden(&msg),
                ResourceError::ValidationError(msg) => bad_request(&msg),
                _ => internal_error("更新资源内容失败"),
            }
        }
    }
}

/// 获取热门资源列表
#[get("/resources/hot")]
pub async fn get_hot_resources(
    state: web::Data<AppState>,
    query: web::Query<crate::models::HotResourcesQuery>,
) -> impl Responder {
    let limit = query.limit.unwrap_or(10);

    match ResourceService::get_hot_resources(&state.pool, limit).await {
        Ok(resources) => HttpResponse::Ok().json(resources),
        Err(e) => {
            log::warn!("获取热门资源失败: {}", e);
            internal_error("获取热门资源失败")
        }
    }
}

/// 资源数量响应
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceCountResponse {
    pub total: i64,
}

/// 获取资源总数
#[get("/resources/count")]
pub async fn get_resource_count(state: web::Data<AppState>) -> impl Responder {
    match ResourceService::get_resource_count(&state.pool).await {
        Ok(count) => HttpResponse::Ok().json(ResourceCountResponse { total: count }),
        Err(e) => {
            log::warn!("[Resource] 获取资源总数失败: {}", e);
            internal_error("获取资源总数失败")
        }
    }
}

/// 搜索可关联的资源
/// 用于在上传资源时搜索要关联的其他资源
#[get("/resources/search-for-relation")]
pub async fn search_resources_for_relation(
    state: web::Data<AppState>,
    query: web::Query<ResourceSearchForRelationQuery>,
) -> impl Responder {
    // 验证搜索关键词
    if query.q.trim().is_empty() {
        return bad_request("搜索关键词不能为空");
    }

    let exclude_id = query.exclude_id.clone().and_then(|id| Uuid::parse_str(&id).ok());

    match ResourceService::search_resources_for_relation(
        &state.pool,
        &query.q,
        exclude_id,
        query.limit.unwrap_or(10),
    )
    .await
    {
        Ok(resources) => HttpResponse::Ok().json(resources),
        Err(e) => {
            log::warn!("[Resource] 搜索可关联资源失败 | error={}", e);
            internal_error("搜索资源失败")
        }
    }
}

/// 获取资源的关联资源列表
#[get("/resources/{resource_id}/relations")]
pub async fn get_resource_relations(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let resource_id = path.into_inner();

    match ResourceService::get_related_resources(&state.pool, resource_id).await {
        Ok(resources) => HttpResponse::Ok().json(resources),
        Err(e) => {
            log::warn!(
                "[Resource] 获取关联资源列表失败 | resource_id={}, error={}",
                resource_id,
                e
            );
            match e {
                ResourceError::NotFound(msg) => not_found(&msg),
                _ => internal_error("获取关联资源失败"),
            }
        }
    }
}

/// 搜索可关联资源的查询参数
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceSearchForRelationQuery {
    pub q: String,
    pub exclude_id: Option<String>,
    pub limit: Option<i32>,
}

/// 记录下载（用于缓存下载和浏览器端打包下载场景）
/// 支持未登录用户（游客）
#[post("/resources/{resource_id}/track-download")]
pub async fn track_download(
    state: web::Data<AppState>,
    user: Option<web::ReqData<CurrentUser>>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> impl Responder {
    let resource_id = path.into_inner();
    let current_user = user.map(|u| u.into_inner());
    let user_id = current_user.map(|u| u.id);

    // 获取资源信息（用于审计日志）
    let resource_detail = match ResourceService::get_resource_detail(&state.pool, resource_id).await {
        Ok(detail) => detail,
        Err(e) => {
            log::warn!(
                "[Resource] 获取资源详情失败(track-download) | resource_id={:?}, error={}",
                resource_id,
                e
            );
            return match e {
                ResourceError::NotFound(msg) => not_found(&msg),
                _ => internal_error("获取资源详情失败"),
            };
        }
    };

    // 记录下载事件
    record_download_events(&state, resource_id, user_id, &resource_detail.title, &req).await;

    HttpResponse::Ok().json(serde_json::json!({
        "message": "下载记录已保存",
        "resourceId": resource_id
    }))
}

/// 根据文件哈希查询资源
/// 用于上传前检查是否已存在相同内容的资源
#[get("/resources/by-hash/{file_hash}")]
pub async fn get_resources_by_hash(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let file_hash = path.into_inner();

    // 验证哈希格式（应该是64位十六进制字符串，即SHA256）
    if file_hash.len() != 64 || !file_hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return bad_request("无效的哈希格式，应为64位十六进制字符串");
    }

    match ResourceService::find_by_file_hash(&state.pool, &file_hash).await {
        Ok(resources) => HttpResponse::Ok().json(resources),
        Err(e) => {
            log::warn!(
                "[Resource] 根据哈希查询资源失败 | hash={}, error={}",
                &file_hash[..16.min(file_hash.len())],
                e
            );
            internal_error("查询资源失败")
        }
    }
}

/// 配置公开资源路由（不需要认证）
pub fn config_public(cfg: &mut web::ServiceConfig) {
    // 注意：具体路径必须放在通配路径之前注册
    // 否则 /resources/hot 会被 /resources/{id} 匹配
    cfg.service(get_pdf_preview_challenge_config) // /resources/pdf-preview-challenge/config
        .service(verify_pdf_preview_challenge) // /resources/pdf-preview-challenge/verify
        .service(get_hot_resources) // /resources/hot （先注册具体路径）
        .service(get_resource_count) // /resources/count
        .service(search_resources_for_relation) // /resources/search-for-relation
        .service(get_resources_by_hash) // /resources/by-hash/{file_hash}
        .service(get_resource_list) // /resources
        .service(search_resources) // /resources/search
        .service(get_resource_detail) // /resources/{id} （后注册通配路径）
        .service(download_resource)
        .service(track_download) // 记录下载（用于缓存/浏览器打包场景）
        .service(get_resource_content)
        .service(get_resource_preview_url) // OSS 直链预览 URL
        .service(get_like_status) // 获取点赞状态（支持未登录用户）
        .service(get_comments) // 获取评论列表（公开）
        .service(get_resource_ratings) // 获取资源评分信息（支持未登录用户）
        .service(get_resource_relations); // /resources/{id}/relations
}

/// 提交评分
#[post("/resources/{resource_id}/rate")]
pub async fn rate_resource(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    request: web::Json<CreateRatingRequest>,
    req: HttpRequest,
) -> impl Responder {
    let resource_id = path.into_inner();

    // 获取资源信息（用于审计日志）
    let resource_detail = match ResourceService::get_resource_detail(&state.pool, resource_id).await
    {
        Ok(detail) => detail,
        Err(e) => {
            log::warn!(
                "[Resource] 获取资源详情失败 | resource_id={}, user_id={}, error={}",
                resource_id,
                user.id,
                e
            );
            return match e {
                ResourceError::NotFound(msg) => not_found(&msg),
                _ => internal_error("获取资源详情失败"),
            };
        }
    };

    let overall_quality = request.overall_quality;

    match RatingService::create_or_update_rating(
        &state.pool,
        resource_id,
        user.id,
        request.into_inner(),
    )
    .await
    {
        Ok(rating) => {
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

            HttpResponse::Ok().json(rating)
        }
        Err(e) => {
            log::warn!(
                "[Resource] 评分失败 | resource_id={}, user_id={}, error={}",
                resource_id,
                user.id,
                e
            );
            bad_request("评分失败")
        }
    }
}

/// 获取当前用户的评分
#[get("/resources/{resource_id}/rate")]
pub async fn get_my_rating(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let resource_id = path.into_inner();

    match RatingService::get_user_rating(&state.pool, resource_id, user.id).await {
        Ok(rating) => HttpResponse::Ok().json(rating),
        Err(e) => {
            log::warn!(
                "[Resource] 获取评分失败 | resource_id={}, user_id={}, error={}",
                resource_id,
                user.id,
                e
            );
            internal_error("获取失败")
        }
    }
}

/// 获取资源评分信息（包含所有维度的平均分，支持未登录用户）
#[get("/resources/{resource_id}/ratings")]
pub async fn get_resource_ratings(
    state: web::Data<AppState>,
    user: Option<web::ReqData<CurrentUser>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let resource_id = path.into_inner();
    let user_id = user.map(|u| u.id);

    match RatingService::get_resource_rating_info(&state.pool, resource_id, user_id).await {
        Ok(info) => HttpResponse::Ok().json(info),
        Err(e) => {
            log::warn!(
                "[Resource] 获取资源评分信息失败 | resource_id={}, error={}",
                resource_id,
                e
            );
            internal_error("获取评分信息失败")
        }
    }
}

/// 删除评分
#[delete("/resources/{resource_id}/rate")]
pub async fn delete_rating(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let resource_id = path.into_inner();

    match RatingService::delete_rating(&state.pool, resource_id, user.id).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => {
            log::warn!(
                "[Resource] 删除评分失败 | resource_id={}, user_id={}, error={}",
                resource_id,
                user.id,
                e
            );
            internal_error("删除失败")
        }
    }
}

/// 点赞/取消点赞
#[post("/resources/{resource_id}/like")]
pub async fn toggle_like(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> impl Responder {
    let resource_id = path.into_inner();

    // 获取资源信息（用于审计日志）
    let resource_detail = match ResourceService::get_resource_detail(&state.pool, resource_id).await {
        Ok(detail) => detail,
        Err(e) => {
            log::warn!(
                "[Resource] 获取资源详情失败 | resource_id={}, user_id={}, error={}",
                resource_id,
                user.id,
                e
            );
            return match e {
                ResourceError::NotFound(msg) => not_found(&msg),
                _ => internal_error("获取资源详情失败"),
            };
        }
    };

    match LikeService::toggle_like(&state.pool, resource_id, user.id).await {
        Ok(result) => {
            // 记录审计日志
            let ip_address = req.peer_addr().map(|addr| addr.ip().to_string());
            if let Err(e) = AuditLogService::log_like_resource(
                &state.pool,
                user.id,
                resource_id,
                &resource_detail.title,
                result.is_liked,
                ip_address.as_deref(),
            )
            .await
            {
                log::warn!(
                    "[Audit] 记录点赞日志失败 | resource_id={}, error={}",
                    resource_id,
                    e
                );
            }

            // 转换为 camelCase 的响应结构
            let response_data = crate::models::LikeToggleResponse {
                is_liked: result.is_liked,
                like_count: result.like_count,
                message: result.message.clone(),
            };
            HttpResponse::Ok().json(response_data)
        }
        Err(e) => {
            log::warn!(
                "[Resource] 点赞操作失败 | resource_id={}, user_id={}, error={}",
                resource_id,
                user.id,
                e
            );
            internal_error("操作失败")
        }
    }
}

/// 获取点赞状态
#[get("/resources/{resource_id}/like")]
pub async fn get_like_status(
    state: web::Data<AppState>,
    user: Option<web::ReqData<CurrentUser>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let resource_id = path.into_inner();

    // 如果有用户登录，检查该用户的点赞状态；否则返回未点赞
    let (is_liked, like_count) = if let Some(user) = user {
        match LikeService::check_like_status(&state.pool, resource_id, user.id).await {
            Ok(status) => (status.is_liked, status.like_count),
            Err(e) => {
                log::warn!(
                    "[Resource] 获取点赞状态失败 | resource_id={}, user_id={}, error={}",
                    resource_id,
                    user.id,
                    e
                );
                (false, 0)
            }
        }
    } else {
        // 未登录用户，获取点赞数但不显示已点赞
        match LikeService::get_like_count(&state.pool, resource_id).await {
            Ok(count) => (false, count),
            Err(e) => {
                log::warn!(
                    "[Resource] 获取点赞数失败 | resource_id={}, error={}",
                    resource_id,
                    e
                );
                (false, 0)
            }
        }
    };

    // 使用 LikeStatusResponse 结构体，确保字段名使用 camelCase
    let response_data = crate::models::LikeStatusResponse {
        is_liked,
        like_count,
    };

    HttpResponse::Ok().json(response_data)
}

/// 获取评论列表（公开接口，不需要登录）
#[get("/resources/{resource_id}/comments")]
pub async fn get_comments(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    query: web::Query<CommentListQuery>,
) -> impl Responder {
    let resource_id = path.into_inner();

    match CommentService::get_comments(&state.pool, resource_id, query.into_inner()).await {
        Ok(comments) => HttpResponse::Ok().json(comments),
        Err(e) => {
            log::warn!(
                "[Resource] 获取评论失败 | resource_id={}, error={}",
                resource_id,
                e
            );
            match e {
                ResourceError::NotFound(msg) => not_found(&msg),
                _ => internal_error("获取评论失败"),
            }
        }
    }
}

/// 发表评论
#[post("/resources/{resource_id}/comments")]
pub async fn create_comment(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    request: web::Json<CreateCommentRequest>,
    req: HttpRequest,
) -> impl Responder {
    let resource_id = path.into_inner();

    // 获取资源信息（用于审计日志）
    let resource_detail = match ResourceService::get_resource_detail(&state.pool, resource_id).await
    {
        Ok(detail) => detail,
        Err(e) => {
            log::warn!(
                "[Resource] 获取资源详情失败 | resource_id={}, user_id={}, error={}",
                resource_id,
                user.id,
                e
            );
            return match e {
                ResourceError::NotFound(msg) => not_found(&msg),
                _ => internal_error("获取资源详情失败"),
            };
        }
    };

    match CommentService::create_comment(&state.pool, resource_id, user.id, request.into_inner())
        .await
    {
        Ok(comment) => {
            // 记录审计日志
            let ip_address = req.peer_addr().map(|addr| addr.ip().to_string());
            if let Err(e) = AuditLogService::log_create_comment(
                &state.pool,
                user.id,
                resource_id,
                comment.id,
                &resource_detail.title,
                ip_address.as_deref(),
            )
            .await
            {
                log::warn!(
                    "[Audit] 记录发表评论日志失败 | comment_id={}, error={}",
                    comment.id,
                    e
                );
            }

            HttpResponse::Created().json(comment)
        }
        Err(e) => {
            log::warn!(
                "[Resource] 发表评论失败 | resource_id={}, user_id={}, error={}",
                resource_id,
                user.id,
                e
            );
            match e {
                ResourceError::ValidationError(msg) => bad_request(&msg),
                ResourceError::NotFound(msg) => not_found(&msg),
                _ => internal_error("评论失败"),
            }
        }
    }
}

/// 更新资源关联信息
#[put("/resources/{resource_id}/relations")]
pub async fn update_resource_relations(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    request: web::Json<UpdateResourceRelationsRequest>,
    req: HttpRequest,
) -> impl Responder {
    let resource_id = path.into_inner();

    // 获取资源信息（用于权限检查和审计日志）
    let resource_detail = match ResourceService::get_resource_detail(&state.pool, resource_id).await {
        Ok(detail) => detail,
        Err(e) => {
            log::warn!(
                "[Resource] 获取资源详情失败 | resource_id={}, user_id={}, error={}",
                resource_id,
                user.id,
                e
            );
            return match e {
                ResourceError::NotFound(msg) => not_found(&msg),
                _ => internal_error("获取资源详情失败"),
            };
        }
    };

    // 检查权限（上传者或管理员可以修改）
    if resource_detail.uploader_id != user.id && user.role != crate::models::UserRole::Admin {
        return forbidden("只有资源上传者或管理员可以修改关联信息");
    }

    match ResourceService::update_resource_relations(
        &state.pool,
        resource_id,
        request.teacher_sns.clone(),
        request.course_sns.clone(),
        request.related_resource_ids.clone(),
    )
    .await
    {
        Ok(_) => {
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
                    "[Audit] 记录资源关联更新日志失败 | resource_id={}, error={}",
                    resource_id,
                    e
                );
            }

            HttpResponse::Ok().json(serde_json::json!({
                "message": "关联信息更新成功"
            }))
        }
        Err(e) => {
            log::warn!(
                "[Resource] 更新资源关联信息失败 | resource_id={}, user_id={}, error={}",
                resource_id,
                user.id,
                e
            );
            match e {
                ResourceError::NotFound(msg) => not_found(&msg),
                ResourceError::Unauthorized(msg) => forbidden(&msg),
                ResourceError::ValidationError(msg) => bad_request(&msg),
                _ => internal_error("更新关联信息失败"),
            }
        }
    }
}

/// 更新资源描述
#[put("/resources/{resource_id}/description")]
pub async fn update_resource_description(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    request: web::Json<UpdateResourceDescriptionRequest>,
    req: HttpRequest,
) -> impl Responder {
    let resource_id = path.into_inner();

    // 验证请求
    if let Err(msg) = request.validate() {
        return bad_request(&msg);
    }

    // 获取资源信息（用于审计日志）
    let resource_detail = match ResourceService::get_resource_detail(&state.pool, resource_id).await {
        Ok(detail) => detail,
        Err(e) => {
            log::warn!(
                "[Resource] 获取资源详情失败 | resource_id={}, user_id={}, error={}",
                resource_id,
                user.id,
                e
            );
            return match e {
                ResourceError::NotFound(msg) => not_found(&msg),
                _ => internal_error("获取资源详情失败"),
            };
        }
    };

    match ResourceService::update_resource_description(
        &state.pool,
        &user,
        resource_id,
        request.description.clone(),
    )
    .await
    {
        Ok(_) => {
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
                    "[Audit] 记录资源描述更新日志失败 | resource_id={}, error={}",
                    resource_id,
                    e
                );
            }

            HttpResponse::Ok().json(serde_json::json!({
                "message": "资源描述更新成功"
            }))
        }
        Err(e) => {
            log::warn!(
                "[Resource] 更新资源描述失败 | resource_id={}, user_id={}, error={}",
                resource_id,
                user.id,
                e
            );
            match e {
                ResourceError::NotFound(msg) => not_found(&msg),
                ResourceError::Unauthorized(msg) => forbidden(&msg),
                ResourceError::ValidationError(msg) => bad_request(&msg),
                _ => internal_error("更新资源描述失败"),
            }
        }
    }
}

/// 获取 PDF 预览检测配置
#[get("/resources/pdf-preview-challenge/config")]
pub async fn get_pdf_preview_challenge_config(
    state: web::Data<AppState>,
) -> impl Responder {
    match &state.pdf_preview_challenge_uuid {
        Some(uuid) => {
            HttpResponse::Ok().json(serde_json::json!({
                "resourceId": uuid,
                "enabled": true
            }))
        }
        None => {
            HttpResponse::Ok().json(serde_json::json!({
                "resourceId": null,
                "enabled": false
            }))
        }
    }
}

/// 验证 PDF 预览检测答案
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct VerifyPdfPreviewChallengeRequest {
    code: String,
}

#[post("/resources/pdf-preview-challenge/verify")]
pub async fn verify_pdf_preview_challenge(
    state: web::Data<AppState>,
    request: web::Json<VerifyPdfPreviewChallengeRequest>,
) -> impl Responder {
    // 检查是否配置了挑战
    if state.pdf_preview_challenge_uuid.is_none() || state.pdf_preview_challenge_code.is_none() {
        return HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "ServiceUnavailable",
            "message": "PDF预览检测功能未配置"
        }));
    }

    let expected_code = state.pdf_preview_challenge_code.as_ref().unwrap();

    // 验证答案
    if request.code.trim() == expected_code.trim() {
        HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "验证成功"
        }))
    } else {
        HttpResponse::Ok().json(serde_json::json!({
            "success": false,
            "message": "验证码错误"
        }))
    }
}

/// 配置资源路由（需要认证）
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(upload_resource)
        .service(delete_resource)
        .service(get_my_resources)
        .service(rate_resource)
        .service(get_my_rating)
        .service(delete_rating)
        .service(toggle_like)
        .service(create_comment)
        .service(update_resource_content)
        .service(get_resource_raw_content)
        .service(update_resource_relations)
        .service(update_resource_description);
}
