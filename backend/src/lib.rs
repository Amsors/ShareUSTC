//! ShareUSTC 后端共享模块。
//!
//! 二进制入口和集成测试共用同一组业务模块，避免测试重复声明模块树。

pub mod api;
pub mod config;
pub mod db;
pub mod middleware;
pub mod models;
pub mod services;
pub mod tasks;
pub mod utils;
