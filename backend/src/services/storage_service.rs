use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;

use tokio::fs;

use crate::config::Config;

use super::oss_service::OssStorage;

pub type StorageFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, StorageError>> + Send + 'a>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageBackendType {
    Local,
    Oss,
}

impl StorageBackendType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Oss => "oss",
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("参数错误: {0}")]
    Validation(String),
    #[error("配置错误: {0}")]
    Config(String),
    #[error("未找到: {0}")]
    NotFound(String),
    #[error("IO 错误: {0}")]
    Io(String),
    #[error("后端错误: {0}")]
    Backend(String),
}

impl actix_web::ResponseError for StorageError {
    fn error_response(&self) -> actix_web::HttpResponse {
        use crate::utils::{bad_request, internal_error, not_found};
        match self {
            StorageError::Validation(msg) => bad_request(msg),
            StorageError::NotFound(msg) => not_found(msg),
            StorageError::Config(msg) => {
                log::error!("[Storage] 配置错误 | error={}", msg);
                internal_error("服务器内部错误")
            }
            StorageError::Io(msg) => {
                log::error!("[Storage] IO 错误 | error={}", msg);
                internal_error("服务器内部错误")
            }
            StorageError::Backend(msg) => {
                log::error!("[Storage] 后端错误 | error={}", msg);
                internal_error("服务器内部错误")
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct StorageFileMetadata {
    pub content_length: Option<u64>,
    pub content_type: Option<String>,
    #[allow(dead_code)]
    pub etag: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StorageStsCredentials {
    pub access_key_id: String,
    pub access_key_secret: String,
    pub security_token: String,
    pub expiration: String,
    pub bucket: String,
    pub region: String,
    pub endpoint: String,
    pub upload_key: String,
    pub expires_in: u64,
}

pub trait StorageBackend: Send + Sync {
    fn save_file<'a>(
        &'a self,
        key: &'a str,
        data: Vec<u8>,
        content_type: Option<&'a str>,
    ) -> StorageFuture<'a, String>;

    fn read_file<'a>(&'a self, key: &'a str) -> StorageFuture<'a, Vec<u8>>;

    fn write_file<'a>(
        &'a self,
        key: &'a str,
        data: Vec<u8>,
        content_type: Option<&'a str>,
    ) -> StorageFuture<'a, ()>;

    fn delete_file<'a>(&'a self, key: &'a str) -> StorageFuture<'a, ()>;

    fn get_file_url<'a>(&'a self, key: &'a str, expires_secs: u64) -> StorageFuture<'a, String>;

    fn get_download_url<'a>(
        &'a self,
        key: &'a str,
        filename: &'a str,
        expires_secs: u64,
    ) -> StorageFuture<'a, String>;

    fn get_upload_url<'a>(
        &'a self,
        key: &'a str,
        expires_secs: u64,
        content_type: Option<&'a str>,
    ) -> StorageFuture<'a, String>;

    fn head_file<'a>(&'a self, key: &'a str) -> StorageFuture<'a, StorageFileMetadata>;

    fn get_sts_token<'a>(
        &'a self,
        _key: &'a str,
        _duration_secs: u64,
    ) -> StorageFuture<'a, StorageStsCredentials> {
        Box::pin(async move {
            Err(StorageError::Backend(
                "当前存储后端不支持 STS 临时凭证".to_string(),
            ))
        })
    }

    fn backend_type(&self) -> StorageBackendType;

    fn supports_sts(&self) -> bool {
        false
    }

    fn default_signed_url_expiry(&self) -> u64 {
        600
    }
}

#[derive(Debug, Clone)]
pub struct LocalStorage {
    base_path: PathBuf,
    base_url: String,
}

impl LocalStorage {
    pub fn new(base_path: String, base_url: String) -> Self {
        Self {
            base_path: PathBuf::from(base_path),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    fn resolve_local_path(&self, key_or_path: &str) -> Result<PathBuf, StorageError> {
        if key_or_path.trim().is_empty() {
            return Err(StorageError::Validation("文件 key 不能为空".to_string()));
        }

        // 拒绝包含路径遍历字符的 key
        if key_or_path.contains("..") || key_or_path.contains("//") {
            return Err(StorageError::Validation(
                "文件 key 包含非法字符".to_string(),
            ));
        }

        let path = Path::new(key_or_path);

        // 尝试将 base_path 转换为绝对路径（尽可能 canonicalize）
        let base_absolute = self.base_path.canonicalize().unwrap_or_else(|_| {
            // 如果无法 canonicalize（目录不存在），使用绝对路径
            if self.base_path.is_absolute() {
                self.base_path.clone()
            } else {
                // 将相对路径转换为绝对路径
                std::env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join(&self.base_path)
            }
        });

        // 如果传入的是绝对路径，直接使用
        if path.is_absolute() {
            // 使用 canonicalize 确保路径在允许的范围内
            let canonical_path = path
                .canonicalize()
                .map_err(|e| StorageError::Validation(format!("无法解析文件路径: {}", e)))?;

            // 确保最终路径在基础目录内，防止路径遍历攻击
            if !canonical_path.starts_with(&base_absolute) {
                return Err(StorageError::Validation(
                    "文件路径超出允许的范围".to_string(),
                ));
            }

            return Ok(canonical_path);
        }

        // 处理相对路径：去掉开头的 "./" 以便正确拼接
        let key = key_or_path.trim_start_matches('/').trim_start_matches("./");

        // 检查路径是否已经包含 base_path 的前缀（处理存储时返回的完整路径）
        let path_str = key_or_path;
        let base_str = self.base_path.to_string_lossy();

        // 如果路径以 base_path 开头（如 ./uploads/resources/xxx.pdf），提取相对部分
        let relative_key = if path_str.starts_with(base_str.as_ref()) {
            path_str[base_str.len()..]
                .trim_start_matches('/')
                .trim_start_matches("./")
        } else {
            key
        };

        let full_path = base_absolute.join(relative_key);

        // 再次检查拼接后的路径是否在基础目录内
        // 使用 canonicalize 解析任何可能的符号链接或相对路径
        match full_path.canonicalize() {
            Ok(canonical) => {
                if !canonical.starts_with(&base_absolute) {
                    return Err(StorageError::Validation(
                        "文件路径超出允许的范围".to_string(),
                    ));
                }
                Ok(canonical)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // 文件不存在时无法 canonicalize，手动检查路径
                // 检查规范化后的路径是否以基础目录开头
                if !full_path.starts_with(&base_absolute) {
                    return Err(StorageError::Validation(
                        "文件路径超出允许的范围".to_string(),
                    ));
                }

                Ok(full_path)
            }
            Err(e) => Err(StorageError::Io(format!("解析文件路径失败: {}", e))),
        }
    }

    fn relative_key(&self, key_or_path: &str) -> String {
        let path = Path::new(key_or_path);

        // 尝试 strip_prefix，处理绝对路径和相对路径
        if let Ok(relative) = path.strip_prefix(&self.base_path) {
            return relative.to_string_lossy().replace('\\', "/");
        }

        // 处理存储的相对路径（如 ./uploads/resources/xxx.pdf）
        let base_str = self.base_path.to_string_lossy();
        if key_or_path.starts_with(base_str.as_ref()) {
            let relative = &key_or_path[base_str.len()..];
            return relative
                .trim_start_matches('/')
                .trim_start_matches("./")
                .to_string();
        }

        // 处理 ./ 开头的路径
        key_or_path
            .trim_start_matches('/')
            .trim_start_matches("./")
            .to_string()
    }
}

impl StorageBackend for LocalStorage {
    fn save_file<'a>(
        &'a self,
        key: &'a str,
        data: Vec<u8>,
        _content_type: Option<&'a str>,
    ) -> StorageFuture<'a, String> {
        Box::pin(async move {
            let full_path = self.resolve_local_path(key)?;

            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)
                    .await
                    .map_err(|e| StorageError::Io(format!("创建目录失败: {}", e)))?;
            }

            fs::write(&full_path, &data)
                .await
                .map_err(|e| StorageError::Io(format!("写入文件失败: {}", e)))?;

            Ok(full_path.to_string_lossy().to_string())
        })
    }

    fn read_file<'a>(&'a self, key: &'a str) -> StorageFuture<'a, Vec<u8>> {
        Box::pin(async move {
            let full_path = self.resolve_local_path(key)?;

            match fs::read(&full_path).await {
                Ok(data) => Ok(data),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(StorageError::NotFound(
                    format!("文件不存在: {}", full_path.to_string_lossy()),
                )),
                Err(e) => Err(StorageError::Io(format!("读取文件失败: {}", e))),
            }
        })
    }

    fn write_file<'a>(
        &'a self,
        key: &'a str,
        data: Vec<u8>,
        _content_type: Option<&'a str>,
    ) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            let full_path = self.resolve_local_path(key)?;

            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)
                    .await
                    .map_err(|e| StorageError::Io(format!("创建目录失败: {}", e)))?;
            }

            fs::write(&full_path, &data)
                .await
                .map_err(|e| StorageError::Io(format!("写入文件失败: {}", e)))?;

            Ok(())
        })
    }

    fn delete_file<'a>(&'a self, key: &'a str) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            let full_path = self.resolve_local_path(key)?;

            match fs::remove_file(&full_path).await {
                Ok(_) => Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(e) => Err(StorageError::Io(format!("删除文件失败: {}", e))),
            }
        })
    }

    fn get_file_url<'a>(&'a self, key: &'a str, expires_secs: u64) -> StorageFuture<'a, String> {
        // 注意：本地存储不使用 expires_secs 参数，但保持签名一致
        // expires_secs 仅对 OSS 签名 URL 有效
        let _ = expires_secs;
        Box::pin(async move {
            let relative = self.relative_key(key);
            if relative.is_empty() {
                return Err(StorageError::Validation("文件 key 不能为空".to_string()));
            }

            Ok(format!("{}/{}", self.base_url, relative))
        })
    }

    fn get_download_url<'a>(
        &'a self,
        key: &'a str,
        _filename: &'a str,
        expires_secs: u64,
    ) -> StorageFuture<'a, String> {
        self.get_file_url(key, expires_secs)
    }

    fn get_upload_url<'a>(
        &'a self,
        _key: &'a str,
        _expires_secs: u64,
        _content_type: Option<&'a str>,
    ) -> StorageFuture<'a, String> {
        Box::pin(async move {
            Err(StorageError::Backend(
                "当前存储后端不支持直传 URL".to_string(),
            ))
        })
    }

    fn head_file<'a>(&'a self, key: &'a str) -> StorageFuture<'a, StorageFileMetadata> {
        Box::pin(async move {
            let full_path = self.resolve_local_path(key)?;
            let metadata = fs::metadata(&full_path).await.map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    StorageError::NotFound(format!("文件不存在: {}", full_path.to_string_lossy()))
                } else {
                    StorageError::Io(format!("读取文件元信息失败: {}", e))
                }
            })?;

            Ok(StorageFileMetadata {
                content_length: Some(metadata.len()),
                content_type: None,
                etag: None,
            })
        })
    }

    fn backend_type(&self) -> StorageBackendType {
        StorageBackendType::Local
    }
}

pub fn create_storage_backend(config: &Config) -> Result<Arc<dyn StorageBackend>, StorageError> {
    if config.storage_backend == "oss" {
        let storage = OssStorage::from_config(config)?;
        return Ok(Arc::new(storage));
    }

    Ok(Arc::new(LocalStorage::new(
        config.file_upload_path.clone(),
        config.image_base_url.clone(),
    )))
}

/// 创建一个本地存储实例，用于在OSS模式下读取本地文件
pub fn create_local_storage(config: &Config) -> Result<Arc<dyn StorageBackend>, StorageError> {
    Ok(Arc::new(LocalStorage::new(
        config.file_upload_path.clone(),
        config.image_base_url.clone(),
    )))
}
