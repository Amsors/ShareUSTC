//! 资源服务错误类型定义

use crate::services::{file_service::FileError, storage_service::StorageError};
use crate::utils::{bad_request, conflict, forbidden, internal_error, not_found};
use actix_web::{HttpResponse, ResponseError};

#[derive(Debug, thiserror::Error)]
pub enum ResourceError {
    #[error("未找到: {0}")]
    NotFound(String),
    #[error("验证错误: {0}")]
    ValidationError(String),
    #[error("未授权: {0}")]
    Unauthorized(String),
    #[error("资源冲突: {0}")]
    Conflict(String), // 资源冲突（如：资源已存在）
    #[error("文件错误: {0}")]
    FileError(String),
    #[error("AI 错误: {0}")]
    AiError(String),
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),
}

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

impl ResponseError for ResourceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ResourceError::NotFound(msg) => not_found(msg),
            ResourceError::ValidationError(msg) => bad_request(msg),
            ResourceError::Unauthorized(msg) => forbidden(msg),
            ResourceError::Conflict(msg) => conflict(msg),
            // 文件/AI 错误按历史行为把内部信息回传（非 SQL 细节）
            ResourceError::FileError(msg) => {
                log::error!("[Resource] 文件错误 | error={}", msg);
                internal_error(msg)
            }
            ResourceError::AiError(msg) => {
                log::error!("[Resource] AI 错误 | error={}", msg);
                internal_error(msg)
            }
            // 数据库错误一律 500，仅记录日志，响应不含 SQL 细节
            ResourceError::Database(e) => {
                log::error!("[Resource] 数据库错误 | error={}", e);
                internal_error("服务器内部错误")
            }
        }
    }
}
