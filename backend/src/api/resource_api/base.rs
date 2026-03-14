use actix_multipart::Multipart;
use actix_web::{delete, get, post, web, HttpRequest, HttpResponse, Responder};
use futures_util::StreamExt;
use uuid::Uuid;

use crate::db::AppState;
use crate::models::{
    CurrentUser, HotResourcesQuery, ResourceListQuery, ResourceSearchQuery, UploadResourceRequest,
};
use crate::services::{AuditLogService, ResourceError, ResourceService};
use crate::utils::{bad_request, conflict, internal_error, not_found};

use super::{ResourceCountResponse, ResourceSearchForRelationQuery};

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
                while field.next().await.is_some() {}
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
                ResourceError::Unauthorized(msg) => crate::utils::forbidden(&msg),
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
                ResourceError::Unauthorized(msg) => crate::utils::forbidden(&msg),
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

/// 获取热门资源列表
#[get("/resources/hot")]
pub async fn get_hot_resources(
    state: web::Data<AppState>,
    query: web::Query<HotResourcesQuery>,
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

    let exclude_id = query
        .exclude_id
        .clone()
        .and_then(|id| Uuid::parse_str(&id).ok());

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
