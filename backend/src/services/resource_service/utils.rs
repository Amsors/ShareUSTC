//! 资源服务工具函数

use crate::models::resource::ResourceType;
use crate::services::file_service::FileService;

use sqlx::QueryBuilder;
use std::sync::Arc;
use tokio::time::Duration;
use uuid::Uuid;

/// 根据文件名和 MIME 类型推断资源类型
pub fn infer_resource_type(file_name: &str, mime_type: Option<&str>) -> Option<ResourceType> {
    let extension = std::path::Path::new(file_name)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase());

    if let Some(ext) = extension {
        let resource_type = ResourceType::from_extension(&ext);
        if resource_type != ResourceType::Other {
            return Some(resource_type);
        }
    }

    mime_type.map(|mime| match mime {
        "application/pdf" => ResourceType::Pdf,
        "text/plain" => ResourceType::Txt,
        "text/markdown" => ResourceType::WebMarkdown,
        "image/jpeg" => ResourceType::Jpeg,
        "image/png" => ResourceType::Png,
        "application/zip" => ResourceType::Zip,
        "application/msword" => ResourceType::Doc,
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
            ResourceType::Docx
        }
        "application/vnd.ms-powerpoint" => ResourceType::Ppt,
        "application/vnd.openxmlformats-officedocument.presentationml.presentation" => {
            ResourceType::Pptx
        }
        _ => ResourceType::Other,
    })
}

/// 添加资源类型筛选条件到 QueryBuilder
pub fn add_resource_type_condition<'a>(
    builder: &mut QueryBuilder<'a, sqlx::Postgres>,
    resource_type: Option<&'a str>,
) {
    if let Some(resource_type) = resource_type {
        match resource_type {
            "ppt" => {
                builder.push(" AND (r.resource_type = 'ppt' OR r.resource_type = 'pptx')");
            }
            "image" => {
                builder.push(" AND (r.resource_type = 'jpeg' OR r.resource_type = 'jpg' OR r.resource_type = 'png')");
            }
            "doc" => {
                builder.push(" AND (r.resource_type = 'doc' OR r.resource_type = 'docx')");
            }
            _ => {
                builder.push(" AND r.resource_type = ");
                builder.push_bind(resource_type);
            }
        }
    }
}

/// 计算平均分辅助函数
pub fn calc_avg(total: Option<i32>, count: Option<i32>) -> Option<f64> {
    match (total, count) {
        (Some(t), Some(c)) if c > 0 => Some(t as f64 / c as f64),
        _ => None,
    }
}

/// 验证 OSS 写入操作（带指数退避重试）
///
/// 用于处理 OSS 最终一致性问题，确保写入的内容可以被正确读取
/// 返回读取到的文件hash（如果验证成功）
pub async fn verify_oss_write_with_retry(
    storage: &Arc<dyn crate::services::StorageBackend>,
    file_path: &str,
    expected_content: &[u8],
    resource_id: Uuid,
) -> Result<String, String> {
    const MAX_RETRIES: u32 = 5;
    const INITIAL_DELAY_MS: u64 = 200;
    const MAX_DELAY_MS: u64 = 5000;

    let expected_hash = FileService::calculate_hash(expected_content);
    let expected_size = expected_content.len();

    for attempt in 0..MAX_RETRIES {
        if attempt > 0 {
            // 指数退避
            let delay_ms = std::cmp::min(
                INITIAL_DELAY_MS * (1_u64 << attempt.saturating_sub(1)),
                MAX_DELAY_MS,
            );
            log::info!(
                "[Resource] OSS 写入验证重试 | resource_id={}, attempt={}/{}, delay={}ms",
                resource_id,
                attempt + 1,
                MAX_RETRIES,
                delay_ms
            );
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        }

        match storage.read_file(file_path).await {
            Ok(data) => {
                // 验证文件大小
                if data.len() != expected_size {
                    log::warn!(
                        "[Resource] OSS 写入验证失败：大小不一致 | resource_id={}, expected={}, got={}",
                        resource_id, expected_size, data.len()
                    );
                    continue;
                }

                // 验证文件内容hash
                let actual_hash = FileService::calculate_hash(&data);
                if actual_hash == expected_hash {
                    log::info!(
                        "[Resource] OSS 写入验证通过 | resource_id={}, attempt={}",
                        resource_id,
                        attempt + 1
                    );
                    return Ok(actual_hash);
                } else {
                    log::warn!(
                        "[Resource] OSS 写入验证失败：hash不一致 | resource_id={}, attempt={}",
                        resource_id,
                        attempt + 1
                    );
                    // hash不一致，继续重试（可能是读取到旧版本）
                }
            }
            Err(e) => {
                log::warn!(
                    "[Resource] OSS 写入验证读取失败 | resource_id={}, attempt={}/{}, error={}",
                    resource_id,
                    attempt + 1,
                    MAX_RETRIES,
                    e
                );
            }
        }
    }

    Err(format!(
        "OSS 写入验证失败：重试 {} 次后仍无法确认写入一致性",
        MAX_RETRIES
    ))
}

/// 从存储后端读取文件并计算哈希（带重试机制）
///
/// 用于 OSS 回调上传时计算文件哈希
pub async fn compute_hash_from_storage_with_retry(
    storage: &Arc<dyn crate::services::StorageBackend>,
    file_path: &str,
    resource_id: Uuid,
) -> Result<String, String> {
    const MAX_RETRIES: u32 = 3;
    const RETRY_DELAY_MS: u64 = 500;

    let mut last_error = String::new();

    for attempt in 0..MAX_RETRIES {
        if attempt > 0 {
            log::info!(
                "[Resource] 重试计算文件哈希 | resource_id={}, attempt={}/{}",
                resource_id,
                attempt + 1,
                MAX_RETRIES
            );
            tokio::time::sleep(Duration::from_millis(RETRY_DELAY_MS * attempt as u64)).await;
        }

        match storage.read_file(file_path).await {
            Ok(data) => {
                let hash = FileService::calculate_hash(&data);
                return Ok(hash);
            }
            Err(e) => {
                log::warn!(
                    "[Resource] 读取文件计算哈希失败 | resource_id={}, path={}, attempt={}/{}, error={}",
                    resource_id, file_path, attempt + 1, MAX_RETRIES, e
                );
                last_error = e.to_string();
            }
        }
    }

    Err(format!("重试 {} 次后仍然失败: {}", MAX_RETRIES, last_error))
}
