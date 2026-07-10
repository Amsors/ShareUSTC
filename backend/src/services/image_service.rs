use crate::config::Config;
use crate::models::{
    image::{Image, ImageInfoResponse, ImageListResponse, UploadImageResponse},
    CurrentUser,
};
use sqlx::PgPool;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum ImageError {
    #[error("文件错误: {0}")]
    FileError(String),
    #[error("未找到: {0}")]
    NotFound(String),
    #[error("验证错误: {0}")]
    ValidationError(String),
    #[error("未授权: {0}")]
    Unauthorized(String),
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),
}

impl actix_web::ResponseError for ImageError {
    fn error_response(&self) -> actix_web::HttpResponse {
        use crate::utils::{bad_request, forbidden, internal_error, not_found};
        match self {
            ImageError::NotFound(msg) => not_found(msg),
            ImageError::ValidationError(msg) => bad_request(msg),
            ImageError::Unauthorized(msg) => forbidden(msg),
            // 文件错误按历史行为回传信息（非 SQL 细节）
            ImageError::FileError(msg) => {
                log::error!("[Image] 文件错误 | error={}", msg);
                internal_error(msg)
            }
            ImageError::Database(e) => {
                log::error!("[Image] 数据库错误 | error={}", e);
                internal_error("服务器内部错误")
            }
        }
    }
}

impl From<super::storage_service::StorageError> for ImageError {
    fn from(err: super::storage_service::StorageError) -> Self {
        match err {
            super::storage_service::StorageError::Validation(msg) => {
                ImageError::ValidationError(msg)
            }
            super::storage_service::StorageError::Config(msg) => ImageError::FileError(msg),
            super::storage_service::StorageError::NotFound(msg) => ImageError::NotFound(msg),
            super::storage_service::StorageError::Io(msg) => ImageError::FileError(msg),
            super::storage_service::StorageError::Backend(msg) => ImageError::FileError(msg),
        }
    }
}

pub struct ImageService;

impl ImageService {
    pub async fn create_image_from_oss_callback(
        pool: &PgPool,
        user: &CurrentUser,
        storage: &Arc<dyn super::StorageBackend>,
        config: &Config,
        oss_key: &str,
        original_name: Option<&str>,
        metadata: super::StorageFileMetadata,
    ) -> Result<UploadImageResponse, ImageError> {
        let file_size = metadata
            .content_length
            .ok_or_else(|| ImageError::ValidationError("无法获取文件大小".to_string()))?
            as usize;
        if file_size == 0 {
            return Err(ImageError::ValidationError("文件不能为空".to_string()));
        }

        const MAX_FILE_SIZE: usize = 5 * 1024 * 1024;
        if file_size > MAX_FILE_SIZE {
            return Err(ImageError::ValidationError(format!(
                "文件大小超过限制。最大允许 5MB，当前 {}MB",
                file_size / 1024 / 1024
            )));
        }

        let object_name = oss_key.rsplit('/').next().unwrap_or(oss_key);
        let file_extension = Path::new(object_name)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase());
        let detected_mime = match file_extension.as_deref() {
            Some("jpg") | Some("jpeg") => Some("image/jpeg".to_string()),
            Some("png") => Some("image/png".to_string()),
            _ => metadata.content_type.clone(),
        };
        let mime_value = detected_mime.as_deref().unwrap_or_default().to_lowercase();
        if mime_value != "image/jpeg" && mime_value != "image/jpg" && mime_value != "image/png" {
            return Err(ImageError::ValidationError(format!(
                "不支持的图片类型: {}。仅支持 JPEG、JPG、PNG",
                if mime_value.is_empty() {
                    "unknown"
                } else {
                    &mime_value
                }
            )));
        }

        let storage_type = storage.backend_type().as_str().to_string();
        let image_id = Uuid::new_v4();
        let image: Image = match sqlx::query_as::<_, Image>(
            r#"
            INSERT INTO images (id, uploader_id, file_path, original_name, file_size, mime_type, storage_type)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(image_id)
        .bind(user.id)
        .bind(oss_key)
        .bind(original_name)
        .bind(file_size as i32)
        .bind(detected_mime)
        .bind(&storage_type)
        .fetch_one(pool)
        .await
        {
            Ok(image) => image,
            Err(e) => {
                if let Err(cleanup_err) = storage.delete_file(oss_key).await {
                    log::warn!(
                        "[Image] OSS 回调入库失败后清理文件失败 | key={}, error={}",
                        oss_key,
                        cleanup_err
                    );
                }
                return Err(ImageError::Database(e));
            }
        };

        let base_url = &config.image_base_url;
        let fallback_name = original_name.unwrap_or("image");
        Ok(UploadImageResponse {
            id: image.id,
            url: image.get_public_url(&base_url),
            markdown_link: image.get_markdown_link(&base_url, fallback_name),
            original_name: image.original_name,
            file_size: image.file_size,
            created_at: image.created_at,
        })
    }

    pub async fn upload_image(
        pool: &PgPool,
        user: &CurrentUser,
        storage: &Arc<dyn super::StorageBackend>,
        config: &Config,
        file_name: &str,
        file_data: Vec<u8>,
        mime_type: Option<&str>,
    ) -> Result<UploadImageResponse, ImageError> {
        let allowed_types = ["image/jpeg", "image/jpg", "image/png"];
        let file_extension = Path::new(file_name)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase());

        let detected_mime = match file_extension.as_deref() {
            Some("jpg") | Some("jpeg") => Some("image/jpeg"),
            Some("png") => Some("image/png"),
            _ => mime_type,
        };

        if let Some(mime) = detected_mime {
            if !allowed_types.contains(&mime) {
                return Err(ImageError::ValidationError(format!(
                    "不支持的文件类型: {}。仅支持 JPEG, JPG, PNG 格式",
                    mime
                )));
            }
        } else {
            return Err(ImageError::ValidationError("无法识别文件类型".to_string()));
        }

        // MIME 类型与扩展名一致性校验
        // 防止通过修改扩展名伪装文件类型
        if let (Some(mime), Some(ext)) = (detected_mime, file_extension.as_deref()) {
            let is_consistent = match ext {
                "jpg" | "jpeg" => mime == "image/jpeg",
                "png" => mime == "image/png",
                _ => true, // 未知扩展名时不进行一致性校验
            };

            if !is_consistent {
                return Err(ImageError::ValidationError(format!(
                    "文件类型与扩展名不一致: 扩展名为 .{}, 但 MIME 类型为 {}",
                    ext, mime
                )));
            }
        }

        const MAX_FILE_SIZE: usize = 5 * 1024 * 1024;
        if file_data.len() > MAX_FILE_SIZE {
            return Err(ImageError::ValidationError(format!(
                "文件大小超过限制。最大允许 5MB，当前 {}MB",
                file_data.len() / 1024 / 1024
            )));
        }

        let image_id = Uuid::new_v4();
        let ext = file_extension.unwrap_or_else(|| "png".to_string());
        let storage_key = format!("images/{}.{}", image_id, ext);
        let storage_type = storage.backend_type().as_str().to_string();
        let file_size = file_data.len() as i32;
        let file_path = storage
            .save_file(&storage_key, file_data, detected_mime)
            .await?;

        let image: Image = match sqlx::query_as::<_, Image>(
            r#"
            INSERT INTO images (id, uploader_id, file_path, original_name, file_size, mime_type, storage_type)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(image_id)
        .bind(user.id)
        .bind(&file_path)
        .bind(file_name)
        .bind(file_size)
        .bind(detected_mime)
        .bind(&storage_type)
        .fetch_one(pool)
        .await {
            Ok(img) => img,
            Err(e) => {
                // 数据库插入失败时清理已保存的文件
                log::warn!(
                    "[Image] 数据库插入失败，清理文件 | image_id={}, path={}, error={}",
                    image_id,
                    file_path,
                    e
                );
                if let Err(cleanup_err) = storage.delete_file(&file_path).await {
                    log::error!(
                        "[Image] 清理文件失败 | path={}, error={}",
                        file_path,
                        cleanup_err
                    );
                }
                return Err(ImageError::Database(e));
            }
        };

        let base_url = &config.image_base_url;
        let url = image.get_public_url(base_url);
        let markdown_link = image.get_markdown_link(base_url, file_name);

        Ok(UploadImageResponse {
            id: image.id,
            url,
            markdown_link,
            original_name: image.original_name,
            file_size: image.file_size,
            created_at: image.created_at,
        })
    }

    pub async fn get_user_images(
        pool: &PgPool,
        user_id: Uuid,
        page: i32,
        per_page: i32,
    ) -> Result<ImageListResponse, ImageError> {
        let offset = (page - 1) * per_page;

        let total: i64 =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM images WHERE uploader_id = $1")
                .bind(user_id)
                .fetch_one(pool)
                .await?;

        let images: Vec<Image> = sqlx::query_as::<_, Image>(
            r#"
            SELECT * FROM images
            WHERE uploader_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(per_page as i64)
        .bind(offset as i64)
        .fetch_all(pool)
        .await?;

        let image_responses: Vec<ImageInfoResponse> =
            images.into_iter().map(ImageInfoResponse::from).collect();

        Ok(ImageListResponse {
            images: image_responses,
            total,
            page,
            per_page,
        })
    }

    pub async fn get_image_by_id(
        pool: &PgPool,
        image_id: Uuid,
    ) -> Result<ImageInfoResponse, ImageError> {
        let image: Image = sqlx::query_as::<_, Image>("SELECT * FROM images WHERE id = $1")
            .bind(image_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| ImageError::NotFound(format!("图片 {} 不存在", image_id)))?;

        Ok(ImageInfoResponse::from(image))
    }

    pub async fn delete_image(
        pool: &PgPool,
        user: &CurrentUser,
        storage: &Arc<dyn super::StorageBackend>,
        image_id: Uuid,
    ) -> Result<(), ImageError> {
        let image: Image = sqlx::query_as::<_, Image>("SELECT * FROM images WHERE id = $1")
            .bind(image_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| ImageError::NotFound(format!("图片 {} 不存在", image_id)))?;

        if image.uploader_id != user.id && user.role != crate::models::UserRole::Admin {
            return Err(ImageError::Unauthorized("没有权限删除此图片".to_string()));
        }

        storage.delete_file(&image.file_path).await?;

        sqlx::query("DELETE FROM images WHERE id = $1")
            .bind(image_id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn get_image_path(
        pool: &PgPool,
        image_id: Uuid,
    ) -> Result<(String, Option<String>, Option<String>), ImageError> {
        let row: (String, Option<String>, Option<String>) =
            sqlx::query_as("SELECT file_path, mime_type, storage_type FROM images WHERE id = $1")
                .bind(image_id)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| ImageError::NotFound(format!("图片 {} 不存在", image_id)))?;

        Ok(row)
    }
}
