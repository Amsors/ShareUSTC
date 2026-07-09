//! 资源文件访问相关功能

use crate::models::{resource::Resource, CurrentUser};
use crate::services::{storage_service::StorageBackend, ResourceError};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

/// 增加下载次数
pub async fn increment_downloads(pool: &PgPool, resource_id: Uuid) -> Result<(), ResourceError> {
    sqlx::query("UPDATE resource_stats SET downloads = downloads + 1 WHERE resource_id = $1")
        .bind(resource_id)
        .execute(pool)
        .await
        .map_err(|e| ResourceError::DatabaseError(e.to_string()))?;

    Ok(())
}

/// 增加访问次数
pub async fn increment_views(pool: &PgPool, resource_id: Uuid) -> Result<(), ResourceError> {
    sqlx::query("UPDATE resource_stats SET views = views + 1 WHERE resource_id = $1")
        .bind(resource_id)
        .execute(pool)
        .await
        .map_err(|e| ResourceError::DatabaseError(e.to_string()))?;

    Ok(())
}

/// 获取资源文件路径（检查审核状态和权限，用于下载）
/// 返回：(file_path, resource_type, title, storage_type)
/// 只有管理员或上传者可以访问未审核的资源，其他情况（包括游客）只能访问已通过审核的资源
pub async fn get_resource_file_path(
    pool: &PgPool,
    resource_id: Uuid,
    user: Option<&CurrentUser>,
) -> Result<(String, String, String, Option<String>), ResourceError> {
    // 获取资源信息，包括审核状态和上传者
    let row: (String, String, String, Option<String>, String, Uuid) = sqlx::query_as(
        "SELECT file_path, resource_type, title, storage_type, audit_status, uploader_id FROM resources WHERE id = $1"
    )
    .bind(resource_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| ResourceError::DatabaseError(e.to_string()))?
    .ok_or_else(|| ResourceError::NotFound(format!("资源 {} 不存在", resource_id)))?;

    let audit_status = row.4;
    let uploader_id = row.5;

    // 检查权限：只有管理员或上传者可以访问未审核的资源
    let is_admin = user
        .map(|u| matches!(u.role, crate::models::UserRole::Admin))
        .unwrap_or(false);
    let is_uploader = user.map(|u| u.id == uploader_id).unwrap_or(false);

    if audit_status != "approved" && !is_admin && !is_uploader {
        return Err(ResourceError::Unauthorized(
            "该资源尚未通过审核，无法下载".to_string(),
        ));
    }

    Ok((row.0, row.1, row.2, row.3))
}

/// 获取资源文件路径（检查审核状态和权限，用于预览）
/// 返回：(file_path, resource_type, storage_type, updated_at)
/// 只有管理员或上传者可以访问未审核的资源，其他情况（包括游客）只能访问已通过审核的资源
pub async fn get_resource_file_path_for_preview(
    pool: &PgPool,
    resource_id: Uuid,
    user: Option<&CurrentUser>,
) -> Result<(String, String, Option<String>, chrono::NaiveDateTime), ResourceError> {
    // 获取资源信息，包括审核状态和上传者
    let row: (String, String, Option<String>, chrono::NaiveDateTime, String, Uuid) =
        sqlx::query_as("SELECT file_path, resource_type, storage_type, updated_at, audit_status, uploader_id FROM resources WHERE id = $1")
            .bind(resource_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| ResourceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ResourceError::NotFound(format!("资源 {} 不存在", resource_id)))?;

    let audit_status = row.4;
    let uploader_id = row.5;

    // 检查权限：只有管理员或上传者可以访问未审核的资源
    let is_admin = user
        .map(|u| matches!(u.role, crate::models::UserRole::Admin))
        .unwrap_or(false);
    let is_uploader = user.map(|u| u.id == uploader_id).unwrap_or(false);

    if audit_status != "approved" && !is_admin && !is_uploader {
        return Err(ResourceError::Unauthorized(
            "该资源尚未通过审核，无法预览".to_string(),
        ));
    }

    Ok((row.0, row.1, row.2, row.3))
}

/// 记录下载日志
/// 将下载记录写入数据库，用于统计和审计
pub async fn record_download(
    pool: &PgPool,
    resource_id: Uuid,
    user_id: Option<Uuid>,
    ip_address: &str,
) -> Result<(), ResourceError> {
    sqlx::query(
        "INSERT INTO download_logs (resource_id, user_id, ip_address) VALUES ($1, $2, $3::inet)",
    )
    .bind(resource_id)
    .bind(user_id)
    .bind(ip_address)
    .execute(pool)
    .await
    .map_err(|e| {
        log::warn!("记录下载日志失败: {}", e);
        ResourceError::DatabaseError(e.to_string())
    })?;

    Ok(())
}

/// 记录一次下载事件（服务级编排）
///
/// 包含：递增下载计数、写入下载日志、记录审计日志。
/// 三个动作均为尽力而为（best-effort），任一失败仅记录日志，不影响下载响应。
/// 由 API 层在完成文件响应/重定向时调用（IP 提取等 Web 层职责留在 API 层）。
pub async fn record_download_event(
    pool: &PgPool,
    resource_id: Uuid,
    user_id: Option<Uuid>,
    title: &str,
    ip_address: &str,
) {
    use crate::services::AuditLogService;

    let _ = increment_downloads(pool, resource_id).await;
    let _ = record_download(pool, resource_id, user_id, ip_address).await;
    let _ =
        AuditLogService::log_download_resource(pool, user_id, resource_id, title, Some(ip_address))
            .await;
}

/// 获取资源原始内容（用于编辑）
pub async fn get_resource_content_raw(
    pool: &PgPool,
    storage: &Arc<dyn StorageBackend>,
    user: &CurrentUser,
    resource_id: Uuid,
) -> Result<String, ResourceError> {
    // 获取资源信息
    let resource: Resource = sqlx::query_as::<_, Resource>("SELECT * FROM resources WHERE id = $1")
        .bind(resource_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| ResourceError::DatabaseError(e.to_string()))?
        .ok_or_else(|| ResourceError::NotFound(format!("资源 {} 不存在", resource_id)))?;

    // 检查权限（上传者或管理员）
    if resource.uploader_id != user.id && user.role != crate::models::UserRole::Admin {
        return Err(ResourceError::Unauthorized(
            "没有权限查看此资源的原始内容".to_string(),
        ));
    }

    // 根据资源实际的存储类型选择正确的存储后端读取文件
    let is_oss = resource.storage_type.as_deref() == Some("oss");
    let content_bytes = if is_oss {
        // OSS 存储
        if storage.backend_type() == crate::services::StorageBackendType::Oss {
            storage.read_file(&resource.file_path).await?
        } else {
            // 当前是 local 模式，但需要读取 OSS 文件
            let config = crate::config::Config::from_env();
            match crate::services::create_storage_backend(&config) {
                Ok(oss_storage)
                    if oss_storage.backend_type() == crate::services::StorageBackendType::Oss =>
                {
                    oss_storage.read_file(&resource.file_path).await?
                }
                _ => return Err(ResourceError::FileError("无法读取 OSS 资源".to_string())),
            }
        }
    } else {
        // 本地存储
        if storage.backend_type() == crate::services::StorageBackendType::Local {
            storage.read_file(&resource.file_path).await?
        } else {
            // 当前是 OSS 模式，但需要读取本地文件
            let config = crate::config::Config::from_env();
            match crate::services::create_local_storage(&config) {
                Ok(local_storage) => local_storage.read_file(&resource.file_path).await?,
                Err(e) => return Err(ResourceError::FileError(format!("无法访问本地存储: {}", e))),
            }
        }
    };

    let content = String::from_utf8(content_bytes)
        .map_err(|e| ResourceError::FileError(format!("文件内容不是有效 UTF-8: {}", e)))?;

    Ok(content)
}
