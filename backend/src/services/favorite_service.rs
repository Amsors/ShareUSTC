use sqlx::PgPool;
use std::io::Write;
use std::sync::Arc;
use uuid::Uuid;

use crate::config::Config;
use crate::services::{create_local_storage, ResourceService, StorageBackend, StorageBackendType};

/// 计算平均分辅助函数
fn calc_avg(total: Option<i32>, count: Option<i32>) -> Option<f64> {
    match (total, count) {
        (Some(t), Some(c)) if c > 0 => Some(t as f64 / c as f64),
        _ => None,
    }
}

use crate::models::{
    AddToFavoriteRequest, CheckResourceInFavoriteResponse, CreateFavoriteRequest,
    CreateFavoriteResponse, Favorite, FavoriteDetailResponse, FavoriteListItem,
    FavoriteListResponse, FavoriteResourceItem, FavoriteResourceStats, UpdateFavoriteRequest,
};

/// 收藏夹服务错误类型
#[derive(Debug, thiserror::Error)]
pub enum FavoriteError {
    #[error("验证错误: {0}")]
    ValidationError(String),
    #[error("未找到: {0}")]
    NotFound(String),
    #[error("资源冲突: {0}")]
    Conflict(String),
    #[error("文件错误: {0}")]
    FileError(String),
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),
}

impl actix_web::ResponseError for FavoriteError {
    fn error_response(&self) -> actix_web::HttpResponse {
        use crate::utils::{bad_request, conflict, internal_error, not_found};
        match self {
            FavoriteError::ValidationError(msg) => bad_request(msg),
            FavoriteError::NotFound(msg) => not_found(msg),
            FavoriteError::Conflict(msg) => conflict(msg),
            // 文件错误按历史行为回传信息（非 SQL 细节）
            FavoriteError::FileError(msg) => {
                log::error!("[Favorite] 文件错误 | error={}", msg);
                internal_error(msg)
            }
            FavoriteError::Database(e) => {
                log::error!("[Favorite] 数据库错误 | error={}", e);
                internal_error("服务器内部错误")
            }
        }
    }
}

pub struct FavoriteService;

impl FavoriteService {
    /// 创建收藏夹
    pub async fn create_favorite(
        pool: &PgPool,
        user_id: Uuid,
        request: CreateFavoriteRequest,
    ) -> Result<CreateFavoriteResponse, FavoriteError> {
        // 验证请求
        request.validate().map_err(FavoriteError::ValidationError)?;

        let name = request.name.trim();

        // 检查是否已存在同名收藏夹
        let existing = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM favorites WHERE user_id = $1 AND name = $2",
        )
        .bind(user_id)
        .bind(name)
        .fetch_one(pool)
        .await?;

        if existing > 0 {
            return Err(FavoriteError::ValidationError(
                "您已存在同名收藏夹".to_string(),
            ));
        }

        // 创建收藏夹
        let favorite = sqlx::query_as::<_, Favorite>(
            r#"
            INSERT INTO favorites (user_id, name)
            VALUES ($1, $2)
            RETURNING id, user_id, name, created_at
            "#,
        )
        .bind(user_id)
        .bind(name)
        .fetch_one(pool)
        .await?;

        Ok(CreateFavoriteResponse {
            id: favorite.id,
            name: favorite.name,
            created_at: favorite.created_at,
        })
    }

    /// 获取用户的收藏夹列表
    pub async fn get_user_favorites(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<FavoriteListResponse, FavoriteError> {
        // 获取收藏夹列表及资源数量
        let rows = sqlx::query!(
            r#"
            SELECT
                f.id,
                f.name,
                f.created_at,
                COUNT(fr.resource_id) as resource_count
            FROM favorites f
            LEFT JOIN favorite_resources fr ON f.id = fr.favorite_id
            WHERE f.user_id = $1
            GROUP BY f.id, f.name, f.created_at
            ORDER BY f.created_at DESC
            "#,
            user_id
        )
        .fetch_all(pool)
        .await?;

        let favorites: Vec<FavoriteListItem> = rows
            .into_iter()
            .map(|row| FavoriteListItem {
                id: row.id,
                name: row.name,
                resource_count: row.resource_count.unwrap_or(0),
                created_at: row
                    .created_at
                    .map(|dt| dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
                    .unwrap_or_else(|| {
                        chrono::Local::now()
                            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                            .to_string()
                    }),
            })
            .collect();

        let total = favorites.len() as i64;

        Ok(FavoriteListResponse { favorites, total })
    }

    /// 获取收藏夹详情
    pub async fn get_favorite_detail(
        pool: &PgPool,
        favorite_id: Uuid,
        user_id: Uuid,
    ) -> Result<FavoriteDetailResponse, FavoriteError> {
        log::debug!(
            "[FavoriteService] 获取收藏夹详情 | favorite_id={}, user_id={}",
            favorite_id,
            user_id
        );

        // 验证收藏夹所有权
        let favorite =
            sqlx::query_as::<_, Favorite>("SELECT * FROM favorites WHERE id = $1 AND user_id = $2")
                .bind(favorite_id)
                .bind(user_id)
                .fetch_optional(pool)
                .await?;

        let favorite = match favorite {
            Some(f) => f,
            None => {
                log::warn!(
                    "[FavoriteService] 收藏夹不存在或无权限 | favorite_id={}, user_id={}",
                    favorite_id,
                    user_id
                );
                return Err(FavoriteError::NotFound("收藏夹不存在".to_string()));
            }
        };

        log::debug!(
            "[FavoriteService] 找到收藏夹 | favorite_id={}, name={}",
            favorite.id,
            favorite.name
        );

        // 获取收藏夹中的资源列表
        let rows = sqlx::query!(
            r#"
            SELECT
                r.id,
                r.title,
                r.course_name,
                r.resource_type,
                r.category,
                r.tags,
                r.file_size,
                COALESCE(r.storage_type, 'local') as storage_type,
                r.updated_at,
                fr.added_at,
                rs.views,
                rs.downloads,
                rs.likes,
                rs.difficulty_total,
                rs.difficulty_count,
                rs.overall_quality_total,
                rs.overall_quality_count,
                rs.answer_quality_total,
                rs.answer_quality_count,
                rs.format_quality_total,
                rs.format_quality_count,
                rs.detail_level_total,
                rs.detail_level_count
            FROM favorite_resources fr
            JOIN resources r ON fr.resource_id = r.id
            LEFT JOIN resource_stats rs ON r.id = rs.resource_id
            WHERE fr.favorite_id = $1
            ORDER BY fr.added_at DESC
            "#,
            favorite_id
        )
        .fetch_all(pool)
        .await?;

        log::debug!(
            "[FavoriteService] 获取到 {} 个资源 | favorite_id={}",
            rows.len(),
            favorite_id
        );

        let resources: Vec<FavoriteResourceItem> = rows
            .into_iter()
            .map(|row| {
                // 解析 tags JSON 字段
                let tags: Option<Vec<String>> = row
                    .tags
                    .and_then(|t| serde_json::from_value::<Vec<String>>(t).ok());

                // 计算各维度的平均分
                let avg_difficulty = calc_avg(row.difficulty_total, row.difficulty_count);
                let avg_overall_quality =
                    calc_avg(row.overall_quality_total, row.overall_quality_count);
                let avg_answer_quality =
                    calc_avg(row.answer_quality_total, row.answer_quality_count);
                let avg_format_quality =
                    calc_avg(row.format_quality_total, row.format_quality_count);
                let avg_detail_level = calc_avg(row.detail_level_total, row.detail_level_count);

                // 评分人数取各维度中的最大值
                let rating_count = [
                    row.difficulty_count.unwrap_or(0),
                    row.overall_quality_count.unwrap_or(0),
                    row.answer_quality_count.unwrap_or(0),
                    row.format_quality_count.unwrap_or(0),
                    row.detail_level_count.unwrap_or(0),
                ]
                .iter()
                .max()
                .copied()
                .unwrap_or(0);

                FavoriteResourceItem {
                    id: row.id,
                    title: row.title,
                    course_name: row.course_name,
                    resource_type: row.resource_type,
                    category: row.category,
                    tags,
                    file_size: row.file_size,
                    added_at: row
                        .added_at
                        .map(|dt: chrono::NaiveDateTime| {
                            dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
                        })
                        .unwrap_or_else(|| {
                            chrono::Local::now()
                                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                                .to_string()
                        }),
                    updated_at: row
                        .updated_at
                        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
                        .unwrap_or_else(|| {
                            chrono::Local::now()
                                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                                .to_string()
                        }),
                    stats: FavoriteResourceStats {
                        views: row.views.unwrap_or(0),
                        downloads: row.downloads.unwrap_or(0),
                        likes: row.likes.unwrap_or(0),
                        avg_difficulty,
                        avg_overall_quality,
                        avg_answer_quality,
                        avg_format_quality,
                        avg_detail_level,
                        rating_count,
                    },
                    storage_type: row.storage_type.unwrap_or_else(|| "local".to_string()),
                }
            })
            .collect();

        let resource_count = resources.len() as i64;

        Ok(FavoriteDetailResponse {
            id: favorite.id,
            name: favorite.name,
            created_at: favorite
                .created_at
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string(),
            resource_count,
            resources,
        })
    }

    /// 更新收藏夹
    pub async fn update_favorite(
        pool: &PgPool,
        favorite_id: Uuid,
        user_id: Uuid,
        request: UpdateFavoriteRequest,
    ) -> Result<(), FavoriteError> {
        // 验证请求
        request.validate().map_err(FavoriteError::ValidationError)?;

        let name = request.name.trim();

        // 检查收藏夹是否存在且属于当前用户
        let existing =
            sqlx::query_as::<_, Favorite>("SELECT * FROM favorites WHERE id = $1 AND user_id = $2")
                .bind(favorite_id)
                .bind(user_id)
                .fetch_optional(pool)
                .await?;

        if existing.is_none() {
            return Err(FavoriteError::NotFound("收藏夹不存在".to_string()));
        }

        // 检查是否已存在其他同名收藏夹
        let duplicate = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM favorites WHERE user_id = $1 AND name = $2 AND id != $3",
        )
        .bind(user_id)
        .bind(name)
        .bind(favorite_id)
        .fetch_one(pool)
        .await?;

        if duplicate > 0 {
            return Err(FavoriteError::ValidationError(
                "您已存在同名收藏夹".to_string(),
            ));
        }

        // 更新收藏夹
        sqlx::query("UPDATE favorites SET name = $1 WHERE id = $2")
            .bind(name)
            .bind(favorite_id)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// 删除收藏夹
    pub async fn delete_favorite(
        pool: &PgPool,
        favorite_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), FavoriteError> {
        // 检查收藏夹是否存在且属于当前用户
        let result = sqlx::query("DELETE FROM favorites WHERE id = $1 AND user_id = $2")
            .bind(favorite_id)
            .bind(user_id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(FavoriteError::NotFound("收藏夹不存在".to_string()));
        }

        Ok(())
    }

    /// 添加资源到收藏夹
    pub async fn add_resource_to_favorite(
        pool: &PgPool,
        favorite_id: Uuid,
        user_id: Uuid,
        request: AddToFavoriteRequest,
    ) -> Result<(), FavoriteError> {
        // 检查收藏夹是否存在且属于当前用户
        let favorite =
            sqlx::query_as::<_, Favorite>("SELECT * FROM favorites WHERE id = $1 AND user_id = $2")
                .bind(favorite_id)
                .bind(user_id)
                .fetch_optional(pool)
                .await?;

        if favorite.is_none() {
            return Err(FavoriteError::NotFound("收藏夹不存在".to_string()));
        }

        // 检查资源是否存在
        let resource_exists =
            sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM resources WHERE id = $1)")
                .bind(request.resource_id)
                .fetch_one(pool)
                .await?;

        if !resource_exists {
            return Err(FavoriteError::NotFound("资源不存在".to_string()));
        }

        // 检查资源是否已在收藏夹中
        let already_in = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM favorite_resources WHERE favorite_id = $1 AND resource_id = $2)"
        )
        .bind(favorite_id)
        .bind(request.resource_id)
        .fetch_one(pool)
        .await?;

        if already_in {
            return Err(FavoriteError::Conflict("资源已在收藏夹中".to_string()));
        }

        // 添加资源到收藏夹
        sqlx::query("INSERT INTO favorite_resources (favorite_id, resource_id) VALUES ($1, $2)")
            .bind(favorite_id)
            .bind(request.resource_id)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// 从收藏夹移除资源
    pub async fn remove_resource_from_favorite(
        pool: &PgPool,
        favorite_id: Uuid,
        resource_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), FavoriteError> {
        // 检查收藏夹是否存在且属于当前用户
        let favorite =
            sqlx::query_as::<_, Favorite>("SELECT * FROM favorites WHERE id = $1 AND user_id = $2")
                .bind(favorite_id)
                .bind(user_id)
                .fetch_optional(pool)
                .await?;

        if favorite.is_none() {
            return Err(FavoriteError::NotFound("收藏夹不存在".to_string()));
        }

        // 删除关联
        let result = sqlx::query(
            "DELETE FROM favorite_resources WHERE favorite_id = $1 AND resource_id = $2",
        )
        .bind(favorite_id)
        .bind(resource_id)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(FavoriteError::NotFound("资源不在该收藏夹中".to_string()));
        }

        Ok(())
    }

    /// 检查资源在哪些收藏夹中
    pub async fn check_resource_in_favorites(
        pool: &PgPool,
        user_id: Uuid,
        resource_id: Uuid,
    ) -> Result<CheckResourceInFavoriteResponse, FavoriteError> {
        // 检查资源是否存在
        let resource_exists =
            sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM resources WHERE id = $1)")
                .bind(resource_id)
                .fetch_one(pool)
                .await?;

        if !resource_exists {
            return Err(FavoriteError::NotFound("资源不存在".to_string()));
        }

        // 获取包含该资源的所有收藏夹ID
        let favorite_ids = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT f.id
            FROM favorites f
            JOIN favorite_resources fr ON f.id = fr.favorite_id
            WHERE f.user_id = $1 AND fr.resource_id = $2
            "#,
        )
        .bind(user_id)
        .bind(resource_id)
        .fetch_all(pool)
        .await?;

        let is_favorited = !favorite_ids.is_empty();

        Ok(CheckResourceInFavoriteResponse {
            in_favorites: favorite_ids,
            is_favorited,
        })
    }

    /// 获取收藏夹中所有资源的文件路径（用于打包下载）
    /// 返回: Vec<(资源ID, 标题, 文件路径, 资源类型, 文件大小, 存储类型)>
    pub async fn get_favorite_resource_paths(
        pool: &PgPool,
        favorite_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<(Uuid, String, String, String, i64, String)>, FavoriteError> {
        // 检查收藏夹是否存在且属于当前用户
        let favorite =
            sqlx::query_as::<_, Favorite>("SELECT * FROM favorites WHERE id = $1 AND user_id = $2")
                .bind(favorite_id)
                .bind(user_id)
                .fetch_optional(pool)
                .await?;

        if favorite.is_none() {
            return Err(FavoriteError::NotFound("收藏夹不存在".to_string()));
        }

        // 获取资源文件路径、标题、资源类型、文件大小和存储类型
        let rows = sqlx::query_as::<_, (Uuid, String, String, String, i64, String)>(
            r#"
            SELECT r.id, r.title, r.file_path, r.resource_type, r.file_size,
                   COALESCE(r.storage_type, 'local') as storage_type
            FROM favorite_resources fr
            JOIN resources r ON fr.resource_id = r.id
            WHERE fr.favorite_id = $1
            "#,
        )
        .bind(favorite_id)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    /// 打包下载收藏夹资源
    /// 返回 ZIP 文件的字节数据
    /// 限制：最多 100 个文件，总大小不超过 500MB
    /// 支持混合存储：根据每个资源的实际 storage_type 选择存储后端
    pub async fn pack_favorite_resources(
        pool: &PgPool,
        storage: &Arc<dyn StorageBackend>,
        config: &Config,
        favorite_id: Uuid,
        user_id: Uuid,
        favorite_name: &str,
    ) -> Result<(Vec<u8>, String), FavoriteError> {
        // 获取资源文件信息（包含存储类型）
        let resources = Self::get_favorite_resource_paths(pool, favorite_id, user_id).await?;

        if resources.is_empty() {
            return Err(FavoriteError::ValidationError("收藏夹为空".to_string()));
        }

        // 检查文件数量限制
        const MAX_FILES: usize = 100;
        if resources.len() > MAX_FILES {
            return Err(FavoriteError::ValidationError(format!(
                "收藏夹资源数量超过限制，最多支持 {} 个文件",
                MAX_FILES
            )));
        }

        // 计算总文件大小并检查限制
        const MAX_TOTAL_SIZE: i64 = 500 * 1024 * 1024; // 500MB
        let total_size: i64 = resources.iter().map(|(_, _, _, _, size, _)| *size).sum();
        if total_size > MAX_TOTAL_SIZE {
            return Err(FavoriteError::ValidationError(format!(
                "收藏夹资源总大小超过限制，最大支持 500MB，当前 {:.2}MB",
                total_size as f64 / 1024.0 / 1024.0
            )));
        }

        // 在内存中创建 ZIP 文件
        let mut zip_buffer = Vec::new();
        {
            let mut zip_writer = zip::ZipWriter::new(std::io::Cursor::new(&mut zip_buffer));
            let options = zip::write::FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .unix_permissions(0o644);

            // 用于检测文件名冲突
            let mut file_names: std::collections::HashMap<String, usize> =
                std::collections::HashMap::new();

            for (resource_id, title, file_path, resource_type, _, storage_type) in &resources {
                // 根据存储类型选择正确的存储后端读取文件
                let file_content = if storage_type == "oss" {
                    // OSS 存储：使用主 storage（如果是 OSS 模式）或创建 OSS 存储
                    if storage.backend_type() == StorageBackendType::Oss {
                        match storage.read_file(file_path).await {
                            Ok(content) => content,
                            Err(e) => {
                                log::warn!(
                                    "读取 OSS 资源文件失败: resource_id={}, path={}, error={}",
                                    resource_id,
                                    file_path,
                                    e
                                );
                                continue; // 跳过无法读取的文件
                            }
                        }
                    } else {
                        // 当前是 local 模式，但需要读取 OSS 文件
                        // 创建临时 OSS 存储实例
                        match crate::services::create_storage_backend(config) {
                            Ok(oss_storage)
                                if oss_storage.backend_type() == StorageBackendType::Oss =>
                            {
                                match oss_storage.read_file(file_path).await {
                                    Ok(content) => content,
                                    Err(e) => {
                                        log::warn!(
                                            "读取 OSS 资源文件失败: resource_id={}, path={}, error={}",
                                            resource_id, file_path, e
                                        );
                                        continue;
                                    }
                                }
                            }
                            _ => {
                                log::warn!(
                                    "无法创建 OSS 存储实例来读取资源: resource_id={}",
                                    resource_id
                                );
                                continue;
                            }
                        }
                    }
                } else {
                    // 本地存储：使用本地存储实例读取
                    match create_local_storage(config) {
                        Ok(local_storage) => match local_storage.read_file(file_path).await {
                            Ok(content) => content,
                            Err(e) => {
                                log::warn!(
                                    "读取本地资源文件失败: resource_id={}, path={}, error={}",
                                    resource_id,
                                    file_path,
                                    e
                                );
                                continue; // 跳过无法读取的文件
                            }
                        },
                        Err(e) => {
                            log::error!("创建本地存储失败: error={}", e);
                            continue;
                        }
                    }
                };

                // 生成安全的文件名 - 保留 Unicode 字符（包括中文），只替换文件系统不安全字符
                let safe_title = title
                    .chars()
                    .map(|c| {
                        // 文件系统不安全的字符: / \ ? % * : | " < > 和控制字符
                        if c.is_control()
                            || matches!(
                                c,
                                '/' | '\\' | '?' | '%' | '*' | ':' | '|' | '"' | '<' | '>'
                            )
                        {
                            '_'
                        } else {
                            c
                        }
                    })
                    .collect::<String>();

                // 确定文件扩展名
                let ext = match resource_type.as_str() {
                    "web_markdown" => "md",
                    "pdf" => "pdf",
                    "ppt" => "ppt",
                    "pptx" => "pptx",
                    "doc" => "doc",
                    "docx" => "docx",
                    "txt" => "txt",
                    "zip" => "zip",
                    _ => "bin",
                };

                // 生成唯一的文件名
                let base_name = format!("{}.{}", safe_title, ext);
                let file_name = if let Some(count) = file_names.get(&base_name) {
                    let new_count = count + 1;
                    file_names.insert(base_name.clone(), new_count);
                    format!("{}_{}.{}", safe_title, new_count, ext)
                } else {
                    file_names.insert(base_name.clone(), 1);
                    base_name
                };

                // 添加到 ZIP
                if let Err(e) = zip_writer.start_file(&file_name, options) {
                    log::warn!("添加文件到ZIP失败: {}, error={}", file_name, e);
                    continue;
                }

                if let Err(e) = zip_writer.write_all(&file_content) {
                    log::warn!("写入文件内容到ZIP失败: {}, error={}", file_name, e);
                    continue;
                }

                log::debug!(
                    "已添加文件到ZIP: {} ({} bytes)",
                    file_name,
                    file_content.len()
                );

                // 增加资源下载计数
                if let Err(e) = ResourceService::increment_downloads(pool, *resource_id).await {
                    log::warn!(
                        "增加资源下载计数失败: resource_id={}, error={}",
                        resource_id,
                        e
                    );
                }
            }

            // 完成 ZIP 文件
            if let Err(e) = zip_writer.finish() {
                return Err(FavoriteError::FileError(format!("创建ZIP文件失败: {}", e)));
            }
        }

        // 生成下载文件名 - 保留 Unicode 字符（包括中文），只替换文件系统不安全字符
        let safe_favorite_name = favorite_name
            .chars()
            .map(|c| {
                // 文件系统不安全的字符: / \ ? % * : | " < > 和控制字符
                if c.is_control()
                    || matches!(
                        c,
                        '/' | '\\' | '?' | '%' | '*' | ':' | '|' | '"' | '<' | '>'
                    )
                {
                    '_'
                } else {
                    c
                }
            })
            .collect::<String>();
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let download_filename = format!("{}_{}.zip", safe_favorite_name, timestamp);

        log::info!(
            "打包下载完成: {}, 文件大小: {} bytes",
            download_filename,
            zip_buffer.len()
        );

        Ok((zip_buffer, download_filename))
    }
}
