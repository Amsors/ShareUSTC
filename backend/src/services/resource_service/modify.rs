//! 资源修改相关功能

use crate::config::Config;
use crate::models::{resource::*, CurrentUser, UpdateResourceContentResponse};
use crate::services::resource_service::utils::verify_oss_write_with_retry;
use crate::services::{
    ai_service::AiService, file_service::FileService, storage_service::StorageBackend,
    ResourceError,
};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

/// 删除资源
/// 返回被删除资源的标题
pub async fn delete_resource(
    pool: &PgPool,
    user: &CurrentUser,
    storage: &Arc<dyn StorageBackend>,
    resource_id: Uuid,
) -> Result<String, ResourceError> {
    // 获取资源信息
    let resource: Resource = sqlx::query_as::<_, Resource>("SELECT * FROM resources WHERE id = $1")
        .bind(resource_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| ResourceError::NotFound(format!("资源 {} 不存在", resource_id)))?;

    // 检查权限（上传者或管理员）
    if resource.uploader_id != user.id && user.role != crate::models::UserRole::Admin {
        return Err(ResourceError::Unauthorized(
            "没有权限删除此资源".to_string(),
        ));
    }

    // 删除文件
    if let Err(e) = storage.delete_file(&resource.file_path).await {
        log::warn!(
            "[Resource] 删除资源文件失败 | resource_id={}, path={}, error={}",
            resource_id,
            resource.file_path,
            e
        );
        // 继续执行，即使文件删除失败也要删除数据库记录
    }

    // 删除源文件（如果存在）
    if let Some(source_path) = &resource.source_file_path {
        if let Err(e) = storage.delete_file(source_path).await {
            log::warn!(
                "[Resource] 删除源文件失败 | resource_id={}, path={}, error={}",
                resource_id,
                source_path,
                e
            );
        }
    }

    // 保存资源标题用于返回
    let title = resource.title.clone();

    // 删除数据库记录
    sqlx::query("DELETE FROM resources WHERE id = $1")
        .bind(resource_id)
        .execute(pool)
        .await?;

    Ok(title)
}

/// 更新资源内容（用于Markdown在线编辑）
/// 更新后会进行AI审核，并更新 file_hash、file_size、updated_at 字段
pub async fn update_resource_content(
    pool: &PgPool,
    user: &CurrentUser,
    storage: &Arc<dyn StorageBackend>,
    config: &Config,
    resource_id: Uuid,
    content: String,
) -> Result<UpdateResourceContentResponse, ResourceError> {
    // 验证内容长度
    if content.len() > 10 * 1024 * 1024 {
        return Err(ResourceError::ValidationError(
            "内容大小超过10MB限制".to_string(),
        ));
    }

    // 获取资源信息
    let resource: Resource = sqlx::query_as::<_, Resource>("SELECT * FROM resources WHERE id = $1")
        .bind(resource_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| ResourceError::NotFound(format!("资源 {} 不存在", resource_id)))?;

    // 检查权限（上传者或管理员）
    if resource.uploader_id != user.id && user.role != crate::models::UserRole::Admin {
        return Err(ResourceError::Unauthorized(
            "没有权限编辑此资源".to_string(),
        ));
    }

    // 检查资源类型是否为web_markdown
    if resource.resource_type != "web_markdown" {
        return Err(ResourceError::ValidationError(
            "只有Markdown类型资源可以在线编辑".to_string(),
        ));
    }

    // AI 审核更新后的内容
    let ai_result =
        AiService::audit_resource(&resource.title, Some(&content), Some(content.as_bytes()))
            .await
            .map_err(|e| ResourceError::AiError(e.to_string()))?;

    // 保存旧的hash用于乐观锁检查
    let old_hash = resource.file_hash.clone();

    // 根据资源实际的存储类型选择正确的存储后端写入文件
    let is_oss = resource.storage_type.as_deref() == Some("oss");
    let content_bytes = content.as_bytes();
    let expected_hash = FileService::calculate_hash(content_bytes);

    if is_oss {
        // OSS 存储
        if storage.backend_type() == crate::services::StorageBackendType::Oss {
            storage
                .write_file(
                    &resource.file_path,
                    content_bytes.to_vec(),
                    Some("text/markdown"),
                )
                .await?;
        } else {
            // 当前是 local 模式，但需要写入 OSS 文件（使用注入的配置）
            match crate::services::create_storage_backend(config) {
                Ok(oss_storage)
                    if oss_storage.backend_type() == crate::services::StorageBackendType::Oss =>
                {
                    oss_storage
                        .write_file(
                            &resource.file_path,
                            content_bytes.to_vec(),
                            Some("text/markdown"),
                        )
                        .await?;
                }
                _ => return Err(ResourceError::FileError("无法写入 OSS 资源".to_string())),
            }
        }

        // OSS写入后验证：读取文件并验证内容一致性（处理最终一致性）
        // 最多重试5次，使用指数退避
        let verification_result =
            verify_oss_write_with_retry(storage, &resource.file_path, content_bytes, resource_id)
                .await;

        match verification_result {
            Ok(verified_hash) => {
                log::info!(
                    "[Resource] OSS 写入验证成功 | resource_id={}, hash={}",
                    resource_id,
                    &verified_hash[..16.min(verified_hash.len())]
                );
            }
            Err(e) => {
                log::error!(
                    "[Resource] OSS 写入验证失败，内容可能不一致 | resource_id={}, error={}",
                    resource_id,
                    e
                );
                return Err(ResourceError::FileError(format!("文件写入验证失败: {}", e)));
            }
        }
    } else {
        // 本地存储
        if storage.backend_type() == crate::services::StorageBackendType::Local {
            storage
                .write_file(
                    &resource.file_path,
                    content_bytes.to_vec(),
                    Some("text/markdown"),
                )
                .await?;
        } else {
            // 当前是 OSS 模式，但需要写入本地文件（使用注入的配置）
            match crate::services::create_local_storage(config) {
                Ok(local_storage) => {
                    local_storage
                        .write_file(
                            &resource.file_path,
                            content_bytes.to_vec(),
                            Some("text/markdown"),
                        )
                        .await?;
                }
                Err(e) => return Err(ResourceError::FileError(format!("无法访问本地存储: {}", e))),
            }
        }
    }

    // 计算新的文件哈希和大小
    // 对于 OSS 存储，使用验证步骤计算的hash（如果验证成功）
    // 对于本地存储，直接使用内存中的 content 计算（更高效）
    let file_hash = if is_oss {
        // OSS存储：直接使用预期hash（已通过验证确认写入正确）
        expected_hash
    } else {
        // 本地存储：直接使用内存中的 content 计算
        FileService::calculate_hash(content_bytes)
    };
    let file_size = content_bytes.len() as i64;

    // 确定审核状态
    let audit_status = if ai_result.passed {
        AuditStatus::Approved
    } else {
        AuditStatus::Pending
    };

    // 更新数据库中的 updated_at、file_hash、file_size、audit_status、content_accuracy
    // 使用乐观锁：只有当file_hash等于old_hash时才更新（防止并发修改）
    let update_result = sqlx::query_scalar::<_, chrono::NaiveDateTime>(
        r#"
        UPDATE resources
        SET
            updated_at = CURRENT_TIMESTAMP,
            file_hash = $1,
            file_size = $2,
            audit_status = $3,
            content_accuracy = $4,
            ai_reject_reason = $5
        WHERE id = $6 AND (file_hash = $7 OR file_hash IS NULL)
        RETURNING updated_at
        "#,
    )
    .bind(&file_hash)
    .bind(file_size)
    .bind(audit_status.to_string())
    .bind(ai_result.accuracy_score)
    .bind(if ai_result.passed {
        None
    } else {
        ai_result.reason
    })
    .bind(resource_id)
    .bind(old_hash)
    .fetch_optional(pool)
    .await?;

    match update_result {
        Some(updated_at) => Ok(UpdateResourceContentResponse {
            id: resource_id,
            updated_at,
        }),
        None => {
            // 乐观锁失败：资源在编辑期间被其他进程修改
            log::warn!(
                "[Resource] 乐观锁冲突，资源在编辑期间被修改 | resource_id={}",
                resource_id
            );
            Err(ResourceError::Conflict(
                "资源在您编辑期间已被修改，请刷新后重试".to_string(),
            ))
        }
    }
}

/// 更新资源描述
/// 用户只能更新自己上传的资源，管理员可以更新所有资源
pub async fn update_resource_description(
    pool: &PgPool,
    user: &CurrentUser,
    resource_id: Uuid,
    description: Option<String>,
) -> Result<(), ResourceError> {
    // 验证描述长度
    if let Some(ref desc) = description {
        if desc.len() > 10 * 1024 {
            return Err(ResourceError::ValidationError(
                "资源描述不能超过10KB".to_string(),
            ));
        }
    }

    // 获取资源信息
    let resource: Resource = sqlx::query_as::<_, Resource>("SELECT * FROM resources WHERE id = $1")
        .bind(resource_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| ResourceError::NotFound(format!("资源 {} 不存在", resource_id)))?;

    // 检查权限（上传者或管理员）
    if resource.uploader_id != user.id && user.role != crate::models::UserRole::Admin {
        return Err(ResourceError::Unauthorized(
            "没有权限编辑此资源的描述".to_string(),
        ));
    }

    // 更新描述
    sqlx::query(
        "UPDATE resources SET description = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2",
    )
    .bind(description)
    .bind(resource_id)
    .execute(pool)
    .await?;

    log::info!(
        "[Resource] 资源描述更新成功 | resource_id={}, user_id={}",
        resource_id,
        user.id
    );

    Ok(())
}

/// 更新资源关联信息
/// 会完全替换原有的关联信息
pub async fn update_resource_relations(
    pool: &PgPool,
    resource_id: Uuid,
    teacher_sns: Vec<i64>,
    course_sns: Vec<i64>,
    related_resource_ids: Vec<Uuid>,
) -> Result<(), ResourceError> {
    // 检查资源是否存在
    let resource_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM resources WHERE id = $1)")
            .bind(resource_id)
            .fetch_one(pool)
            .await?;

    if !resource_exists {
        return Err(ResourceError::NotFound(format!(
            "资源 {} 不存在",
            resource_id
        )));
    }

    // 开启事务
    let mut tx = pool.begin().await.map_err(ResourceError::Database)?;

    // 1. 更新教师关联 - 先删除旧的，再插入新的
    if let Err(e) = sqlx::query("DELETE FROM resource_teachers WHERE resource_id = $1")
        .bind(resource_id)
        .execute(&mut *tx)
        .await
    {
        let _ = tx.rollback().await;
        return Err(ResourceError::Database(e));
    }

    for teacher_sn in &teacher_sns {
        if let Err(e) = sqlx::query(
            "INSERT INTO resource_teachers (resource_id, teacher_sn) VALUES ($1, $2) ON CONFLICT DO NOTHING"
        )
        .bind(resource_id)
        .bind(teacher_sn)
        .execute(&mut *tx)
        .await {
            log::warn!(
                "[Resource] 插入教师关联失败 | resource_id={}, teacher_sn={}, error={}",
                resource_id, teacher_sn, e
            );
        }
    }

    // 2. 更新课程关联 - 先删除旧的，再插入新的
    if let Err(e) = sqlx::query("DELETE FROM resource_courses WHERE resource_id = $1")
        .bind(resource_id)
        .execute(&mut *tx)
        .await
    {
        let _ = tx.rollback().await;
        return Err(ResourceError::Database(e));
    }

    for course_sn in &course_sns {
        if let Err(e) = sqlx::query(
            "INSERT INTO resource_courses (resource_id, course_sn) VALUES ($1, $2) ON CONFLICT DO NOTHING"
        )
        .bind(resource_id)
        .bind(course_sn)
        .execute(&mut *tx)
        .await {
            log::warn!(
                "[Resource] 插入课程关联失败 | resource_id={}, course_sn={}, error={}",
                resource_id, course_sn, e
            );
        }
    }

    // 3. 更新资源关联 - 先删除旧的，再插入新的
    if let Err(e) = sqlx::query("DELETE FROM resource_relations WHERE source_resource_id = $1")
        .bind(resource_id)
        .execute(&mut *tx)
        .await
    {
        let _ = tx.rollback().await;
        return Err(ResourceError::Database(e));
    }

    for related_id in &related_resource_ids {
        // 跳过自关联
        if *related_id == resource_id {
            log::warn!("[Resource] 跳过自关联 | resource_id={}", resource_id);
            continue;
        }

        if let Err(e) = sqlx::query(
            "INSERT INTO resource_relations (source_resource_id, target_resource_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
        )
        .bind(resource_id)
        .bind(related_id)
        .execute(&mut *tx)
        .await {
            log::warn!(
                "[Resource] 插入资源关联失败 | source_id={}, target_id={}, error={}",
                resource_id, related_id, e
            );
        }
    }

    // 提交事务
    if let Err(e) = tx.commit().await {
        return Err(ResourceError::Database(e));
    }

    log::info!(
        "[Resource] 资源关联信息更新成功 | resource_id={}, teachers={}, courses={}, related_resources={}",
        resource_id,
        teacher_sns.len(),
        course_sns.len(),
        related_resource_ids.len()
    );

    Ok(())
}
