//! 资源服务模块
//!
//! 该模块提供资源管理相关的所有功能，包括：
//! - 资源上传（包括 OSS 回调上传）
//! - 资源查询（详情、列表、搜索）
//! - 资源修改（删除、内容更新、描述更新、关联更新）
//! - 文件访问（下载、预览、原始内容获取）
//! - 资源关联（搜索可关联资源、获取关联资源列表）

pub mod error;
pub mod file_access;
pub mod modify;
pub mod query;
pub mod relation;
pub mod upload;
pub mod utils;

// 重新导出错误类型
pub use error::ResourceError;

// 为了保持向后兼容，保留 ResourceService 结构体
// 所有方法都委托给具体的函数
use crate::config::Config;
use crate::models::{resource::*, CurrentUser, UpdateResourceContentResponse};
use crate::services::storage_service::{StorageBackend, StorageFileMetadata};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

/// 资源服务结构体
///
/// 注意：该结构体的方法已迁移为独立函数，
/// 建议直接使用模块级别的函数。
pub struct ResourceService;

impl ResourceService {
    /// 从 OSS 回调创建资源
    pub async fn create_resource_from_oss_callback(
        pool: &PgPool,
        user: &CurrentUser,
        storage: &Arc<dyn StorageBackend>,
        request: UploadResourceRequest,
        oss_key: &str,
        metadata: StorageFileMetadata,
    ) -> Result<UploadResourceResponse, ResourceError> {
        upload::create_resource_from_oss_callback(pool, user, storage, request, oss_key, metadata)
            .await
    }

    /// 上传资源
    pub async fn upload_resource(
        pool: &PgPool,
        user: &CurrentUser,
        storage: &Arc<dyn StorageBackend>,
        request: UploadResourceRequest,
        file_name: &str,
        file_data: Vec<u8>,
        mime_type: Option<&str>,
    ) -> Result<UploadResourceResponse, ResourceError> {
        upload::upload_resource(
            pool, user, storage, request, file_name, file_data, mime_type,
        )
        .await
    }

    /// 获取资源详情
    pub async fn get_resource_detail(
        pool: &PgPool,
        resource_id: Uuid,
    ) -> Result<ResourceDetailResponse, ResourceError> {
        query::get_resource_detail(pool, resource_id).await
    }

    /// 获取资源列表
    pub async fn get_resource_list(
        pool: &PgPool,
        query: &ResourceListQuery,
    ) -> Result<ResourceListResponse, ResourceError> {
        query::get_resource_list(pool, query).await
    }

    /// 搜索资源
    pub async fn search_resources(
        pool: &PgPool,
        query: &ResourceSearchQuery,
    ) -> Result<ResourceListResponse, ResourceError> {
        query::search_resources(pool, query).await
    }

    /// 删除资源
    pub async fn delete_resource(
        pool: &PgPool,
        user: &CurrentUser,
        storage: &Arc<dyn StorageBackend>,
        resource_id: Uuid,
    ) -> Result<String, ResourceError> {
        modify::delete_resource(pool, user, storage, resource_id).await
    }

    /// 获取用户上传的资源列表
    pub async fn get_user_resources(
        pool: &PgPool,
        user_id: Uuid,
        page: i32,
        per_page: i32,
    ) -> Result<ResourceListResponse, ResourceError> {
        query::get_user_resources(pool, user_id, page, per_page).await
    }

    /// 增加下载次数
    pub async fn increment_downloads(
        pool: &PgPool,
        resource_id: Uuid,
    ) -> Result<(), ResourceError> {
        file_access::increment_downloads(pool, resource_id).await
    }

    /// 增加访问次数
    pub async fn increment_views(pool: &PgPool, resource_id: Uuid) -> Result<(), ResourceError> {
        file_access::increment_views(pool, resource_id).await
    }

    /// 获取资源文件路径（用于下载）
    pub async fn get_resource_file_path(
        pool: &PgPool,
        resource_id: Uuid,
        user: Option<&CurrentUser>,
    ) -> Result<(String, String, String, Option<String>), ResourceError> {
        file_access::get_resource_file_path(pool, resource_id, user).await
    }

    /// 获取资源文件路径（用于预览）
    pub async fn get_resource_file_path_for_preview(
        pool: &PgPool,
        resource_id: Uuid,
        user: Option<&CurrentUser>,
    ) -> Result<(String, String, Option<String>, chrono::NaiveDateTime), ResourceError> {
        file_access::get_resource_file_path_for_preview(pool, resource_id, user).await
    }

    /// 记录一次下载事件（递增计数 + 下载日志 + 审计日志，best-effort）
    pub async fn record_download_event(
        pool: &PgPool,
        resource_id: Uuid,
        user_id: Option<Uuid>,
        title: &str,
        ip_address: &str,
    ) {
        file_access::record_download_event(pool, resource_id, user_id, title, ip_address).await
    }

    /// 更新资源内容
    pub async fn update_resource_content(
        pool: &PgPool,
        user: &CurrentUser,
        storage: &Arc<dyn StorageBackend>,
        config: &Config,
        resource_id: Uuid,
        content: String,
    ) -> Result<UpdateResourceContentResponse, ResourceError> {
        modify::update_resource_content(pool, user, storage, config, resource_id, content).await
    }

    /// 获取资源原始内容
    pub async fn get_resource_content_raw(
        pool: &PgPool,
        storage: &Arc<dyn StorageBackend>,
        config: &Config,
        user: &CurrentUser,
        resource_id: Uuid,
    ) -> Result<String, ResourceError> {
        file_access::get_resource_content_raw(pool, storage, config, user, resource_id).await
    }

    /// 获取热门资源列表
    pub async fn get_hot_resources(
        pool: &PgPool,
        limit: i32,
    ) -> Result<Vec<HotResourceItem>, ResourceError> {
        query::get_hot_resources(pool, limit).await
    }

    /// 获取资源总数
    pub async fn get_resource_count(pool: &PgPool) -> Result<i64, ResourceError> {
        query::get_resource_count(pool).await
    }

    /// 更新资源关联信息
    pub async fn update_resource_relations(
        pool: &PgPool,
        resource_id: Uuid,
        teacher_sns: Vec<i64>,
        course_sns: Vec<i64>,
        related_resource_ids: Vec<Uuid>,
    ) -> Result<(), ResourceError> {
        modify::update_resource_relations(
            pool,
            resource_id,
            teacher_sns,
            course_sns,
            related_resource_ids,
        )
        .await
    }

    /// 搜索可关联的资源
    pub async fn search_resources_for_relation(
        pool: &PgPool,
        query: &str,
        exclude_id: Option<Uuid>,
        limit: i32,
    ) -> Result<Vec<RelatedResourceInfo>, ResourceError> {
        relation::search_resources_for_relation(pool, query, exclude_id, limit).await
    }

    /// 获取资源的关联资源列表
    pub async fn get_related_resources(
        pool: &PgPool,
        resource_id: Uuid,
    ) -> Result<Vec<RelatedResourceInfo>, ResourceError> {
        relation::get_related_resources(pool, resource_id).await
    }

    /// 更新资源描述
    pub async fn update_resource_description(
        pool: &PgPool,
        user: &CurrentUser,
        resource_id: Uuid,
        description: Option<String>,
    ) -> Result<(), ResourceError> {
        modify::update_resource_description(pool, user, resource_id, description).await
    }

    /// 根据文件哈希查询资源列表
    pub async fn find_by_file_hash(
        pool: &PgPool,
        file_hash: &str,
    ) -> Result<Vec<ResourceListItem>, ResourceError> {
        query::find_by_file_hash(pool, file_hash).await
    }
}
