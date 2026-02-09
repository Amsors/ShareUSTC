use std::env;

/// 应用配置结构体
#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub server_host: String,
    pub server_port: u16,
    pub log_level: String,
    pub image_upload_path: String,
    pub resource_upload_path: String,
    pub cors_allowed_origins: Vec<String>,
}

impl Config {
    /// 从环境变量加载配置
    pub fn from_env() -> Self {
        // 解析 CORS 允许的域名列表
        let cors_origins = env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| {
                "http://localhost:5173,http://127.0.0.1:5173".to_string()
            })
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/shareustc".to_string()),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "your-secret-key".to_string()),
            server_host: env::var("SERVER_HOST")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
            server_port: env::var("SERVER_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            log_level: env::var("RUST_LOG")
                .unwrap_or_else(|_| "backend=debug,actix_web=info,sqlx=warn".to_string()),
            image_upload_path: env::var("IMAGE_UPLOAD_PATH")
                .unwrap_or_else(|_| "./uploads/images".to_string()),
            resource_upload_path: env::var("RESOURCE_UPLOAD_PATH")
                .unwrap_or_else(|_| "./uploads/resources".to_string()),
            cors_allowed_origins: cors_origins,
        }
    }
}
