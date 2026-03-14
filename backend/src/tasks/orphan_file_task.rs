/// 孤立文件扫描任务
///
/// 后台定时任务，用于：
/// 1. 启动时扫描 uploads 文件夹中的文件
/// 2. 每24小时重新扫描一次
/// 3. 检查文件是否在数据库中有记录（resources 表或 images 表）
/// 4. 如果发现孤立文件，记录 WARNING 日志，但不删除
/// 5. 支持本地存储和 OSS 存储
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use sqlx::PgPool;
use tokio::time::interval;

use crate::config::Config;
use crate::services::{StorageBackend, StorageBackendType};

/// 扫描批次大小（每次处理的文件数量）
const SCAN_BATCH_SIZE: usize = 100;

/// 启动孤立文件扫描任务
///
/// 在服务启动时调用，会：
/// 1. 立即执行一次全量扫描
/// 2. 之后每24小时执行一次扫描
pub async fn start_orphan_file_task(pool: PgPool, storage: Arc<dyn StorageBackend>) {
    tokio::spawn(async move {
        log::info!("[OrphanFileTask] 启动孤立文件扫描任务");

        // 延迟10秒等待服务完全启动
        tokio::time::sleep(Duration::from_secs(10)).await;

        // 执行首次扫描
        scan_orphan_files(&pool, &storage).await;

        // 设置定时器：每24小时执行一次
        let mut ticker = interval(Duration::from_secs(24 * 60 * 60));
        ticker.tick().await; // 跳过第一次立即触发

        loop {
            ticker.tick().await;
            log::info!("[OrphanFileTask] 开始定期扫描孤立文件");
            scan_orphan_files(&pool, &storage).await;
        }
    });
}

/// 扫描孤立文件
async fn scan_orphan_files(pool: &PgPool, storage: &Arc<dyn StorageBackend>) {
    log::info!("[OrphanFileTask] 开始扫描孤立文件...");

    // 获取所有数据库中的文件路径（包含存储类型信息）
    let db_resource_files = match get_all_resource_files_with_storage(pool).await {
        Ok(files) => files,
        Err(e) => {
            log::error!("[OrphanFileTask] 获取资源文件列表失败 | error={}", e);
            return;
        }
    };

    let db_image_files = match get_all_image_files_with_storage(pool).await {
        Ok(files) => files,
        Err(e) => {
            log::error!("[OrphanFileTask] 获取图片文件列表失败 | error={}", e);
            return;
        }
    };

    // 分离本地存储和 OSS 存储的文件
    let mut db_local_files: HashSet<String> = HashSet::new();
    let mut db_oss_files: HashSet<String> = HashSet::new();

    for (path, storage_type) in db_resource_files.iter().chain(db_image_files.iter()) {
        let is_oss = storage_type.as_deref() == Some("oss");
        if is_oss {
            db_oss_files.insert(path.clone());
        } else {
            db_local_files.insert(path.clone());
        }
    }

    log::info!(
        "[OrphanFileTask] 数据库中共有 {} 个文件记录 | 本地={}, OSS={}",
        db_local_files.len() + db_oss_files.len(),
        db_local_files.len(),
        db_oss_files.len()
    );

    let config = Config::from_env();

    // 总是扫描本地目录（即使当前使用 OSS，也可能有历史遗留的本地文件）
    log::info!("[OrphanFileTask] 开始扫描本地目录...");
    scan_local_directory(&config.resource_upload_path, "resources", &db_local_files).await;
    scan_local_directory(&config.image_upload_path, "images", &db_local_files).await;

    // 如果当前是 OSS 模式，检查数据库中标记为 OSS 的文件是否需要从本地清理
    if storage.backend_type() == StorageBackendType::Oss {
        log::info!("[OrphanFileTask] 当前使用 OSS 存储，检查本地是否存在已迁移到 OSS 的文件...");
        check_local_files_migrated_to_oss(
            &config.resource_upload_path,
            &config.image_upload_path,
            &db_oss_files,
        )
        .await;
    }

    log::info!("[OrphanFileTask] 孤立文件扫描完成");
}

/// 获取所有资源文件路径（包含存储类型）
async fn get_all_resource_files_with_storage(
    pool: &PgPool,
) -> Result<Vec<(String, Option<String>)>, sqlx::Error> {
    let rows: Vec<(String, Option<String>)> = sqlx::query_as(
        "SELECT file_path, COALESCE(storage_type, 'local') as storage_type FROM resources WHERE file_path IS NOT NULL"
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// 获取所有图片文件路径（包含存储类型）
async fn get_all_image_files_with_storage(
    pool: &PgPool,
) -> Result<Vec<(String, Option<String>)>, sqlx::Error> {
    let rows: Vec<(String, Option<String>)> = sqlx::query_as(
        "SELECT file_path, COALESCE(storage_type, 'local') as storage_type FROM images WHERE file_path IS NOT NULL"
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// 检查本地是否存在已迁移到 OSS 的文件
/// 这些文件在数据库中标记为 OSS 存储，但可能还残留在本地
async fn check_local_files_migrated_to_oss(
    resource_dir: &str,
    image_dir: &str,
    db_oss_files: &HashSet<String>,
) {
    // 扫描本地目录，检查是否有文件同时存在于本地和 OSS（重复文件）
    let check_dir = async |dir: &str, dir_type: &str| {
        let path = Path::new(dir);

        if !path.exists() {
            return;
        }

        match walk_directory(path).await {
            Ok(files) => {
                for file_path in files {
                    let normalized = normalize_path(&file_path, dir);

                    // 检查此文件是否在数据库中标记为 OSS
                    if is_file_in_database(&normalized, db_oss_files) {
                        log::warn!(
                            "[OrphanFileTask] 发现重复文件（本地和OSS同时存在） | type={}, path={}。该文件在数据库中标记为OSS存储，但本地仍有副本",
                            dir_type,
                            file_path.display()
                        );
                    }
                }
            }
            Err(e) => {
                log::warn!(
                    "[OrphanFileTask] 检查重复文件失败 | type={}, path={}, error={}",
                    dir_type,
                    dir,
                    e
                );
            }
        }
    };

    check_dir(resource_dir, "resources").await;
    check_dir(image_dir, "images").await;
}

/// 扫描本地目录
async fn scan_local_directory(dir_path: &str, dir_type: &str, db_files: &HashSet<String>) {
    let path = Path::new(dir_path);

    if !path.exists() {
        log::warn!(
            "[OrphanFileTask] 目录不存在 | type={}, path={}",
            dir_type,
            dir_path
        );
        return;
    }

    log::info!("[OrphanFileTask] 开始扫描 {} 目录: {}", dir_type, dir_path);

    let mut orphan_files = Vec::new();
    let mut total_files = 0;

    match walk_directory(path).await {
        Ok(files) => {
            for file_path in files {
                total_files += 1;

                // 规范化路径以便比较
                let normalized_path = normalize_path(&file_path, dir_path);

                // 检查是否在数据库中
                if !is_file_in_database(&normalized_path, db_files) {
                    orphan_files.push(file_path.clone());

                    // 记录 WARNING 日志
                    log::warn!(
                        "[OrphanFileTask] 发现孤立文件 | type={}, path={} (normalized: {})",
                        dir_type,
                        file_path.display(),
                        normalized_path
                    );
                }

                // 批次处理，避免内存占用过大
                if total_files % SCAN_BATCH_SIZE == 0 {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }
        }
        Err(e) => {
            log::error!(
                "[OrphanFileTask] 扫描目录失败 | type={}, path={}, error={}",
                dir_type,
                dir_path,
                e
            );
            return;
        }
    }

    // 汇总报告
    if orphan_files.is_empty() {
        log::info!(
            "[OrphanFileTask] {} 目录扫描完成 | 总文件数={}, 孤立文件数=0，一切正常",
            dir_type,
            total_files
        );
    } else {
        log::warn!(
            "[OrphanFileTask] {} 目录扫描完成 | 总文件数={}, 孤立文件数={}。发现孤立文件，请检查这些文件是否应该被删除：",
            dir_type, total_files, orphan_files.len()
        );

        // 列出所有孤立文件
        for (idx, file) in orphan_files.iter().enumerate() {
            log::warn!(
                "[OrphanFileTask] 孤立文件 #{} | path={}",
                idx + 1,
                file.display()
            );
        }
    }
}

/// 遍历目录（非递归实现，避免 async 递归问题）
async fn walk_directory(dir: &Path) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
    let mut files = Vec::new();

    if !dir.is_dir() {
        return Ok(files);
    }

    // 使用栈来实现非递归遍历
    let mut dirs_to_scan: Vec<std::path::PathBuf> = vec![dir.to_path_buf()];

    while let Some(current_dir) = dirs_to_scan.pop() {
        let mut entries = tokio::fs::read_dir(&current_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.is_dir() {
                // 将子目录加入待扫描列表
                dirs_to_scan.push(path);
            } else if path.is_file() {
                files.push(path);
            }
        }
    }

    Ok(files)
}

/// 规范化路径以便与数据库记录比较
fn normalize_path(file_path: &Path, base_dir: &str) -> String {
    // 获取绝对路径
    let absolute_path = if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(file_path)
    };

    // 尝试获取相对于上传目录的路径
    let base = Path::new(base_dir);
    if let Ok(relative) = absolute_path.strip_prefix(base) {
        return relative.to_string_lossy().replace('\\', "/");
    }

    // 如果无法获取相对路径，返回绝对路径
    absolute_path.to_string_lossy().replace('\\', "/")
}

/// 检查文件是否在数据库中
fn is_file_in_database(file_path: &str, db_files: &HashSet<String>) -> bool {
    // 直接匹配
    if db_files.contains(file_path) {
        return true;
    }

    // 尝试匹配不同形式的路径
    for db_path in db_files.iter() {
        // 提取文件名进行比较
        if let Some(file_name) = Path::new(file_path).file_name() {
            if let Some(db_name) = Path::new(db_path).file_name() {
                if file_name == db_name {
                    return true;
                }
            }
        }

        // 检查后缀匹配（处理相对路径和绝对路径的情况）
        if db_path.ends_with(file_path) || file_path.ends_with(db_path) {
            return true;
        }

        // 处理 ./ 开头的路径
        let trimmed_path = file_path.trim_start_matches("./");
        let trimmed_db_path = db_path.trim_start_matches("./");

        if trimmed_path == trimmed_db_path {
            return true;
        }

        if db_path.ends_with(trimmed_path) || trimmed_db_path.ends_with(file_path) {
            return true;
        }
    }

    false
}
