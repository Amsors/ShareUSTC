use std::env;

/// 品牌配置结构体
#[derive(Clone, Debug)]
pub struct BrandConfig {
    /// 服务名称（用于日志和健康检查）
    pub service_name: String,
}

impl Default for BrandConfig {
    fn default() -> Self {
        Self {
            service_name: "ShareUSTC Backend".to_string(),
        }
    }
}

impl BrandConfig {
    /// 从环境变量加载品牌配置
    pub fn from_env() -> Self {
        Self {
            service_name: env::var("SERVICE_NAME")
                .unwrap_or_else(|_| "ShareUSTC Backend".to_string()),
        }
    }
}

/// 应用配置结构体
#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub server_host: String,
    pub server_port: u16,
    pub cors_allowed_origins: Vec<String>,
    pub admin_usernames: Vec<String>,
    pub cookie_secure: bool,
    pub image_base_url: String,
    pub file_upload_path: String,
    pub storage_backend: String,
    pub oss_access_key_id: Option<String>,
    pub oss_access_key_secret: Option<String>,
    pub oss_endpoint: Option<String>,
    pub oss_bucket: Option<String>,
    pub oss_region: Option<String>,
    pub oss_sts_role_arn: Option<String>,
    pub oss_sts_session_duration: u64,
    pub oss_key_prefix: String,
    pub oss_signed_url_expiry: u64,
    /// 注册时是否强制要求邮箱
    pub require_email_on_register: bool,
    /// 是否允许用户修改用户名
    pub allow_username_change: bool,
    /// 是否允许用户修改邮箱
    pub allow_email_change: bool,
    /// 品牌配置
    pub brand: BrandConfig,
    /// PDF 预览检测资源 UUID
    pub pdf_preview_challenge_uuid: Option<String>,
    /// PDF 预览检测验证码
    pub pdf_preview_challenge_code: Option<String>,
}

impl Config {
    /// 从环境变量加载配置
    pub fn from_env() -> Self {
        let optional_env = |name: &str| {
            env::var(name).ok().and_then(|value| {
                let trimmed = value.trim().to_string();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            })
        };

        // 必填环境变量：缺失或为空时立即退出（容器里漏配应快速失败而非静默用错误默认值）。
        // 注意：调用方需在此之前初始化日志系统，否则错误信息无法输出。
        let required_env = |name: &str| -> String {
            match env::var(name) {
                Ok(value) if !value.trim().is_empty() => value.trim().to_string(),
                _ => {
                    log::error!("[Config] 缺少必填环境变量 {name}，无法启动");
                    log::warn!("[Config] 请在 .env 或部署环境（compose secrets / deploy/.env）中设置 {name}");
                    std::process::exit(1);
                }
            }
        };

        // 解析 CORS 允许的域名列表
        let cors_origins = env::var("CORS_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:5173,http://127.0.0.1:5173".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // 解析管理员用户名列表（逗号分隔）
        let admin_usernames = env::var("ADMIN_USERNAMES")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let storage_backend = match env::var("STORAGE_BACKEND")
            .unwrap_or_else(|_| "local".to_string())
            .to_lowercase()
            .as_str()
        {
            "oss" => "oss".to_string(),
            _ => "local".to_string(),
        };

        // JWT_SECRET：必填且校验强度，拒绝空串、过短与已知占位值
        let jwt_secret = required_env("JWT_SECRET");
        const JWT_SECRET_PLACEHOLDERS: &[&str] = &[
            "your-secret-key",
            "change_me",
            "changeme",
            "secret",
            "your-super-secret-jwt-key-change-this-in-production",
        ];
        if jwt_secret.len() < 16 || JWT_SECRET_PLACEHOLDERS.contains(&jwt_secret.as_str()) {
            log::error!(
                "[Config] JWT_SECRET 过弱或使用了占位值，请设置为长度不少于 16 的高强度随机字符串"
            );
            std::process::exit(1);
        }

        Self {
            // DATABASE_URL：容器中主机名应为 compose 服务名（如 postgres），不能用 localhost
            database_url: required_env("DATABASE_URL"),
            jwt_secret,
            // 默认 0.0.0.0：容器内通常不显式配置，loopback 会导致端口映射后不可访问；
            // 本机开发在 .env 中显式写值不受影响
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: env::var("SERVER_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            cors_allowed_origins: cors_origins,
            admin_usernames,
            cookie_secure: env::var("COOKIE_SECURE")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
            // IMAGE_BASE_URL：图床外链功能依赖，必填。生成的图片/Markdown 链接以此为前缀，
            // 同域反代部署时应设为站点公开 origin（如 https://shareustc.example.com）
            image_base_url: required_env("IMAGE_BASE_URL"),
            file_upload_path: env::var("FILE_UPLOAD_PATH")
                .unwrap_or_else(|_| "./uploads".to_string()),
            storage_backend,
            oss_access_key_id: optional_env("OSS_ACCESS_KEY_ID"),
            oss_access_key_secret: optional_env("OSS_ACCESS_KEY_SECRET"),
            oss_endpoint: optional_env("OSS_ENDPOINT"),
            oss_bucket: optional_env("OSS_BUCKET"),
            oss_region: optional_env("OSS_REGION"),
            oss_sts_role_arn: optional_env("OSS_STS_ROLE_ARN"),
            oss_sts_session_duration: env::var("OSS_STS_SESSION_DURATION")
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(900),
            oss_key_prefix: env::var("OSS_KEY_PREFIX").unwrap_or_default(),
            oss_signed_url_expiry: env::var("OSS_SIGNED_URL_EXPIRY")
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(600),
            // 用户配置项
            require_email_on_register: env::var("REGISTER_REQUIRE_EMAIL")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
            allow_username_change: env::var("ALLOW_USERNAME_CHANGE")
                .map(|v| v != "false" && v != "0")
                .unwrap_or(true),
            allow_email_change: env::var("ALLOW_EMAIL_CHANGE")
                .map(|v| v != "false" && v != "0")
                .unwrap_or(true),
            // 品牌配置
            brand: BrandConfig::from_env(),
            // PDF 预览检测配置
            pdf_preview_challenge_uuid: optional_env("PDF_PREVIEW_CHALLENGE_UUID"),
            pdf_preview_challenge_code: optional_env("PDF_PREVIEW_CHALLENGE_CODE"),
        }
    }

    /// 图片上传子目录（由上传根路径 `file_upload_path` 派生，不再单独配置环境变量）。
    /// 与存储层 key 结构（`images/{uuid}`）一致，容器内单卷即可覆盖全部上传文件。
    pub fn image_upload_path(&self) -> String {
        derive_upload_subdir(&self.file_upload_path, "images")
    }

    /// 资源上传子目录（由上传根路径 `file_upload_path` 派生，不再单独配置环境变量）。
    pub fn resource_upload_path(&self) -> String {
        derive_upload_subdir(&self.file_upload_path, "resources")
    }
}

/// 在上传根路径下派生子目录，统一去除根路径末尾多余的 `/`
fn derive_upload_subdir(root: &str, sub: &str) -> String {
    let trimmed = root.trim_end_matches('/');
    if trimmed.is_empty() {
        sub.to_string()
    } else {
        format!("{trimmed}/{sub}")
    }
}
