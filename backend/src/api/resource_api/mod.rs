use actix_web::web;
use uuid::Uuid;

use crate::db::AppState;

mod base;
mod comment;
mod file;
mod like;
pub mod pdf_challenge;
mod rating;
mod relation;

pub use base::*;
pub use comment::*;
pub use file::*;
pub use like::*;
pub use pdf_challenge::*;
pub use rating::*;
pub use relation::*;

/// 记录下载事件（Web 层适配：提取 IP 后委托给 service 层编排）
async fn record_download_events(
    state: &web::Data<AppState>,
    resource_id: Uuid,
    user_id: Option<Uuid>,
    title: &str,
    req: &actix_web::HttpRequest,
) {
    let ip_address = req
        .peer_addr()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "0.0.0.0".to_string());

    crate::services::ResourceService::record_download_event(
        &state.pool,
        resource_id,
        user_id,
        title,
        &ip_address,
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

/// 搜索可关联资源的查询参数
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceSearchForRelationQuery {
    pub q: String,
    pub exclude_id: Option<String>,
    pub limit: Option<i32>,
}

/// 资源数量响应
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceCountResponse {
    pub total: i64,
}
