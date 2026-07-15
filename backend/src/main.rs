use actix_cors::Cors;
use actix_web::{
    get, http::Method, middleware::Logger, web, App, HttpResponse, HttpServer, Responder,
};
use serde::Serialize;
use uuid::Uuid;

use backend::api;
use backend::config::{BrandConfig, Config};
use backend::db::{self, AppState};
use backend::middleware::{JwtAuth, PublicPathRule};
use backend::services::{self, StorageBackendType};
use backend::tasks;
use backend::utils::{error_response, internal_error, not_found};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HelloResponse {
    message: String,
    status: String,
}

#[get("/api/hello")]
async fn hello(data: web::Data<AppState>) -> impl Responder {
    // 测试数据库连接
    let result: Result<(i32,), sqlx::Error> =
        sqlx::query_as("SELECT 1").fetch_one(&data.pool).await;

    let db_status = match result {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    HttpResponse::Ok().json(HelloResponse {
        message: format!("Hello from Rust backend! DB: {}", db_status),
        status: "ok".to_string(),
    })
}

/// Liveness 探针：仅表示进程存活，不探测依赖。恒定返回 200。
#[get("/health")]
async fn health_check(brand: web::Data<BrandConfig>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": brand.service_name
    }))
}

/// Readiness 探针：探测数据库可用性（`SELECT 1`）。
/// 就绪返回 200，数据库不可达返回 503（错误体遵循 api_design.md 的 `{error, message}`）。
/// compose / K8s 的 healthcheck 指向本端点。
#[get("/health/ready")]
async fn readiness_check(
    pool: web::Data<sqlx::PgPool>,
    brand: web::Data<BrandConfig>,
) -> impl Responder {
    match services::HealthService::check_readiness(pool.get_ref()).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "status": "ready",
            "service": brand.service_name
        })),
        Err(e) => {
            log::warn!("[System] readiness 探测失败：数据库不可达 | error={}", e);
            error_response(503, "数据库不可达")
        }
    }
}

/// 构建 JWT 中间件的公开路径规则。
fn public_path_rules() -> Vec<PublicPathRule> {
    vec![
        // 存活与就绪探针供容器编排系统访问，不携带用户认证信息
        PublicPathRule::with_methods("/api/health", vec![Method::GET]),
        // /api/auth 全部公开
        PublicPathRule::all_methods("/api/auth"),
        // /api/resources GET 方法公开（列表、搜索、详情、下载），但排除需要登录的接口
        PublicPathRule::with_methods("/api/resources", vec![Method::GET])
            .exclude(vec!["/api/resources/my", "/api/resources/{id}/rate"]),
        // /api/resources/pdf-preview-challenge 全部公开（支持未登录用户检测）
        PublicPathRule::all_methods("/api/resources/pdf-preview-challenge"),
        // /api/users/{user_id} 和 /api/users/{user_id}/homepage GET 方法公开
        // 排除 /api/users/me 和 /api/users/verify
        PublicPathRule::with_methods("/api/users", vec![Method::GET])
            .exclude(vec!["/api/users/me", "/api/users/verify"]),
        // /api/config 公开（站点配置）
        PublicPathRule::with_methods("/api/config", vec![Method::GET]),
        // /api/teachers 和 /api/courses GET 方法公开（供游客筛选资源）
        PublicPathRule::with_methods("/api/teachers", vec![Method::GET]),
        PublicPathRule::with_methods("/api/courses", vec![Method::GET]),
    ]
}

/// 获取图片文件（公开访问）
/// 使用后端代理模式读取文件，避免浏览器直接访问 OSS 产生 CORS 问题
#[get("/images/{image_id}")]
async fn serve_image(data: web::Data<AppState>, path: web::Path<Uuid>) -> impl Responder {
    let image_id = path.into_inner();

    // 从数据库获取图片路径和存储类型
    match services::ImageService::get_image_path(&data.pool, image_id).await {
        Ok((file_path, mime_type, storage_type)) => {
            // 根据图片实际的存储类型选择正确的存储后端读取文件
            // 使用后端代理模式，避免浏览器直接访问 OSS 产生 CORS 问题
            let is_oss = storage_type.as_deref() == Some("oss");

            let read_result = if is_oss {
                // OSS 存储：使用主 storage（如果是 OSS 模式）或创建 OSS 存储实例
                if data.storage.backend_type() == StorageBackendType::Oss {
                    data.storage.read_file(&file_path).await
                } else {
                    // 当前是 local 模式，但需要读取 OSS 文件（使用注入的配置，不再每请求解析环境变量）
                    match services::create_storage_backend(&data.config) {
                        Ok(oss_storage)
                            if oss_storage.backend_type() == StorageBackendType::Oss =>
                        {
                            oss_storage.read_file(&file_path).await
                        }
                        _ => {
                            log::warn!(
                                "[Image] 无法创建 OSS 存储实例来读取图片 | image_id={}",
                                image_id
                            );
                            return internal_error("无法读取 OSS 图片");
                        }
                    }
                }
            } else {
                // 本地存储：使用主 storage（如果是 Local 模式）或创建本地存储实例
                if data.storage.backend_type() == StorageBackendType::Local {
                    data.storage.read_file(&file_path).await
                } else {
                    // 当前是 OSS 模式，但需要读取本地文件（使用注入的配置）
                    match services::create_local_storage(&data.config) {
                        Ok(local_storage) => local_storage.read_file(&file_path).await,
                        Err(e) => {
                            log::error!("[Image] 创建本地存储失败 | error={}", e);
                            return internal_error("无法访问本地存储");
                        }
                    }
                }
            };

            match read_result {
                Ok(file_content) => {
                    // 根据MIME类型设置Content-Type
                    let content_type = mime_type
                        .and_then(|m| m.parse::<mime::Mime>().ok())
                        .unwrap_or(mime::APPLICATION_OCTET_STREAM);

                    HttpResponse::Ok()
                        .content_type(content_type)
                        .body(file_content)
                }
                Err(e) => {
                    log::warn!(
                        "[Image] 读取图片文件失败 | image_id={}, path={}, storage={}, error={}",
                        image_id,
                        file_path,
                        if is_oss { "oss" } else { "local" },
                        e
                    );
                    not_found("图片不存在")
                }
            }
        }
        Err(e) => {
            log::warn!(
                "[Image] 获取图片路径失败 | image_id={}, error={}",
                image_id,
                e
            );
            not_found("图片不存在")
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 加载环境变量
    dotenvy::dotenv().ok();

    // 先初始化日志系统，确保 Config::from_env() 的必填项校验失败信息可见。
    // 未设置 RUST_LOG 时使用与开发环境一致的默认过滤器。
    const DEFAULT_LOG_FILTER: &str = "backend=debug,actix_web=info,sqlx=warn";
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(DEFAULT_LOG_FILTER))
        .init();

    // 加载配置（DATABASE_URL / JWT_SECRET / IMAGE_BASE_URL 等关键项缺失时会在此打印错误并退出）
    let config = Config::from_env();

    // 构建服务器地址
    let server_addr = format!("{}:{}", config.server_host, config.server_port);

    // 确保上传目录存在（子目录由上传根路径派生）
    std::fs::create_dir_all(config.image_upload_path()).unwrap_or_else(|e| {
        log::warn!("[System] 创建图片上传目录失败 | error={}", e);
    });
    std::fs::create_dir_all(config.resource_upload_path()).unwrap_or_else(|e| {
        log::warn!("[System] 创建资源上传目录失败 | error={}", e);
    });

    log::info!("[System] Starting {} server...", config.brand.service_name);
    log::info!("[System] Server address: http://{}", server_addr);
    log::info!(
        "[System] Image upload directory: {}",
        config.image_upload_path()
    );
    log::info!(
        "[System] Resource upload directory: {}",
        config.resource_upload_path()
    );

    // 创建数据库连接池
    let pool = match db::create_pool(&config.database_url).await {
        Ok(pool) => {
            log::info!("[System] 数据库连接池创建成功");
            pool
        }
        Err(e) => {
            log::error!("[System] 数据库连接失败 | error={}", e);
            log::warn!("[System] 请检查 DATABASE_URL 环境变量是否正确设置");
            log::warn!(
                "[System] 示例: DATABASE_URL=postgres://username:password@localhost:5432/shareustc"
            );
            std::process::exit(1);
        }
    };

    // 执行数据库迁移（backend/migrations/）
    // 迁移采用 IF NOT EXISTS 幂等写法：新库建全量 schema，存量库重复执行无副作用
    match sqlx::migrate!().run(&pool).await {
        Ok(()) => log::info!("[System] 数据库迁移完成"),
        Err(e) => {
            log::error!("[System] 数据库迁移失败 | error={}", e);
            std::process::exit(1);
        }
    }

    // 同步管理员权限（根据环境变量配置）
    if !config.admin_usernames.is_empty() {
        log::info!(
            "[Admin] 正在同步管理员权限 | admins={:?}",
            config.admin_usernames
        );
        match services::AdminService::sync_admin_roles(&pool, &config.admin_usernames).await {
            Ok((granted, revoked)) => {
                log::info!(
                    "[Admin] 管理员权限同步完成 | granted={}, revoked={}",
                    granted,
                    revoked
                );
            }
            Err(e) => {
                log::warn!("[Admin] 管理员权限同步失败 | error={}", e);
            }
        }
    } else {
        log::info!("[Admin] 未配置管理员用户名列表 (ADMIN_USERNAMES)，跳过权限同步");
    }

    // 初始化用户 sn（为没有 sn 的用户分配编号）
    log::info!("[System] 正在初始化用户 sn...");
    match initialize_user_sn(&pool).await {
        Ok(count) => {
            if count > 0 {
                log::info!("[System] 已为 {} 个用户分配 sn", count);
            } else {
                log::info!("[System] 所有用户都已分配 sn");
            }
        }
        Err(e) => {
            log::warn!("[System] 初始化用户 sn 失败 | error={}", e);
        }
    }

    // 初始化存储后端
    let storage = match services::create_storage_backend(&config) {
        Ok(storage) => storage,
        Err(e) => {
            log::error!("[System] 初始化存储后端失败 | error={}", e);
            std::process::exit(1);
        }
    };
    log::info!(
        "[System] Storage backend: {}",
        storage.backend_type().as_str()
    );

    // 创建应用状态
    let app_state = web::Data::new(AppState::new(
        pool.clone(),
        config.clone(),
        config.jwt_secret.clone(),
        config.cookie_secure,
        storage.clone(),
        config.require_email_on_register,
        config.allow_username_change,
        config.allow_email_change,
        config.brand.clone(),
        config.pdf_preview_challenge_uuid.clone(),
        config.pdf_preview_challenge_code.clone(),
    ));

    // 启动文件哈希计算后台任务（注入配置，避免任务内重复解析环境变量）
    tasks::file_hash_task::start_file_hash_task(pool.clone(), storage.clone(), config.clone())
        .await;

    // 启动孤立文件扫描后台任务（注入配置）
    tasks::orphan_file_task::start_orphan_file_task(pool.clone(), storage.clone(), config.clone())
        .await;

    log::info!("[System] Server starting at http://{}", server_addr);
    log::debug!("[System] Debug logging enabled");
    log::debug!("[System] API endpoints:");
    log::debug!("[System]   POST /api/auth/register - 用户注册");
    log::debug!("[System]   POST /api/auth/login    - 用户登录");
    log::debug!("[System]   POST /api/auth/refresh  - 刷新Token");
    log::debug!("[System]   POST /api/auth/logout   - 用户登出");
    log::debug!("[System]   GET  /api/users/me      - 获取当前用户");
    log::debug!("[System]   PUT  /api/users/me      - 更新用户资料");
    log::debug!("[System]   POST /api/users/verify  - 实名认证");
    log::debug!("[System]   GET  /api/users/{{user_id}} - 获取用户资料");
    log::debug!("[System]   POST /api/images/upload - 上传图片");
    log::debug!("[System]   GET  /api/images        - 获取我的图片列表");
    log::debug!("[System]   GET  /api/images/{{id}}   - 获取图片信息");
    log::debug!("[System]   DEL  /api/images/{{id}}   - 删除图片");
    log::debug!("[System]   GET  /images/{{id}}       - 访问图片文件（公开）");
    log::debug!("[System]   POST /api/resources     - 上传资源");
    log::debug!("[System]   GET  /api/resources     - 获取资源列表");
    log::debug!("[System]   GET  /api/resources/search - 搜索资源");
    log::debug!("[System]   GET  /api/resources/my  - 获取我的资源列表");
    log::debug!("[System]   GET  /api/resources/{{id}} - 获取资源详情");
    log::debug!("[System]   GET  /api/resources/{{id}}/download - 下载资源");
    log::debug!("[System]   DEL  /api/resources/{{id}} - 删除资源");
    log::debug!("[System]   POST /api/favorites     - 创建收藏夹");
    log::debug!("[System]   GET  /api/favorites     - 获取我的收藏夹列表");
    log::debug!("[System]   GET  /api/favorites/{{id}} - 获取收藏夹详情");
    log::debug!("[System]   PUT  /api/favorites/{{id}} - 更新收藏夹");
    log::debug!("[System]   DEL  /api/favorites/{{id}} - 删除收藏夹");
    log::debug!("[System]   POST /api/favorites/{{id}}/resources - 添加资源到收藏夹");
    log::debug!("[System]   DEL  /api/favorites/{{id}}/resources/{{rid}} - 从收藏夹移除资源");
    log::debug!("[System]   GET  /api/favorites/check/{{rid}} - 检查资源收藏状态");
    log::debug!("[System]   GET  /api/health        - 健康检查");
    log::debug!("[System]   GET  /api/hello         - 测试接口");

    // 克隆配置数据用于闭包
    let jwt_secret = config.jwt_secret.clone();
    let cors_origins = config.cors_allowed_origins.clone();

    // 记录 CORS 配置信息
    log::info!("[System] CORS allowed origins: {:?}", cors_origins);

    HttpServer::new(move || {
        // 克隆 CORS 域名列表供此 worker 线程使用
        let cors_origins_worker = cors_origins.clone();

        let jwt_auth = JwtAuth::new(jwt_secret.clone()).with_public_rules(public_path_rules());

        // 构建 CORS 配置
        // 注意：使用 Cookie 认证必须设置 supports_credentials(true)
        let cors = Cors::default()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec!["Content-Type", "Authorization", "Accept"])
            .expose_headers(vec!["Content-Disposition"])
            .supports_credentials() // 必须启用，以支持 Cookie 传输
            .max_age(3600);

        // 动态添加允许的域名
        // 注意：使用 supports_credentials() 时，不能同时使用 allow_any_origin()
        // 必须指定具体的允许域名
        let cors = if cors_origins_worker.contains(&"*".to_string()) {
            // 允许任何来源，但需要验证 origin 头部（用于 Cookie 认证）
            cors.allowed_origin_fn(|_origin, _req_head| true)
        } else {
            cors.allowed_origin_fn(move |origin, _req_head| {
                let origin_str = origin.to_str().unwrap_or("");
                cors_origins_worker.iter().any(|allowed| {
                    if allowed.ends_with('/') {
                        origin_str.starts_with(&allowed[..allowed.len() - 1])
                    } else {
                        origin_str == allowed || origin_str.starts_with(&format!("{}/", allowed))
                    }
                })
            })
        };

        App::new()
            .app_data(app_state.clone())
            .app_data(web::Data::new(app_state.pool.clone()))
            .app_data(web::Data::new(app_state.brand.clone()))
            .wrap(cors)
            .wrap(Logger::new("%a %r %s %b %Dms").log_target("backend::access"))
            // API 路由（统一使用 /api 前缀，通过中间件控制认证）
            // 注意：config 必须在 config_public 之前注册，否则 /resources/my 会被 /resources/{id} 匹配
            .service(
                web::scope("/api")
                    .wrap(jwt_auth)
                    .service(health_check)
                    .service(readiness_check)
                    .configure(api::auth::config)
                    .configure(api::user::config)
                    .configure(api::oss::config)
                    .configure(api::image_host::config)
                    .configure(api::comment::config) // 评论路由
                    .configure(api::notification::config) // 通知路由
                    .configure(api::admin_api::config) // 管理后台路由
                    .configure(api::favorite::config) // 收藏夹路由
                    .configure(api::teacher::config) // 教师路由（公开）
                    .configure(api::course::config) // 课程路由（公开）
                    .configure(api::resource_api::config) // 需要认证的资源路由（先注册）
                    .configure(api::resource_api::config_public), // 公开资源路由（后注册）
            )
            // 独立的公开服务（非 /api 前缀）
            .service(serve_image)
            .service(hello)
    })
    .bind(&server_addr)?
    // 显式优雅停机超时：docker stop（SIGTERM）后给在途请求（尤其大文件下载）最多 30s 完成
    .shutdown_timeout(30)
    .run()
    .await
}

/// 初始化用户 sn
/// 为没有 sn 的用户按创建时间顺序分配 sn
async fn initialize_user_sn(pool: &sqlx::PgPool) -> Result<usize, sqlx::Error> {
    // 确保序列存在（从1开始）
    sqlx::query("CREATE SEQUENCE IF NOT EXISTS user_sn_seq START 1")
        .execute(pool)
        .await
        .ok();

    // 获取当前最大的 sn 值
    let max_sn: Option<i64> = sqlx::query_scalar("SELECT MAX(sn) FROM users")
        .fetch_one(pool)
        .await?;

    // 如果有用户已有 sn，将序列设置为该值，这样 nextval 会从下一个开始
    if let Some(max) = max_sn {
        sqlx::query("SELECT setval('user_sn_seq', $1, true)")
            .bind(max)
            .fetch_optional(pool)
            .await
            .ok();
    }

    // 获取没有 sn 的用户列表
    let rows: Vec<(uuid::Uuid,)> =
        sqlx::query_as("SELECT id FROM users WHERE sn IS NULL ORDER BY created_at ASC")
            .fetch_all(pool)
            .await?;

    let count = rows.len();
    if count == 0 {
        return Ok(0);
    }

    // 为每个没有 sn 的用户分配 sn
    let mut assigned = 0;
    for (user_id,) in rows {
        let result = sqlx::query(
            "UPDATE users SET sn = nextval('user_sn_seq') WHERE id = $1 AND sn IS NULL",
        )
        .bind(user_id)
        .execute(pool)
        .await;

        if result.is_ok() {
            assigned += 1;
        }
    }

    Ok(assigned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http::StatusCode, test as actix_test, App};
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;

    #[test]
    fn readiness_probe_is_public_for_get_requests() {
        let rules = public_path_rules();
        let is_public = rules
            .iter()
            .any(|rule| rule.matches("/api/health/ready", &Method::GET));
        let post_is_public = rules
            .iter()
            .any(|rule| rule.matches("/api/health/ready", &Method::POST));

        assert!(is_public, "readiness 探针不应要求认证信息");
        assert!(!post_is_public, "健康检查路径只应公开 GET 方法");
    }

    #[actix_web::test]
    async fn readiness_probe_without_auth_reaches_handler() {
        let pool = PgPoolOptions::new()
            .acquire_timeout(Duration::from_millis(100))
            .connect_lazy("postgres://test:test@127.0.0.1:1/test")
            .expect("测试数据库连接串应可解析");
        let jwt_auth =
            JwtAuth::new("test-jwt-secret".to_string()).with_public_rules(public_path_rules());
        let app = actix_test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .app_data(web::Data::new(BrandConfig::default()))
                .service(
                    web::scope("/api")
                        .wrap(jwt_auth)
                        .service(health_check)
                        .service(readiness_check),
                ),
        )
        .await;

        let response = actix_test::call_service(
            &app,
            actix_test::TestRequest::get()
                .uri("/api/health/ready")
                .to_request(),
        )
        .await;

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        let body: serde_json::Value = actix_test::read_body_json(response).await;
        assert_eq!(body["error"], "ServiceUnavailable");
        assert_eq!(body["message"], "数据库不可达");
    }
}
