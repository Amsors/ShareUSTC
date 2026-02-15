use actix_cors::Cors;
use actix_web::{http::Method, middleware::Logger, get, web, App, HttpResponse, HttpServer, Responder};
use serde::Serialize;
use uuid::Uuid;

mod config;
mod api;
mod models;
mod utils;
mod services;
mod middleware;
mod db;

use config::Config;
use db::AppState;
use middleware::{JwtAuth, PublicPathRule};
use crate::utils::not_found;

#[derive(Serialize)]
struct HelloResponse {
    message: String,
    status: String,
}

#[get("/api/hello")]
async fn hello(data: web::Data<AppState>) -> impl Responder {
    // 测试数据库连接
    let result: Result<(i32,), sqlx::Error> = sqlx::query_as("SELECT 1")
        .fetch_one(&data.pool)
        .await;

    let db_status = match result {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    HttpResponse::Ok().json(HelloResponse {
        message: format!("Hello from Rust backend! DB: {}", db_status),
        status: "ok".to_string(),
    })
}

#[get("/api/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "ShareUSTC Backend"
    }))
}

/// 获取图片文件（公开访问）
#[get("/images/{image_id}")]
async fn serve_image(
    data: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let image_id = path.into_inner();

    // 从数据库获取图片路径
    match services::ImageService::get_image_path(&data.pool, image_id).await {
        Ok((file_path, mime_type)) => {
            match tokio::fs::read(&file_path).await {
                Ok(file_content) => {
                    // 根据MIME类型设置Content-Type
                    let content_type = mime_type
                        .map(|m| m.parse::<mime::Mime>().ok())
                        .flatten()
                        .unwrap_or(mime::APPLICATION_OCTET_STREAM);

                    HttpResponse::Ok()
                        .content_type(content_type)
                        .body(file_content)
                }
                Err(e) => {
                    log::warn!("[Image] 读取图片文件失败 | image_id={}, path={}, error={}", image_id, file_path, e);
                    not_found("图片文件不存在")
                }
            }
        }
        Err(e) => {
            log::warn!("[Image] 获取图片路径失败 | image_id={}, error={}", image_id, e);
            not_found("图片不存在")
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 加载环境变量
    dotenvy::dotenv().ok();

    // 加载配置
    let config = Config::from_env();

    // 初始化日志系统
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(&config.log_level)
    ).init();

    // 构建服务器地址
    let server_addr = format!("{}:{}", config.server_host, config.server_port);

    // 确保上传目录存在
    std::fs::create_dir_all(&config.image_upload_path).unwrap_or_else(|e| {
        log::warn!("[System] 创建图片上传目录失败 | error={}", e);
    });
    std::fs::create_dir_all(&config.resource_upload_path).unwrap_or_else(|e| {
        log::warn!("[System] 创建资源上传目录失败 | error={}", e);
    });

    log::info!("[System] Starting ShareUSTC backend server...");
    log::info!("[System] Server address: http://{}", server_addr);
    log::info!("[System] Image upload directory: {}", config.image_upload_path);
    log::info!("[System] Resource upload directory: {}", config.resource_upload_path);

    // 创建数据库连接池
    let pool = match db::create_pool(&config.database_url).await {
        Ok(pool) => {
            log::info!("[System] 数据库连接池创建成功");
            pool
        }
        Err(e) => {
            log::error!("[System] 数据库连接失败 | error={}", e);
            log::warn!("[System] 请检查 DATABASE_URL 环境变量是否正确设置");
            log::warn!("[System] 示例: DATABASE_URL=postgres://username:password@localhost:5432/shareustc");
            std::process::exit(1);
        }
    };

    // 同步管理员权限（根据环境变量配置）
    if !config.admin_usernames.is_empty() {
        log::info!("[Admin] 正在同步管理员权限 | admins={:?}", config.admin_usernames);
        match services::AdminService::sync_admin_roles(&pool, &config.admin_usernames).await {
            Ok((granted, revoked)) => {
                log::info!("[Admin] 管理员权限同步完成 | granted={}, revoked={}", granted, revoked);
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

    // 创建应用状态
    let app_state = web::Data::new(AppState::new(
        pool,
        config.jwt_secret.clone(),
        config.cookie_secure,
    ));

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

        // 配置公开路径规则
        let public_rules = vec![
            // /api/auth 全部公开
            PublicPathRule::all_methods("/api/auth"),
            // /api/resources GET 方法公开（列表、搜索、详情、下载），但排除 /api/resources/my
            PublicPathRule::with_methods("/api/resources", vec![Method::GET])
                .exclude(vec!["/api/resources/my"]),
            // /api/users/{user_id} 和 /api/users/{user_id}/homepage GET 方法公开
            // 排除 /api/users/me 和 /api/users/verify
            PublicPathRule::with_methods("/api/users", vec![Method::GET])
                .exclude(vec!["/api/users/me", "/api/users/verify"]),
        ];

        let jwt_auth = JwtAuth::new(jwt_secret.clone()).with_public_rules(public_rules);

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
                        origin_str.starts_with(&allowed[..allowed.len()-1])
                    } else {
                        origin_str == allowed || origin_str.starts_with(&format!("{}/", allowed))
                    }
                })
            })
        };

        App::new()
            .app_data(app_state.clone())
            .wrap(cors)
            .wrap(Logger::new("%a %r %s %b %Dms")
                .log_target("backend::access"))
            // API 路由（统一使用 /api 前缀，通过中间件控制认证）
            // 注意：config 必须在 config_public 之前注册，否则 /resources/my 会被 /resources/{id} 匹配
            .service(
                web::scope("/api")
                    .wrap(jwt_auth)
                    .configure(api::auth::config)
                    .configure(api::user::config)
                    .configure(api::image_host::config)
                    .configure(api::comment::config)          // 评论路由
                    .configure(api::notification::config)     // 通知路由
                    .configure(api::admin::config)            // 管理后台路由
                    .configure(api::favorite::config)         // 收藏夹路由
                    .configure(api::resource::config)          // 需要认证的资源路由（先注册）
                    .configure(api::resource::config_public)  // 公开资源路由（后注册）
            )
            // 独立的公开服务（非 /api 前缀）
            .service(serve_image)
            .service(health_check)
            .service(hello)
    })
        .bind(&server_addr)?
        .run()
        .await
}

/// 初始化用户 sn
/// 为没有 sn 的用户按创建时间顺序分配 sn
async fn initialize_user_sn(pool: &sqlx::PgPool) -> Result<usize, sqlx::Error> {
    // 确保序列存在（从1开始）
    sqlx::query(
        "CREATE SEQUENCE IF NOT EXISTS user_sn_seq START 1"
    )
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
    let rows: Vec<(uuid::Uuid,)> = sqlx::query_as(
        "SELECT id FROM users WHERE sn IS NULL ORDER BY created_at ASC"
    )
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
            "UPDATE users SET sn = nextval('user_sn_seq') WHERE id = $1 AND sn IS NULL"
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
