use crate::models::resource::ResourceType;
use sha2::{Digest, Sha256};
use std::path::Path;
use uuid::Uuid;

#[derive(Debug)]
pub enum FileError {
    ValidationError(String),
    FileSystemError(String),
    NotFound(String),
}

impl std::fmt::Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
            FileError::FileSystemError(msg) => write!(f, "文件系统错误: {}", msg),
            FileError::NotFound(msg) => write!(f, "未找到: {}", msg),
        }
    }
}

impl std::error::Error for FileError {}

pub struct FileService;

impl FileService {
    /// 最大文件大小 (100MB)
    pub const MAX_FILE_SIZE: usize = 100 * 1024 * 1024;

    /// 资源上传路径
    pub fn get_resource_upload_path() -> String {
        std::env::var("RESOURCE_UPLOAD_PATH")
            .unwrap_or_else(|_| "./uploads/resources".to_string())
    }

    /// 计算文件 SHA-256 哈希
    pub fn calculate_hash(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// 验证资源文件
    pub fn validate_resource_file(
        file_name: &str,
        file_data: &[u8],
        mime_type: Option<&str>,
    ) -> Result<ResourceType, FileError> {
        // 检查文件大小
        if file_data.is_empty() {
            return Err(FileError::ValidationError("文件不能为空".to_string()));
        }

        if file_data.len() > Self::MAX_FILE_SIZE {
            return Err(FileError::ValidationError(format!(
                "文件大小超过限制。最大允许 100MB，当前 {:.2}MB",
                file_data.len() as f64 / 1024.0 / 1024.0
            )));
        }

        // 从文件名获取扩展名
        let extension = Path::new(file_name)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase());

        let resource_type = match extension.as_deref() {
            Some(ext) => ResourceType::from_extension(ext),
            None => {
                // 尝试从 MIME 类型推断
                mime_type.map_or(ResourceType::Other, |mime| match mime {
                    "application/pdf" => ResourceType::Pdf,
                    "text/plain" => ResourceType::Txt,
                    "text/markdown" => ResourceType::WebMarkdown,
                    "image/jpeg" => ResourceType::Jpeg,
                    "image/png" => ResourceType::Png,
                    "application/zip" => ResourceType::Zip,
                    _ => ResourceType::Other,
                })
            }
        };

        // 检查文件扩展名是否受支持
        if resource_type == ResourceType::Other {
            return Err(FileError::ValidationError(format!(
                "不支持的文件类型。支持的类型: {}",
                ResourceType::supported_extensions().join(", ")
            )));
        }

        Ok(resource_type)
    }

    /// 保存资源文件
    pub async fn save_resource_file(
        file_data: Vec<u8>,
        resource_type: &ResourceType,
    ) -> Result<(String, String, i64), FileError> {
        let resource_id = Uuid::new_v4();
        let extension = match resource_type {
            ResourceType::WebMarkdown => "md",
            ResourceType::Ppt => "ppt",
            ResourceType::Pptx => "pptx",
            ResourceType::Doc => "doc",
            ResourceType::Docx => "docx",
            ResourceType::Pdf => "pdf",
            ResourceType::Txt => "txt",
            ResourceType::Jpeg => "jpeg",
            ResourceType::Jpg => "jpg",
            ResourceType::Png => "png",
            ResourceType::Zip => "zip",
            ResourceType::Other => "bin",
        };

        let file_name = format!("{}.{}", resource_id, extension);
        let upload_dir = Self::get_resource_upload_path();
        let file_path = Path::new(&upload_dir).join(&file_name);

        // 创建目录（如果不存在）
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| FileError::FileSystemError(format!("创建目录失败: {}", e)))?;
        }

        // 写入文件
        tokio::fs::write(&file_path, &file_data)
            .await
            .map_err(|e| FileError::FileSystemError(format!("保存文件失败: {}", e)))?;

        let file_hash = Self::calculate_hash(&file_data);
        let file_size = file_data.len() as i64;

        Ok((
            file_path.to_string_lossy().to_string(),
            file_hash,
            file_size,
        ))
    }

    /// 删除资源文件
    pub async fn delete_resource_file(file_path: &str) -> Result<(), FileError> {
        let path = Path::new(file_path);
        if path.exists() {
            tokio::fs::remove_file(path)
                .await
                .map_err(|e| FileError::FileSystemError(format!("删除文件失败: {}", e)))?;
        }
        Ok(())
    }

    /// 读取资源文件
    pub async fn read_resource_file(file_path: &str) -> Result<Vec<u8>, FileError> {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(FileError::NotFound(format!("文件不存在: {}", file_path)));
        }

        tokio::fs::read(path)
            .await
            .map_err(|e| FileError::FileSystemError(format!("读取文件失败: {}", e)))
    }

    /// 获取文件 MIME 类型（通过文件路径）（预留接口）
    #[allow(dead_code)]
    pub fn get_mime_type(file_path: &str) -> String {
        // 处理 .ext.bin 格式的文件名（如 xxx.pdf.bin）
        let path = Path::new(file_path);
        let file_stem = path.file_stem().and_then(|s| s.to_str());

        // 如果文件名包含多个扩展名，尝试获取真实的扩展名
        let extension = if let Some(stem) = file_stem {
            // 检查是否还有子扩展名（如 xxx.pdf.bin 中的 .pdf）
            let stem_path = Path::new(stem);
            stem_path.extension().and_then(|ext| ext.to_str())
        } else {
            path.extension().and_then(|ext| ext.to_str())
        };

        Self::mime_type_from_extension(extension)
    }

    /// 根据资源类型获取 MIME 类型
    pub fn get_mime_type_by_type(resource_type: &str) -> String {
        let resource_type_lower = resource_type.to_lowercase();
        match resource_type_lower.as_str() {
            "web_markdown" | "md" | "markdown" => "text/markdown",
            "ppt" => "application/vnd.ms-powerpoint",
            "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            "doc" => "application/msword",
            "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "pdf" => "application/pdf",
            "txt" => "text/plain",
            "jpeg" | "jpg" => "image/jpeg",
            "png" => "image/png",
            "zip" => "application/zip",
            _ => "application/octet-stream",
        }
        .to_string()
    }

    fn mime_type_from_extension(extension: Option<&str>) -> String {
        match extension.map(|e| e.to_lowercase()).as_deref() {
            Some("md") => "text/markdown",
            Some("ppt") => "application/vnd.ms-powerpoint",
            Some("pptx") => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            Some("doc") => "application/msword",
            Some("docx") => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            Some("pdf") => "application/pdf",
            Some("txt") => "text/plain",
            Some("jpeg") | Some("jpg") => "image/jpeg",
            Some("png") => "image/png",
            Some("zip") => "application/zip",
            _ => "application/octet-stream",
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_hash() {
        let data = b"hello world";
        let hash = FileService::calculate_hash(data);
        assert_eq!(hash.len(), 64); // SHA-256 hash is 64 hex characters
    }

    #[test]
    fn test_validate_resource_file_empty() {
        let result = FileService::validate_resource_file("test.pdf", &[], Some("application/pdf"));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_resource_file_too_large() {
        let large_data = vec![0u8; FileService::MAX_FILE_SIZE + 1];
        let result = FileService::validate_resource_file("test.pdf", &large_data, Some("application/pdf"));
        assert!(result.is_err());
    }

    #[test]
    fn test_resource_type_from_extension() {
        assert_eq!(ResourceType::from_extension("pdf"), ResourceType::Pdf);
        assert_eq!(ResourceType::from_extension("PDF"), ResourceType::Pdf);
        assert_eq!(ResourceType::from_extension("md"), ResourceType::WebMarkdown);
        assert_eq!(ResourceType::from_extension("jpg"), ResourceType::Jpg);
        assert_eq!(ResourceType::from_extension("unknown"), ResourceType::Other);
    }
}
