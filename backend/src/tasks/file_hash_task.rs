/// 文件哈希计算任务
///
/// 后台定时任务，用于：
/// 1. 启动时扫描数据库，为没有哈希记录的资源计算哈希
/// 2. 每24小时重新扫描一次
/// 3. 支持 OSS 和本地存储
/// 4. 支持分批处理和流式计算，避免内存压力
use std::sync::Arc;
use std::time::Duration;

use sqlx::PgPool;
use tokio::time::interval;
use uuid::Uuid;

use crate::config::Config;
use crate::services::{FileService, StorageBackend, StorageBackendType};

/// 批次大小（每次处理的数量）
const BATCH_SIZE: i64 = 50;
/// 告警阈值：当待处理数量超过此值时记录警告
const ALERT_THRESHOLD: i64 = 500;
/// 最大文件大小 (100MB)
const MAX_FILE_SIZE: i64 = 100 * 1024 * 1024;
/// 流式计算缓冲区大小 (64KB)
const STREAM_BUFFER_SIZE: usize = 64 * 1024;

/// 启动文件哈希计算任务
///
/// 在服务启动时调用，会：
/// 1. 立即执行一次全量扫描（分批处理直到全部完成）
/// 2. 之后每24小时执行一次扫描
pub async fn start_file_hash_task(pool: PgPool, storage: Arc<dyn StorageBackend>) {
    // 首次运行：启动时立即执行一次
    tokio::spawn(async move {
        log::info!("[FileHashTask] 启动文件哈希计算任务");

        // 延迟5秒等待服务完全启动
        tokio::time::sleep(Duration::from_secs(5)).await;

        // 执行首次全量扫描（分批处理直到全部完成）
        process_all_missing_hashes(&pool, &storage).await;

        // 设置定时器：每24小时执行一次
        let mut ticker = interval(Duration::from_secs(24 * 60 * 60));
        ticker.tick().await; // 跳过第一次立即触发

        loop {
            ticker.tick().await;
            log::info!("[FileHashTask] 开始定期扫描缺失的哈希值");
            process_all_missing_hashes(&pool, &storage).await;
        }
    });
}

/// 处理所有缺失哈希的资源记录（分批处理直到全部完成）
async fn process_all_missing_hashes(pool: &PgPool, storage: &Arc<dyn StorageBackend>) {
    let mut total_success = 0;
    let mut total_fail = 0;
    let mut batch_count = 0;

    loop {
        batch_count += 1;
        let (success, fail, processed) = process_hash_batch(pool, storage, BATCH_SIZE).await;

        total_success += success;
        total_fail += fail;

        // 如果没有处理任何记录，说明全部完成
        if processed == 0 {
            break;
        }

        // 批次间延迟，避免对存储后端造成过大压力
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    if total_success > 0 || total_fail > 0 {
        log::info!(
            "[FileHashTask] 全量扫描完成 | 批次={}, 成功={}, 失败={}",
            batch_count,
            total_success,
            total_fail
        );
    } else {
        log::info!("[FileHashTask] 没有需要计算哈希的资源");
    }
}

/// 处理一批缺失哈希的资源记录
///
/// 返回 (成功数, 失败数, 处理数)
async fn process_hash_batch(
    pool: &PgPool,
    storage: &Arc<dyn StorageBackend>,
    batch_size: i64,
) -> (i32, i32, i32) {
    // 查询待处理的资源数量（用于告警）
    let pending_count: i64 =
        match sqlx::query_scalar("SELECT COUNT(*) FROM resources WHERE file_hash IS NULL")
            .fetch_one(pool)
            .await
        {
            Ok(count) => count,
            Err(e) => {
                log::error!("[FileHashTask] 查询待处理数量失败 | error={}", e);
                0
            }
        };

    // 告警：待处理数量过多
    if pending_count > ALERT_THRESHOLD {
        log::warn!(
            "[FileHashTask] [告警] 待计算哈希的资源数量过多 | pending={}, threshold={}",
            pending_count,
            ALERT_THRESHOLD
        );
    }

    // 查询一批 file_hash 为 NULL 的资源
    let records = match sqlx::query_as::<_, (Uuid, String, Option<String>, i64)>(
        r#"
        SELECT id, file_path, file_hash, file_size
        FROM resources
        WHERE file_hash IS NULL
        ORDER BY created_at ASC
        LIMIT $1
        "#,
    )
    .bind(batch_size)
    .fetch_all(pool)
    .await
    {
        Ok(records) => records,
        Err(e) => {
            log::error!("[FileHashTask] 查询缺失哈希的资源失败 | error={}", e);
            return (0, 0, 0);
        }
    };

    let count = records.len() as i32;
    if count == 0 {
        return (0, 0, 0);
    }

    log::info!(
        "[FileHashTask] 本批处理 {} 个资源 (总待处理: {})",
        count,
        pending_count
    );

    let mut success_count = 0;
    let mut fail_count = 0;

    for (resource_id, file_path, existing_hash, file_size) in records {
        // 如果已经有哈希，跳过（双重检查）
        if existing_hash.is_some() {
            continue;
        }

        // 限制文件大小
        if file_size > MAX_FILE_SIZE {
            log::warn!(
                "[FileHashTask] 文件过大，跳过哈希计算 | resource_id={}, size={}",
                resource_id,
                file_size
            );
            fail_count += 1;
            continue;
        }

        // 尝试计算哈希（带指数退避重试）
        match calculate_hash_with_exponential_backoff(pool, storage, &file_path, file_size as usize)
            .await
        {
            Ok(hash) => {
                // 更新数据库（使用乐观锁防止并发修改）
                match sqlx::query(
                    "UPDATE resources SET file_hash = $1 WHERE id = $2 AND file_hash IS NULL",
                )
                .bind(&hash)
                .bind(resource_id)
                .execute(pool)
                .await
                {
                    Ok(result) => {
                        if result.rows_affected() > 0 {
                            log::info!(
                                "[FileHashTask] 哈希计算成功 | resource_id={}, hash={}",
                                resource_id,
                                &hash[..16.min(hash.len())]
                            );
                            success_count += 1;
                        } else {
                            log::warn!(
                                "[FileHashTask] 资源已被其他进程更新 | resource_id={}",
                                resource_id
                            );
                        }
                    }
                    Err(e) => {
                        log::error!(
                            "[FileHashTask] 更新哈希失败 | resource_id={}, error={}",
                            resource_id,
                            e
                        );
                        fail_count += 1;
                    }
                }
            }
            Err(e) => {
                log::error!(
                    "[FileHashTask] 计算哈希失败 | resource_id={}, error={}",
                    resource_id,
                    e
                );
                fail_count += 1;
            }
        }

        // 记录间短暂延迟
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    log::info!(
        "[FileHashTask] 本批完成 | 成功={}, 失败={}",
        success_count,
        fail_count
    );

    (success_count, fail_count, count)
}

/// 带指数退避重试机制的哈希计算
///
/// 重试间隔：1s, 2s, 4s, 8s, 16s（最大30s）
async fn calculate_hash_with_exponential_backoff(
    pool: &PgPool,
    storage: &Arc<dyn StorageBackend>,
    file_path: &str,
    file_size: usize,
) -> Result<String, String> {
    const MAX_RETRIES: u32 = 5;
    const INITIAL_DELAY_MS: u64 = 1000;
    const MAX_DELAY_MS: u64 = 30000;

    let mut last_error = String::new();

    for attempt in 0..MAX_RETRIES {
        if attempt > 0 {
            // 指数退避：2^attempt * INITIAL_DELAY_MS
            let delay_ms = std::cmp::min(
                INITIAL_DELAY_MS * (1_u64 << attempt.saturating_sub(1)),
                MAX_DELAY_MS,
            );
            log::info!(
                "[FileHashTask] 重试计算哈希 | attempt={}/{}, delay={}ms",
                attempt + 1,
                MAX_RETRIES,
                delay_ms
            );
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        }

        // 尝试计算hash
        match try_calculate_hash(pool, storage, file_path, file_size).await {
            Ok(hash) => return Ok(hash),
            Err(e) => {
                log::warn!(
                    "[FileHashTask] 读取文件失败 | path={}, attempt={}/{}, error={}",
                    file_path,
                    attempt + 1,
                    MAX_RETRIES,
                    e
                );
                last_error = e;
            }
        }
    }

    Err(format!("重试 {} 次后仍然失败: {}", MAX_RETRIES, last_error))
}

/// 尝试计算hash（辅助函数）
async fn try_calculate_hash(
    pool: &PgPool,
    storage: &Arc<dyn StorageBackend>,
    file_path: &str,
    _file_size: usize,
) -> Result<String, String> {
    // 首先尝试使用当前 storage 读取文件
    match storage.read_file(file_path).await {
        Ok(data) => {
            // 大文件使用流式计算
            if data.len() > 10 * 1024 * 1024 {
                let mut cursor = std::io::Cursor::new(&data);
                match FileService::calculate_hash_streaming(&mut cursor, Some(STREAM_BUFFER_SIZE))
                    .await
                {
                    Ok(hash) => Ok(hash),
                    Err(e) => Err(format!("流式计算hash失败: {}", e)),
                }
            } else {
                let hash = FileService::calculate_hash(&data);
                Ok(hash)
            }
        }
        Err(e) => {
            // 如果是 OSS 存储失败，尝试创建对应的 storage 实例
            let storage_type: Option<String> =
                sqlx::query_scalar("SELECT storage_type FROM resources WHERE file_path = $1")
                    .bind(file_path)
                    .fetch_optional(pool)
                    .await
                    .ok()
                    .flatten();

            if let Some(st) = storage_type {
                if st == "oss" && storage.backend_type() != StorageBackendType::Oss {
                    // 当前是 local 模式，但资源在 OSS
                    let config = Config::from_env();
                    match crate::services::create_storage_backend(&config) {
                        Ok(oss_storage)
                            if oss_storage.backend_type() == StorageBackendType::Oss =>
                        {
                            match oss_storage.read_file(file_path).await {
                                Ok(data) => {
                                    let hash = FileService::calculate_hash(&data);
                                    return Ok(hash);
                                }
                                Err(e2) => {
                                    return Err(format!("OSS 读取失败: {}", e2));
                                }
                            }
                        }
                        _ => {}
                    }
                } else if st == "local" && storage.backend_type() != StorageBackendType::Local {
                    // 当前是 OSS 模式，但资源在本地
                    let config = Config::from_env();
                    match crate::services::create_local_storage(&config) {
                        Ok(local_storage) => match local_storage.read_file(file_path).await {
                            Ok(data) => {
                                let hash = FileService::calculate_hash(&data);
                                return Ok(hash);
                            }
                            Err(e2) => {
                                return Err(format!("本地读取失败: {}", e2));
                            }
                        },
                        Err(e) => {
                            return Err(format!("创建本地存储失败: {}", e));
                        }
                    }
                }
            }
            Err(format!("读取文件失败: {}", e))
        }
    }
}

/// 立即计算指定资源的哈希
///
/// 用于 OSS 上传回调时立即计算哈希
/// 返回计算出的哈希值
#[allow(dead_code)]
pub async fn compute_hash_for_resource(
    storage: &Arc<dyn StorageBackend>,
    file_path: &str,
) -> Result<String, String> {
    // 读取文件内容
    let data = storage
        .read_file(file_path)
        .await
        .map_err(|e| format!("读取文件失败: {}", e))?;

    // 计算哈希
    let hash = FileService::calculate_hash(&data);
    Ok(hash)
}

/// 计算资源哈希并更新数据库（带验证）
///
/// 用于需要确保hash准确性的场景（如Markdown编辑后）
/// 会验证写入的内容与读取的内容一致
#[allow(dead_code)]
pub async fn compute_and_verify_hash(
    pool: &PgPool,
    storage: &Arc<dyn StorageBackend>,
    resource_id: Uuid,
    file_path: &str,
    expected_content: Option<&[u8]>,
) -> Result<String, String> {
    // 读取文件内容
    let data = storage
        .read_file(file_path)
        .await
        .map_err(|e| format!("读取文件失败: {}", e))?;

    // 如果提供了预期内容，验证一致性
    if let Some(expected) = expected_content {
        if data.as_slice() != expected {
            return Err("文件内容验证失败：写入的内容与读取的内容不一致".to_string());
        }
    }

    // 计算哈希
    let hash = FileService::calculate_hash(&data);

    // 更新数据库（使用乐观锁）
    let result = sqlx::query(
        "UPDATE resources SET file_hash = $1 WHERE id = $2 AND (file_hash IS NULL OR file_hash != $1)"
    )
    .bind(&hash)
    .bind(resource_id)
    .execute(pool)
    .await
    .map_err(|e| format!("更新哈希失败: {}", e))?;

    if result.rows_affected() == 0 {
        log::warn!(
            "[FileHashTask] 更新hash时无记录被修改 | resource_id={}",
            resource_id
        );
    }

    Ok(hash)
}
