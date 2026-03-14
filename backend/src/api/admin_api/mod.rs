pub mod audit_logs;
pub mod comments;
pub mod courses;
pub mod dashboard;
pub mod favorites;
pub mod notifications;
pub mod resources;
pub mod teachers;
pub mod users;
pub mod utils;

use actix_web::web;

use crate::models::CurrentUser;
use crate::services::AdminError;

/// 检查用户是否是管理员
pub fn check_admin(current_user: &CurrentUser) -> Result<(), AdminError> {
    if !matches!(current_user.role, crate::models::UserRole::Admin) {
        return Err(AdminError::Forbidden("需要管理员权限".to_string()));
    }
    Ok(())
}

/// 配置管理后台路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.configure(dashboard::config)
        .configure(users::config)
        .configure(resources::config)
        .configure(comments::config)
        .configure(notifications::config)
        .configure(audit_logs::config)
        .configure(teachers::config)
        .configure(courses::config)
        .configure(favorites::config);
}
