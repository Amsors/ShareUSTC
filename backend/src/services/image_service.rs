use crate::models::{
    image::{Image, ImageInfoResponse, ImageListResponse, UploadImageResponse},
    CurrentUser,
};
use sqlx::PgPool;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug)]
pub enum ImageError {
    DatabaseError(String),
    FileError(String),
    NotFound(String),
    ValidationError(String),
    Unauthorized(String),
}

impl std::fmt::Display for ImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageError::DatabaseError(msg) => write!(f, "数据库错误: {}", msg),
            ImageError::FileError(msg) => write!(f, "文件错误: {}", msg),
            ImageError::NotFound(msg) => write!(f, "未找到: {}", msg),
            ImageError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
            ImageError::Unauthorized(msg) => write!(f, "未授权: {}", msg),
        }
    }
}

impl std::error::Error for ImageError {}

pub struct ImageService;

impl ImageService {
    pub async fn upload_image(
        pool: &PgPool,
        user: &CurrentUser,
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
            return Err(ImageError::ValidationError(
                "无法识别文件类型".to_string(),
            ));
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
        let storage_filename = format!("{}.{}", image_id, ext);

        let upload_dir = std::env::var("IMAGE_UPLOAD_PATH")
            .unwrap_or_else(|_| "./uploads/images".to_string());
        let file_path = Path::new(&upload_dir).join(&storage_filename);

        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| ImageError::FileError(format!("创建目录失败: {}", e)))?;
        }

        // 先保存文件到临时路径，等数据库插入成功后再确认
        tokio::fs::write(&file_path, &file_data)
            .await
            .map_err(|e| ImageError::FileError(format!("保存文件失败: {}", e)))?;

        let file_size = file_data.len() as i32;

        // 将文件路径转换为字符串（用于错误处理中的清理）
        let file_path_str = file_path.to_string_lossy().to_string();

        let image: Image = match sqlx::query_as::<_, Image>(
            r#"
            INSERT INTO images (id, uploader_id, file_path, original_name, file_size, mime_type)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(image_id)
        .bind(user.id)
        .bind(&file_path_str)
        .bind(file_name)
        .bind(file_size)
        .bind(detected_mime)
        .fetch_one(pool)
        .await {
            Ok(img) => img,
            Err(e) => {
                // 数据库插入失败时清理已保存的文件
                log::warn!("[Image] 数据库插入失败，清理文件 | image_id={}, path={}, error={}", image_id, file_path_str, e);
                if let Err(cleanup_err) = tokio::fs::remove_file(&file_path).await {
                    log::error!("[Image] 清理文件失败 | path={}, error={}", file_path_str, cleanup_err);
                }
                return Err(ImageError::DatabaseError(e.to_string()));
            }
        };

        let base_url = std::env::var("IMAGE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let url = image.get_public_url(&base_url);
        let markdown_link = image.get_markdown_link(&base_url, file_name);

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

        let total: i64 = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM images WHERE uploader_id = $1",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| ImageError::DatabaseError(e.to_string()))?;

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
        .await
        .map_err(|e| ImageError::DatabaseError(e.to_string()))?;

        let image_responses: Vec<ImageInfoResponse> = images
            .into_iter()
            .map(ImageInfoResponse::from)
            .collect();

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
        let image: Image = sqlx::query_as::<_, Image>(
            "SELECT * FROM images WHERE id = $1",
        )
        .bind(image_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| ImageError::DatabaseError(e.to_string()))?
        .ok_or_else(|| ImageError::NotFound(format!("图片 {} 不存在", image_id)))?;

        Ok(ImageInfoResponse::from(image))
    }

    pub async fn delete_image(
        pool: &PgPool,
        user: &CurrentUser,
        image_id: Uuid,
    ) -> Result<(), ImageError> {
        let image: Image = sqlx::query_as::<_, Image>(
            "SELECT * FROM images WHERE id = $1",
        )
        .bind(image_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| ImageError::DatabaseError(e.to_string()))?
        .ok_or_else(|| ImageError::NotFound(format!("图片 {} 不存在", image_id)))?;

        if image.uploader_id != user.id && user.role != crate::models::UserRole::Admin {
            return Err(ImageError::Unauthorized(
                "没有权限删除此图片".to_string(),
            ));
        }

        let file_path = Path::new(&image.file_path);
        if file_path.exists() {
            tokio::fs::remove_file(file_path)
                .await
                .map_err(|e| ImageError::FileError(format!("删除文件失败: {}", e)))?;
        }

        sqlx::query("DELETE FROM images WHERE id = $1")
            .bind(image_id)
            .execute(pool)
            .await
            .map_err(|e| ImageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_image_path(
        pool: &PgPool,
        image_id: Uuid,
    ) -> Result<(String, Option<String>), ImageError> {
        let row: (String, Option<String>) = sqlx::query_as(
            "SELECT file_path, mime_type FROM images WHERE id = $1",
        )
        .bind(image_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| ImageError::DatabaseError(e.to_string()))?
        .ok_or_else(|| ImageError::NotFound(format!("图片 {} 不存在", image_id)))?;

        Ok(row)
    }
}
