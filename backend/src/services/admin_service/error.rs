/// 管理员服务错误类型
#[derive(Debug, thiserror::Error)]
pub enum AdminError {
    #[error("未找到: {0}")]
    NotFound(String),
    #[error("验证错误: {0}")]
    ValidationError(String),
    #[error("权限不足: {0}")]
    Forbidden(String),
    #[error("内部错误: {0}")]
    Internal(String),
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),
}

impl actix_web::ResponseError for AdminError {
    fn error_response(&self) -> actix_web::HttpResponse {
        use crate::utils::{bad_request, forbidden, internal_error, not_found};
        match self {
            AdminError::NotFound(msg) => not_found(msg),
            AdminError::ValidationError(msg) => bad_request(msg),
            AdminError::Forbidden(msg) => forbidden(msg),
            AdminError::Internal(msg) => {
                log::error!("[Admin] 内部错误 | error={}", msg);
                internal_error("服务器内部错误")
            }
            AdminError::Database(e) => {
                log::error!("[Admin] 数据库错误 | error={}", e);
                internal_error("服务器内部错误")
            }
        }
    }
}
