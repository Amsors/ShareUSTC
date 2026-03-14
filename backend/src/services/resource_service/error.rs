//! 资源服务错误类型定义

use crate::services::{file_service::FileError, storage_service::StorageError};

#[derive(Debug)]
pub enum ResourceError {
    DatabaseError(String),
    FileError(String),
    NotFound(String),
    ValidationError(String),
    Unauthorized(String),
    AiError(String),
    Conflict(String), // 资源冲突（如：资源已存在）
}

impl std::fmt::Display for ResourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceError::DatabaseError(msg) => write!(f, "数据库错误: {}", msg),
            ResourceError::FileError(msg) => write!(f, "文件错误: {}", msg),
            ResourceError::NotFound(msg) => write!(f, "未找到: {}", msg),
            ResourceError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
            ResourceError::Unauthorized(msg) => write!(f, "未授权: {}", msg),
            ResourceError::AiError(msg) => write!(f, "AI 错误: {}", msg),
            ResourceError::Conflict(msg) => write!(f, "资源冲突: {}", msg),
        }
    }
}

impl std::error::Error for ResourceError {}

impl From<FileError> for ResourceError {
    fn from(err: FileError) -> Self {
        match err {
            FileError::ValidationError(msg) => ResourceError::ValidationError(msg),
        }
    }
}

impl From<StorageError> for ResourceError {
    fn from(err: StorageError) -> Self {
        match err {
            StorageError::Validation(msg) => ResourceError::ValidationError(msg),
            StorageError::Config(msg) => ResourceError::FileError(msg),
            StorageError::NotFound(msg) => ResourceError::NotFound(msg),
            StorageError::Io(msg) => ResourceError::FileError(msg),
            StorageError::Backend(msg) => ResourceError::FileError(msg),
        }
    }
}

impl From<sqlx::Error> for ResourceError {
    fn from(err: sqlx::Error) -> Self {
        ResourceError::DatabaseError(err.to_string())
    }
}
