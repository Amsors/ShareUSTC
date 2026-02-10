// 服务层模块

pub mod auth_service;
pub mod user_service;
pub mod image_service;
pub mod file_service;
pub mod ai_service;
pub mod resource_service;
pub mod rating_service;
pub mod like_service;
pub mod comment_service;
pub mod notification_service;
pub mod admin_service;
pub mod favorite_service;

pub use auth_service::*;
pub use user_service::*;
pub use image_service::*;
pub use file_service::*;
pub use ai_service::*;
pub use resource_service::*;
pub use rating_service::*;
pub use like_service::*;
pub use comment_service::*;
pub use notification_service::*;
pub use admin_service::*;
pub use favorite_service::*;
