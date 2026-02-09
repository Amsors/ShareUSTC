use actix_multipart::Multipart;
use actix_web::{delete, get, post, web, HttpRequest, HttpResponse, Responder};
use futures_util::StreamExt;
use uuid::Uuid;

use crate::db::AppState;
use crate::models::{
    resource::*,
    CreateRatingRequest, CreateCommentRequest, CommentListQuery,
    CurrentUser,
};
use crate::services::{ResourceService, RatingService, LikeService, CommentService};

/// 上传资源
#[post("/resources")]
pub async fn upload_resource(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    mut payload: Multipart,
) -> impl Responder {
    let mut metadata: Option<UploadResourceRequest> = None;
    let mut file_data: Option<(String, Vec<u8>, Option<String>)> = None;

    // 解析 multipart 表单数据
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

        match field_name {
            "metadata" => {
                // 读取元数据 JSON
                let mut data = Vec::new();
                while let Some(chunk) = field.next().await {
                    match chunk {
                        Ok(bytes) => data.extend_from_slice(&bytes),
                        Err(e) => {
                            log::warn!("读取元数据失败: {}", e);
                            return HttpResponse::Ok().json(serde_json::json!({
                                "code": 400,
                                "message": "读取元数据失败",
                                "data": null
                            }));
                        }
                    }
                }

                // 解析 JSON
                match serde_json::from_slice::<UploadResourceRequest>(&data) {
                    Ok(req) => metadata = Some(req),
                    Err(e) => {
                        log::warn!("解析元数据 JSON 失败: {}", e);
                        return HttpResponse::Ok().json(serde_json::json!({
                            "code": 400,
                            "message": format!("元数据格式错误: {}", e),
                            "data": null
                        }));
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
            return HttpResponse::Ok().json(serde_json::json!({
                "code": 400,
                "message": "缺少资源元数据",
                "data": null
            }));
        }
    };

    // 检查是否有文件数据
    let (filename, data, mime_type) = match file_data {
        Some(d) => d,
        None => {
            return HttpResponse::Ok().json(serde_json::json!({
                "code": 400,
                "message": "请选择要上传的文件",
                "data": null
            }));
        }
    };

    // 调用服务上传资源
    match ResourceService::upload_resource(
        &state.pool,
        &user,
        metadata,
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
            log::error!("上传资源失败: {:?}", e);
            let (code, message) = match e {
                crate::services::ResourceError::ValidationError(msg) => (400, msg),
                crate::services::ResourceError::FileError(msg) => (500, msg),
                crate::services::ResourceError::DatabaseError(msg) => {
                    log::error!("数据库错误详情: {}", msg);
                    (500, format!("数据库错误: {}", msg))
                },
                crate::services::ResourceError::AiError(msg) => (500, msg),
                crate::services::ResourceError::NotFound(msg) => (404, msg),
                crate::services::ResourceError::Unauthorized(msg) => (403, msg),
            };
            HttpResponse::Ok().json(serde_json::json!({
                "code": code,
                "message": message,
                "data": null
            }))
        }
    }
}

/// 获取资源列表
#[get("/resources")]
pub async fn get_resource_list(
    state: web::Data<AppState>,
    query: web::Query<ResourceListQuery>,
) -> impl Responder {
    // TODO: 以下延迟仅用于测试加载效果，生产环境请删除
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    match ResourceService::get_resource_list(&state.pool, &query).await {
        Ok(response) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "获取成功",
            "data": response
        })),
        Err(e) => {
            log::warn!("获取资源列表失败: {}", e);
            HttpResponse::Ok().json(serde_json::json!({
                "code": 500,
                "message": "获取资源列表失败",
                "data": null
            }))
        }
    }
}

/// 搜索资源
#[get("/resources/search")]
pub async fn search_resources(
    state: web::Data<AppState>,
    query: web::Query<ResourceSearchQuery>,
) -> impl Responder {
    // TODO: 以下延迟仅用于测试加载效果，生产环境请删除
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // 验证搜索关键词
    if query.q.trim().is_empty() {
        return HttpResponse::Ok().json(serde_json::json!({
            "code": 400,
            "message": "搜索关键词不能为空",
            "data": null
        }));
    }

    match ResourceService::search_resources(&state.pool, &query).await {
        Ok(response) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "搜索成功",
            "data": response
        })),
        Err(e) => {
            log::warn!("搜索资源失败: {}", e);
            HttpResponse::Ok().json(serde_json::json!({
                "code": 500,
                "message": "搜索资源失败",
                "data": null
            }))
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
        Ok(response) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "获取成功",
            "data": response
        })),
        Err(e) => {
            log::warn!("获取资源详情失败: {}", e);
            let (code, message) = match e {
                crate::services::ResourceError::NotFound(msg) => (404, msg),
                _ => (500, "获取资源详情失败".to_string()),
            };
            HttpResponse::Ok().json(serde_json::json!({
                "code": code,
                "message": message,
                "data": null
            }))
        }
    }
}

/// 删除资源
#[delete("/resources/{resource_id}")]
pub async fn delete_resource(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let resource_id = path.into_inner();

    match ResourceService::delete_resource(&state.pool, &user, resource_id).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "删除成功",
            "data": null
        })),
        Err(e) => {
            log::warn!("删除资源失败: {}", e);
            let (code, message) = match e {
                crate::services::ResourceError::NotFound(msg) => (404, msg),
                crate::services::ResourceError::Unauthorized(msg) => (403, msg),
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
        Ok(response) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "获取成功",
            "data": response
        })),
        Err(e) => {
            log::warn!("获取我的资源列表失败: {}", e);
            HttpResponse::Ok().json(serde_json::json!({
                "code": 500,
                "message": "获取资源列表失败",
                "data": null
            }))
        }
    }
}

/// 下载资源
#[get("/resources/{resource_id}/download")]
pub async fn download_resource(
    state: web::Data<AppState>,
    user: Option<web::ReqData<CurrentUser>>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> impl Responder {
    let resource_id = path.into_inner();

    // 获取资源文件路径
    match ResourceService::get_resource_file_path(&state.pool, resource_id).await {
        Ok((file_path, resource_type)) => {
            // 读取文件
            match crate::services::FileService::read_resource_file(&file_path).await {
                Ok(file_content) => {
                    // 增加下载次数
                    let _ = ResourceService::increment_downloads(&state.pool, resource_id).await;

                    // 记录下载日志
                    let user_id = user.as_ref().map(|u| u.id);

                    // 获取真实 IP 地址
                    let ip_address = req
                        .peer_addr()
                        .map(|addr| addr.ip().to_string())
                        .unwrap_or_else(|| "0.0.0.0".to_string());

                    // 调用 Service 层记录下载日志
                    let _ = ResourceService::record_download(
                        &state.pool,
                        resource_id,
                        user_id,
                        &ip_address
                    ).await;

                    // 设置 Content-Type 和 Content-Disposition
                    // 使用 resource_type 获取 MIME 类型，因为它更准确
                    let content_type = crate::services::FileService::get_mime_type_by_type(&resource_type);
                    let filename = std::path::Path::new(&file_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("download");

                    HttpResponse::Ok()
                        .content_type(content_type)
                        .insert_header((
                            "Content-Disposition",
                            format!("attachment; filename=\"{}\"", filename),
                        ))
                        .body(file_content)
                }
                Err(e) => {
                    log::warn!("读取资源文件失败: {}", e);
                    HttpResponse::Ok().json(serde_json::json!({
                        "code": 500,
                        "message": "文件读取失败",
                        "data": null
                    }))
                }
            }
        }
        Err(e) => {
            log::warn!("获取资源文件路径失败: {}", e);
            let (code, message) = match e {
                crate::services::ResourceError::NotFound(msg) => (404, msg),
                _ => (500, "获取资源失败".to_string()),
            };
            HttpResponse::Ok().json(serde_json::json!({
                "code": code,
                "message": message,
                "data": null
            }))
        }
    }
}

/// 获取资源文件内容（用于预览）
#[get("/resources/{resource_id}/content")]
pub async fn get_resource_content(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let resource_id = path.into_inner();

    // 获取资源文件路径（预览不检查审核状态）
    match ResourceService::get_resource_file_path_for_preview(&state.pool, resource_id).await {
        Ok((file_path, resource_type)) => {
            // 读取文件
            match crate::services::FileService::read_resource_file(&file_path).await {
                Ok(file_content) => {
                    // 获取 MIME 类型 - 优先使用 resource_type，因为它更准确
                    let content_type = crate::services::FileService::get_mime_type_by_type(&resource_type);

                    log::debug!("预览资源 {}: 文件路径={}, 类型={}, MIME={}",
                        resource_id, file_path, resource_type, content_type);

                    // 返回文件内容（inline 显示，不是下载）
                    HttpResponse::Ok()
                        .content_type(content_type)
                        .insert_header(("Cache-Control", "public, max-age=3600"))
                        .body(file_content)
                }
                Err(e) => {
                    log::warn!("读取资源文件失败: {}", e);
                    HttpResponse::Ok().json(serde_json::json!({
                        "code": 500,
                        "message": "文件读取失败",
                        "data": null
                    }))
                }
            }
        }
        Err(e) => {
            log::warn!("获取资源文件路径失败: {}", e);
            let (code, message) = match e {
                crate::services::ResourceError::NotFound(msg) => (404, msg),
                _ => (500, "获取资源失败".to_string()),
            };
            HttpResponse::Ok().json(serde_json::json!({
                "code": code,
                "message": message,
                "data": null
            }))
        }
    }
}

/// 配置公开资源路由（不需要认证）
pub fn config_public(cfg: &mut web::ServiceConfig) {
    cfg.service(get_resource_list)
        .service(search_resources)
        .service(get_resource_detail)
        .service(download_resource)
        .service(get_resource_content)
        .service(get_like_status)  // 获取点赞状态（支持未登录用户）
        .service(get_comments);    // 获取评论列表（公开）
}

/// 提交评分
#[post("/resources/{resource_id}/rate")]
pub async fn rate_resource(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    request: web::Json<CreateRatingRequest>,
) -> impl Responder {
    let resource_id = path.into_inner();

    match RatingService::create_or_update_rating(&state.pool, resource_id, user.id, request.into_inner()).await {
        Ok(rating) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "评分成功",
            "data": rating
        })),
        Err(e) => {
            log::warn!("评分失败: {}", e);
            HttpResponse::Ok().json(serde_json::json!({
                "code": 400,
                "message": "评分失败",
                "data": null
            }))
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
        Ok(rating) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "获取成功",
            "data": rating
        })),
        Err(e) => {
            log::warn!("获取评分失败: {}", e);
            HttpResponse::Ok().json(serde_json::json!({
                "code": 500,
                "message": "获取失败",
                "data": null
            }))
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
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "删除成功",
            "data": null
        })),
        Err(e) => {
            log::warn!("删除评分失败: {}", e);
            HttpResponse::Ok().json(serde_json::json!({
                "code": 500,
                "message": "删除失败",
                "data": null
            }))
        }
    }
}

/// 点赞/取消点赞
#[post("/resources/{resource_id}/like")]
pub async fn toggle_like(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let resource_id = path.into_inner();

    match LikeService::toggle_like(&state.pool, resource_id, user.id).await {
        Ok(result) => {
            // 转换为 camelCase 的响应结构
            let response_data = crate::models::LikeToggleResponse {
                is_liked: result.is_liked,
                like_count: result.like_count,
                message: result.message.clone(),
            };
            HttpResponse::Ok().json(serde_json::json!({
                "code": 200,
                "message": result.message,
                "data": response_data
            }))
        }
        Err(e) => {
            log::warn!("点赞操作失败: {}", e);
            HttpResponse::Ok().json(serde_json::json!({
                "code": 500,
                "message": "操作失败",
                "data": null
            }))
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
                log::warn!("获取点赞状态失败: {}", e);
                (false, 0)
            }
        }
    } else {
        // 未登录用户，获取点赞数但不显示已点赞
        match LikeService::get_like_count(&state.pool, resource_id).await {
            Ok(count) => (false, count),
            Err(e) => {
                log::warn!("获取点赞数失败: {}", e);
                (false, 0)
            }
        }
    };

    // 使用 LikeStatusResponse 结构体，确保字段名使用 camelCase
    let response_data = crate::models::LikeStatusResponse {
        is_liked,
        like_count,
    };

    HttpResponse::Ok().json(serde_json::json!({
        "code": 200,
        "message": "获取成功",
        "data": response_data
    }))
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
        Ok(comments) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "获取成功",
            "data": comments
        })),
        Err(e) => {
            log::warn!("获取评论失败: {}", e);
            let message = match e {
                crate::services::ResourceError::NotFound(msg) => msg,
                _ => "获取评论失败".to_string(),
            };
            HttpResponse::Ok().json(serde_json::json!({
                "code": 500,
                "message": message,
                "data": null
            }))
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
) -> impl Responder {
    let resource_id = path.into_inner();

    match CommentService::create_comment(&state.pool, resource_id, user.id, request.into_inner()).await {
        Ok(comment) => HttpResponse::Ok().json(serde_json::json!({
            "code": 200,
            "message": "评论成功",
            "data": comment
        })),
        Err(e) => {
            log::warn!("评论失败: {}", e);
            let (code, message) = match e {
                crate::services::ResourceError::ValidationError(msg) => (400, msg),
                crate::services::ResourceError::NotFound(msg) => (404, msg),
                _ => (500, "评论失败".to_string()),
            };
            HttpResponse::Ok().json(serde_json::json!({
                "code": code,
                "message": message,
                "data": null
            }))
        }
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
        .service(create_comment);
}
