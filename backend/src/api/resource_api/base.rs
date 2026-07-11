use actix_multipart::Multipart;
use actix_web::{delete, get, post, web, HttpRequest, HttpResponse};
use futures_util::StreamExt;
use uuid::Uuid;

use crate::db::AppState;
use crate::models::{
    CurrentUser, HotResourcesQuery, ResourceListQuery, ResourceSearchQuery, UploadResourceRequest,
};
use crate::services::{AuditLogService, ResourceError, ResourceService};
use crate::utils::bad_request;

use super::{ResourceCountResponse, ResourceSearchForRelationQuery};

/// 上传资源
#[post("/resources")]
pub async fn upload_resource(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    mut payload: Multipart,
    req: HttpRequest,
) -> Result<HttpResponse, ResourceError> {
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
                return Ok(bad_request("解析上传数据失败"));
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
                            return Ok(bad_request("读取元数据失败"));
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
                        return Ok(bad_request(&format!("元数据格式错误: {}", e)));
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
                            return Ok(bad_request("读取文件数据失败"));
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
    let Some(metadata) = metadata else {
        return Ok(bad_request("缺少资源元数据"));
    };

    // 检查是否有文件数据
    let Some((filename, data, mime_type)) = file_data else {
        return Ok(bad_request("请选择要上传的文件"));
    };

    // 调用服务上传资源
    let response = ResourceService::upload_resource(
        &state.pool,
        &user,
        &state.storage,
        metadata,
        &filename,
        data,
        mime_type.as_deref(),
    )
    .await?;

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

    Ok(HttpResponse::Created().json(response))
}

/// 获取资源列表
#[get("/resources")]
pub async fn get_resource_list(
    state: web::Data<AppState>,
    query: web::Query<ResourceListQuery>,
) -> Result<HttpResponse, ResourceError> {
    let response = ResourceService::get_resource_list(&state.pool, &query).await?;
    Ok(HttpResponse::Ok().json(response))
}

/// 搜索资源
#[get("/resources/search")]
pub async fn search_resources(
    state: web::Data<AppState>,
    query: web::Query<ResourceSearchQuery>,
) -> Result<HttpResponse, ResourceError> {
    // 验证搜索关键词
    if query.q.trim().is_empty() {
        return Ok(bad_request("搜索关键词不能为空"));
    }

    let response = ResourceService::search_resources(&state.pool, &query).await?;
    Ok(HttpResponse::Ok().json(response))
}

/// 获取资源详情
#[get("/resources/{resource_id}")]
pub async fn get_resource_detail(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ResourceError> {
    let resource_id = path.into_inner();

    // 增加浏览量
    let _ = ResourceService::increment_views(&state.pool, resource_id).await;

    let response = ResourceService::get_resource_detail(&state.pool, resource_id).await?;
    Ok(HttpResponse::Ok().json(response))
}

/// 删除资源
#[delete("/resources/{resource_id}")]
pub async fn delete_resource(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<HttpResponse, ResourceError> {
    let resource_id = path.into_inner();

    log::info!(
        "[Resource] 删除资源 | resource_id={}, user_id={}",
        resource_id,
        user.id
    );

    let title =
        ResourceService::delete_resource(&state.pool, &user, &state.storage, resource_id).await?;

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

    Ok(HttpResponse::NoContent().finish())
}

/// 获取当前用户的资源列表
#[get("/resources/my")]
pub async fn get_my_resources(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    query: web::Query<ResourceListQuery>,
) -> Result<HttpResponse, ResourceError> {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20).min(100);

    let response =
        ResourceService::get_user_resources(&state.pool, user.id, page, per_page).await?;
    Ok(HttpResponse::Ok().json(response))
}

/// 获取热门资源列表
#[get("/resources/hot")]
pub async fn get_hot_resources(
    state: web::Data<AppState>,
    query: web::Query<HotResourcesQuery>,
) -> Result<HttpResponse, ResourceError> {
    let limit = query.limit.unwrap_or(10);

    let resources = ResourceService::get_hot_resources(&state.pool, limit).await?;
    Ok(HttpResponse::Ok().json(resources))
}

/// 获取资源总数
#[get("/resources/count")]
pub async fn get_resource_count(state: web::Data<AppState>) -> Result<HttpResponse, ResourceError> {
    let count = ResourceService::get_resource_count(&state.pool).await?;
    Ok(HttpResponse::Ok().json(ResourceCountResponse { total: count }))
}

/// 搜索可关联的资源
/// 用于在上传资源时搜索要关联的其他资源
#[get("/resources/search-for-relation")]
pub async fn search_resources_for_relation(
    state: web::Data<AppState>,
    query: web::Query<ResourceSearchForRelationQuery>,
) -> Result<HttpResponse, ResourceError> {
    // 验证搜索关键词
    if query.q.trim().is_empty() {
        return Ok(bad_request("搜索关键词不能为空"));
    }

    let exclude_id = query
        .exclude_id
        .clone()
        .and_then(|id| Uuid::parse_str(&id).ok());

    let resources = ResourceService::search_resources_for_relation(
        &state.pool,
        &query.q,
        exclude_id,
        query.limit.unwrap_or(10),
    )
    .await?;
    Ok(HttpResponse::Ok().json(resources))
}

/// 根据文件哈希查询资源
/// 用于上传前检查是否已存在相同内容的资源
#[get("/resources/by-hash/{file_hash}")]
pub async fn get_resources_by_hash(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ResourceError> {
    let file_hash = path.into_inner();

    // 验证哈希格式（应该是64位十六进制字符串，即SHA256）
    if file_hash.len() != 64 || !file_hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Ok(bad_request("无效的哈希格式，应为64位十六进制字符串"));
    }

    let resources = ResourceService::find_by_file_hash(&state.pool, &file_hash).await?;
    Ok(HttpResponse::Ok().json(resources))
}
