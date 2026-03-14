/// 管理员服务错误类型
#[derive(Debug)]
pub enum AdminError {
    DatabaseError(String),
    NotFound(String),
    ValidationError(String),
    Forbidden(String),
}

impl std::fmt::Display for AdminError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdminError::DatabaseError(msg) => write!(f, "数据库错误: {}", msg),
            AdminError::NotFound(msg) => write!(f, "未找到: {}", msg),
            AdminError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
            AdminError::Forbidden(msg) => write!(f, "权限不足: {}", msg),
        }
    }
}

impl std::error::Error for AdminError {}
