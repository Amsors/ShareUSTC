//! 资源上传相关功能

use crate::models::{resource::*, CurrentUser};
use crate::services::resource_service::utils::{
    compute_hash_from_storage_with_retry, infer_resource_type,
};
use crate::services::{
    ai_service::AiService,
    file_service::FileService,
    storage_service::{StorageBackend, StorageFileMetadata},
    ResourceError,
};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

/// 从 OSS 回调创建资源
pub async fn create_resource_from_oss_callback(
    pool: &PgPool,
    user: &CurrentUser,
    storage: &Arc<dyn StorageBackend>,
    request: UploadResourceRequest,
    oss_key: &str,
    metadata: StorageFileMetadata,
) -> Result<UploadResourceResponse, ResourceError> {
    request.validate().map_err(ResourceError::ValidationError)?;

    let file_size = metadata
        .content_length
        .ok_or_else(|| ResourceError::ValidationError("无法获取文件大小".to_string()))?;
    if file_size == 0 {
        return Err(ResourceError::ValidationError("文件不能为空".to_string()));
    }
    if file_size > FileService::MAX_FILE_SIZE as u64 {
        return Err(ResourceError::ValidationError(format!(
            "文件大小超过限制。最大允许 100MB，当前 {:.2}MB",
            file_size as f64 / 1024.0 / 1024.0
        )));
    }

    let object_name = oss_key.rsplit('/').next().unwrap_or(oss_key);
    let resource_type = infer_resource_type(object_name, metadata.content_type.as_deref())
        .ok_or_else(|| {
            ResourceError::ValidationError(format!(
                "不支持的文件类型。支持的类型: {}",
                ResourceType::supported_extensions().join(", ")
            ))
        })?;
    if resource_type == ResourceType::Other {
        return Err(ResourceError::ValidationError(format!(
            "不支持的文件类型。支持的类型: {}",
            ResourceType::supported_extensions().join(", ")
        )));
    }

    let ai_result = AiService::audit_resource(&request.title, request.description.as_deref(), None)
        .await
        .map_err(|e| ResourceError::AiError(e.to_string()))?;
    let audit_status = if ai_result.passed {
        AuditStatus::Approved
    } else {
        AuditStatus::Pending
    };

    let resource_id = Uuid::new_v4();
    let tags_json = request
        .tags
        .as_ref()
        .map(|tags| serde_json::to_value(tags).unwrap_or(serde_json::Value::Array(vec![])));
    let storage_type = storage.backend_type().as_str().to_string();

    let mut tx = pool.begin().await.map_err(ResourceError::Database)?;

    let resource: Resource = match sqlx::query_as::<_, Resource>(
        r#"
        INSERT INTO resources (
            id, title, author_id, uploader_id, course_name,
            resource_type, category, tags, file_path, source_file_path,
            file_hash, file_size, content_accuracy, audit_status, ai_reject_reason, storage_type, description
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
        RETURNING *
        "#,
    )
    .bind(resource_id)
    .bind(&request.title)
    .bind(None::<Uuid>)
    .bind(user.id)
    .bind(request.course_name.clone())
    .bind(resource_type.to_string())
    .bind(request.category.to_string())
    .bind(tags_json)
    .bind(oss_key)
    .bind(None::<String>)
    .bind(None::<String>)
    .bind(file_size as i64)
    .bind(ai_result.accuracy_score)
    .bind(audit_status.to_string())
    .bind(if ai_result.passed {
        None
    } else {
        ai_result.reason.as_deref()
    })
    .bind(&storage_type)
    .bind(request.description.as_ref())
    .fetch_one(&mut *tx)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            if let Err(cleanup_err) = storage.delete_file(oss_key).await {
                log::warn!(
                    "[Resource] OSS 回调入库失败后清理文件失败 | key={}, error={}",
                    oss_key,
                    cleanup_err
                );
            }
            return Err(ResourceError::Database(e));
        }
    };

    if let Err(e) = sqlx::query(
        "INSERT INTO resource_stats (resource_id, views, downloads, likes, rating_count) VALUES ($1, 0, 0, 0, 0)",
    )
    .bind(resource_id)
    .execute(&mut *tx)
    .await
    {
        if let Err(rollback_err) = tx.rollback().await {
            log::warn!("[Resource] 资源统计初始化失败后回滚失败: {}", rollback_err);
        }
        if let Err(cleanup_err) = storage.delete_file(oss_key).await {
            log::warn!(
                "[Resource] 资源统计初始化失败后清理文件失败 | key={}, error={}",
                oss_key,
                cleanup_err
            );
        }
        return Err(ResourceError::Database(e));
    }

    // 插入教师关联
    if let Some(teacher_sns) = &request.teacher_sns {
        for teacher_sn in teacher_sns {
            if let Err(e) = sqlx::query(
                "INSERT INTO resource_teachers (resource_id, teacher_sn) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(resource_id)
            .bind(teacher_sn)
            .execute(&mut *tx)
            .await
            {
                log::warn!(
                    "[Resource] 回调插入教师关联失败 | resource_id={}, teacher_sn={}, error={}",
                    resource_id,
                    teacher_sn,
                    e
                );
            }
        }
    }

    // 插入课程关联
    if let Some(course_sns) = &request.course_sns {
        for course_sn in course_sns {
            if let Err(e) = sqlx::query(
                "INSERT INTO resource_courses (resource_id, course_sn) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(resource_id)
            .bind(course_sn)
            .execute(&mut *tx)
            .await
            {
                log::warn!(
                    "[Resource] 回调插入课程关联失败 | resource_id={}, course_sn={}, error={}",
                    resource_id,
                    course_sn,
                    e
                );
            }
        }
    }

    // 插入资源关联记录（OSS回调）
    if let Some(related_resource_ids) = &request.related_resource_ids {
        for related_id in related_resource_ids {
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
                log::warn!("[Resource] 回调插入资源关联失败 | source_id={}, target_id={}, error={}", resource_id, related_id, e);
            }
        }
        log::debug!(
            "[Resource] 回调资源关联插入完成 | resource_id={}, count={}",
            resource_id,
            related_resource_ids.len()
        );
    }

    // 从 OSS 下载文件并计算哈希（带重试机制）
    let file_hash = compute_hash_from_storage_with_retry(storage, oss_key, resource_id).await;
    match file_hash {
        Ok(hash) => {
            // 更新数据库中的 file_hash
            if let Err(e) = sqlx::query("UPDATE resources SET file_hash = $1 WHERE id = $2")
                .bind(&hash)
                .bind(resource_id)
                .execute(&mut *tx)
                .await
            {
                log::warn!(
                    "[Resource] OSS 回调更新文件哈希失败 | resource_id={}, error={}",
                    resource_id,
                    e
                );
                // 哈希更新失败不影响主流程，继续提交事务
            } else {
                log::info!(
                    "[Resource] OSS 回调文件哈希计算成功 | resource_id={}, hash={}",
                    resource_id,
                    &hash[..16.min(hash.len())]
                );
            }
        }
        Err(e) => {
            log::warn!(
                "[Resource] OSS 回调计算文件哈希失败 | resource_id={}, error={}",
                resource_id,
                e
            );
            // 哈希计算失败不影响主流程，后续定时任务会重新计算
        }
    }

    if let Err(e) = tx.commit().await {
        if let Err(cleanup_err) = storage.delete_file(oss_key).await {
            log::warn!(
                "[Resource] 回调提交事务失败后清理文件失败 | key={}, error={}",
                oss_key,
                cleanup_err
            );
        }
        return Err(ResourceError::Database(e));
    }

    Ok(UploadResourceResponse {
        id: resource.id,
        title: resource.title,
        resource_type: resource.resource_type,
        audit_status: resource.audit_status,
        ai_message: if ai_result.passed {
            Some("AI 审核通过".to_string())
        } else {
            Some("AI 审核未通过，等待人工审核".to_string())
        },
        created_at: resource.created_at,
    })
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
    // 验证请求
    request.validate().map_err(ResourceError::ValidationError)?;

    // 验证并确定资源类型
    let resource_type = FileService::validate_resource_file(file_name, &file_data, mime_type)?;

    // AI 审核
    let ai_result = AiService::audit_resource(
        &request.title,
        request.description.as_deref(),
        Some(&file_data),
    )
    .await
    .map_err(|e| ResourceError::AiError(e.to_string()))?;

    // 生成资源 ID
    let resource_id = Uuid::new_v4();
    let resource_type_str = resource_type.to_string();
    let extension = FileService::get_extension_by_type(&resource_type_str);
    let file_key = format!("resources/{}.{}.", resource_id, extension);
    let file_hash = FileService::calculate_hash(&file_data);
    let file_size = file_data.len() as i64;
    let storage_type = storage.backend_type().as_str().to_string();

    // 保存文件（统一走存储抽象）
    let file_path = storage.save_file(&file_key, file_data, mime_type).await?;

    // 确定审核状态
    let audit_status = if ai_result.passed {
        AuditStatus::Approved
    } else {
        AuditStatus::Pending
    };

    // 转换标签为 JSON
    let tags_json = request
        .tags
        .map(|tags| serde_json::to_value(tags).unwrap_or(serde_json::Value::Array(vec![])));

    // 开启事务
    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => {
            log::error!(
                "[Resource] 开启事务失败 | resource_id={}, error={}",
                resource_id,
                e
            );
            // 开启事务失败时清理已保存的文件
            if let Err(cleanup_err) = storage.delete_file(&file_path).await {
                log::error!(
                    "[Resource] 开启事务失败后清理文件出错 | path={}, error={}",
                    file_path,
                    cleanup_err
                );
            }
            return Err(ResourceError::Database(e));
        }
    };

    // 插入资源记录
    log::debug!(
        "[Resource] 准备插入资源记录 | title={}, resource_type={}",
        request.title,
        resource_type
    );

    let resource: Resource = match sqlx::query_as::<_, Resource>(
        r#"
        INSERT INTO resources (
            id, title, author_id, uploader_id, course_name,
            resource_type, category, tags, file_path, source_file_path,
            file_hash, file_size, content_accuracy, audit_status, ai_reject_reason, storage_type, description
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
        RETURNING *
        "#,
    )
    .bind(resource_id)
    .bind(&request.title)
    .bind(None::<Uuid>) // author_id 为空，等待申领
    .bind(user.id)
    .bind(request.course_name)
    .bind(resource_type.to_string())
    .bind(request.category.to_string())
    .bind(tags_json)
    .bind(&file_path)
    .bind(None::<String>) // source_file_path 暂不处理源文件
    .bind(&file_hash)
    .bind(file_size)
    .bind(ai_result.accuracy_score)
    .bind(audit_status.to_string())
    .bind(if ai_result.passed {
        None
    } else {
        ai_result.reason.as_deref()
    })
    .bind(&storage_type)
    .bind(request.description.as_ref())
    .fetch_one(&mut *tx)
    .await
    {
        Ok(r) => r,
        Err(e) => {
            log::error!(
                "[Resource] 数据库插入失败 | resource_id={}, error={}",
                resource_id,
                e
            );
            // 数据库插入失败时清理已保存的文件
            if let Err(cleanup_err) = storage.delete_file(&file_path).await {
                log::error!(
                    "[Resource] 数据库插入失败后清理文件出错 | path={}, error={}",
                    file_path,
                    cleanup_err
                );
            }
            return Err(ResourceError::Database(e));
        }
    };

    log::debug!("[Resource] 资源记录插入成功 | resource_id={}", resource.id);

    // 创建资源统计记录
    if let Err(e) = sqlx::query(
        "INSERT INTO resource_stats (resource_id, views, downloads, likes, rating_count) VALUES ($1, 0, 0, 0, 0)"
    )
    .bind(resource_id)
    .execute(&mut *tx)
    .await {
        log::error!("[Resource] 创建统计记录失败 | resource_id={}, error={}", resource_id, e);
        // 统计记录创建失败时，回滚事务并清理文件
        if let Err(rollback_err) = tx.rollback().await {
            log::error!("[Resource] 回滚事务失败 | error={}", rollback_err);
        }
        if let Err(cleanup_err) = storage.delete_file(&file_path).await {
            log::error!("[Resource] 创建统计记录失败后清理文件出错 | path={}, error={}", file_path, cleanup_err);
        }
        return Err(ResourceError::Database(e));
    }

    // 插入教师关联记录
    if let Some(teacher_sns) = &request.teacher_sns {
        for teacher_sn in teacher_sns {
            if let Err(e) = sqlx::query(
                "INSERT INTO resource_teachers (resource_id, teacher_sn) VALUES ($1, $2) ON CONFLICT DO NOTHING"
            )
            .bind(resource_id)
            .bind(teacher_sn)
            .execute(&mut *tx)
            .await {
                log::warn!("[Resource] 插入教师关联失败 | resource_id={}, teacher_sn={}, error={}", resource_id, teacher_sn, e);
                // 非关键错误，继续处理
            }
        }
        log::debug!(
            "[Resource] 教师关联插入完成 | resource_id={}, count={}",
            resource_id,
            teacher_sns.len()
        );
    }

    // 插入课程关联记录
    if let Some(course_sns) = &request.course_sns {
        for course_sn in course_sns {
            if let Err(e) = sqlx::query(
                "INSERT INTO resource_courses (resource_id, course_sn) VALUES ($1, $2) ON CONFLICT DO NOTHING"
            )
            .bind(resource_id)
            .bind(course_sn)
            .execute(&mut *tx)
            .await {
                log::warn!("[Resource] 插入课程关联失败 | resource_id={}, course_sn={}, error={}", resource_id, course_sn, e);
                // 非关键错误，继续处理
            }
        }
        log::debug!(
            "[Resource] 课程关联插入完成 | resource_id={}, count={}",
            resource_id,
            course_sns.len()
        );
    }

    // 插入资源关联记录
    if let Some(related_resource_ids) = &request.related_resource_ids {
        for related_id in related_resource_ids {
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
                log::warn!("[Resource] 插入资源关联失败 | source_id={}, target_id={}, error={}", resource_id, related_id, e);
                // 非关键错误，继续处理
            }
        }
        log::debug!(
            "[Resource] 资源关联插入完成 | resource_id={}, count={}",
            resource_id,
            related_resource_ids.len()
        );
    }

    // 提交事务
    if let Err(e) = tx.commit().await {
        log::error!(
            "[Resource] 提交事务失败 | resource_id={}, error={}",
            resource_id,
            e
        );

        // 事务提交失败时尝试清理已保存的文件，避免产生孤立文件
        if let Err(cleanup_err) = storage.delete_file(&file_path).await {
            log::error!(
                "[Resource] 事务提交失败后清理文件出错 | path={}, error={}",
                file_path,
                cleanup_err
            );
        }

        return Err(ResourceError::Database(e));
    }
    Ok(UploadResourceResponse {
        id: resource.id,
        title: resource.title,
        resource_type: resource.resource_type,
        audit_status: resource.audit_status,
        ai_message: if ai_result.passed {
            Some("AI 审核通过".to_string())
        } else {
            Some("AI 审核未通过，等待人工审核".to_string())
        },
        created_at: resource.created_at,
    })
}
