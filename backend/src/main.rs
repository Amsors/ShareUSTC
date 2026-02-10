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
                    log::warn!("无法读取图片文件 {}: {}", file_path, e);
                    HttpResponse::NotFound().json(serde_json::json!({
                        "code": 404,
                        "message": "图片文件不存在",
                        "data": null
                    }))
                }
            }
        }
        Err(e) => {
            log::warn!("获取图片路径失败: {}", e);
            HttpResponse::NotFound().json(serde_json::json!({
                "code": 404,
                "message": "图片不存在",
                "data": null
            }))
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
        log::warn!("创建图片上传目录失败: {}", e);
    });
    std::fs::create_dir_all(&config.resource_upload_path).unwrap_or_else(|e| {
        log::warn!("创建资源上传目录失败: {}", e);
    });

    log::info!("Starting ShareUSTC backend server...");
    log::info!("Server address: http://{}", server_addr);
    log::info!("Image upload directory: {}", config.image_upload_path);
    log::info!("Resource upload directory: {}", config.resource_upload_path);

    // 创建数据库连接池
    let pool = match db::create_pool_from_env().await {
        Ok(pool) => {
            log::info!("数据库连接池创建成功");
            pool
        }
        Err(e) => {
            log::error!("数据库连接失败: {}", e);
            log::warn!("请检查 DATABASE_URL 环境变量是否正确设置");
            log::warn!("示例: DATABASE_URL=postgres://username:password@localhost:5432/shareustc");
            std::process::exit(1);
        }
    };

    // 创建应用状态
    let app_state = web::Data::new(AppState::new(pool, config.jwt_secret.clone()));

    log::info!("Server starting at http://{}", server_addr);
    log::debug!("Debug logging enabled");
    log::debug!("API endpoints:");
    log::debug!("  POST /api/auth/register - 用户注册");
    log::debug!("  POST /api/auth/login    - 用户登录");
    log::debug!("  POST /api/auth/refresh  - 刷新Token");
    log::debug!("  POST /api/auth/logout   - 用户登出");
    log::debug!("  GET  /api/users/me      - 获取当前用户");
    log::debug!("  PUT  /api/users/me      - 更新用户资料");
    log::debug!("  POST /api/users/verify  - 实名认证");
    log::debug!("  GET  /api/users/{{user_id}} - 获取用户资料");
    log::debug!("  POST /api/images/upload - 上传图片");
    log::debug!("  GET  /api/images        - 获取我的图片列表");
    log::debug!("  GET  /api/images/{{id}}   - 获取图片信息");
    log::debug!("  DEL  /api/images/{{id}}   - 删除图片");
    log::debug!("  GET  /images/{{id}}       - 访问图片文件（公开）");
    log::debug!("  POST /api/resources     - 上传资源");
    log::debug!("  GET  /api/resources     - 获取资源列表");
    log::debug!("  GET  /api/resources/search - 搜索资源");
    log::debug!("  GET  /api/resources/my  - 获取我的资源列表");
    log::debug!("  GET  /api/resources/{{id}} - 获取资源详情");
    log::debug!("  GET  /api/resources/{{id}}/download - 下载资源");
    log::debug!("  DEL  /api/resources/{{id}} - 删除资源");
    log::debug!("  POST /api/favorites     - 创建收藏夹");
    log::debug!("  GET  /api/favorites     - 获取我的收藏夹列表");
    log::debug!("  GET  /api/favorites/{{id}} - 获取收藏夹详情");
    log::debug!("  PUT  /api/favorites/{{id}} - 更新收藏夹");
    log::debug!("  DEL  /api/favorites/{{id}} - 删除收藏夹");
    log::debug!("  POST /api/favorites/{{id}}/resources - 添加资源到收藏夹");
    log::debug!("  DEL  /api/favorites/{{id}}/resources/{{rid}} - 从收藏夹移除资源");
    log::debug!("  GET  /api/favorites/check/{{rid}} - 检查资源收藏状态");
    log::debug!("  GET  /api/health        - 健康检查");
    log::debug!("  GET  /api/hello         - 测试接口");

    // 克隆配置数据用于闭包
    let jwt_secret = config.jwt_secret.clone();
    let cors_origins = config.cors_allowed_origins.clone();

    // 记录 CORS 配置信息
    log::info!("CORS allowed origins: {:?}", cors_origins);

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
        ];

        let jwt_auth = JwtAuth::new(jwt_secret.clone()).with_public_rules(public_rules);

        // 构建 CORS 配置
        let cors = Cors::default()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec!["Content-Type", "Authorization", "Accept"])
            .supports_credentials()
            .max_age(3600);

        // 动态添加允许的域名
        let cors = if cors_origins_worker.contains(&"*".to_string()) {
            cors.allow_any_origin()
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
