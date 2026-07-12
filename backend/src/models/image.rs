use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

/// 图片结构体（对应数据库 images 表）
#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub id: Uuid,
    pub uploader_id: Uuid,
    pub file_path: String,
    pub original_name: Option<String>,
    pub file_size: Option<i32>,
    pub mime_type: Option<String>,
    pub created_at: NaiveDateTime,
    pub storage_type: Option<String>,
}

/// 图片上传响应 DTO
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadImageResponse {
    pub id: Uuid,
    pub url: String,
    pub markdown_link: String,
    pub original_name: Option<String>,
    pub file_size: Option<i32>,
    pub created_at: NaiveDateTime,
}

/// 图片信息响应 DTO
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageInfoResponse {
    pub id: Uuid,
    pub url: String,
    pub markdown_link: String,
    pub original_name: Option<String>,
    pub file_size: Option<i32>,
    pub mime_type: Option<String>,
    pub created_at: NaiveDateTime,
    pub storage_type: String,
}

/// 图片列表响应 DTO
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageListResponse {
    pub images: Vec<ImageInfoResponse>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

impl Image {
    /// 生成图片的公开访问URL
    pub fn get_public_url(&self, base_url: &str) -> String {
        format!("{}/images/{}", base_url, self.id)
    }

    /// 生成Markdown格式的图片链接
    pub fn get_markdown_link(&self, base_url: &str, description: &str) -> String {
        format!("![{}]({})", description, self.get_public_url(base_url))
    }
}

impl ImageInfoResponse {
    /// 从 Image 创建响应，`base_url` 由调用方从 `Config::image_base_url` 传入。
    /// 不提供 `From<Image>`（会诱导散点读取环境变量），统一走本方法显式传参。
    pub fn from_image_with_base_url(image: Image, base_url: &str) -> Self {
        ImageInfoResponse {
            id: image.id,
            url: image.get_public_url(base_url),
            markdown_link: image
                .get_markdown_link(base_url, image.original_name.as_deref().unwrap_or("image")),
            original_name: image.original_name.clone(),
            file_size: image.file_size,
            mime_type: image.mime_type.clone(),
            created_at: image.created_at,
            storage_type: image
                .storage_type
                .clone()
                .unwrap_or_else(|| "local".to_string()),
        }
    }
}
