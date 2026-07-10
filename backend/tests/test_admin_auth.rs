#![allow(clippy::expect_used)]

mod common;

use actix_web::{http::StatusCode, test, web, App};
use backend::api::admin_api;
use backend::config::BrandConfig;
use backend::db::AppState;
use backend::middleware::JwtAuth;
use backend::models::UserRole;
use backend::utils::generate_access_token;
use sqlx::PgPool;
use uuid::Uuid;

use common::MemoryStorage;

const JWT_SECRET: &str = "stage6-admin-boundary-secret";

#[sqlx::test(migrations = "./migrations")]
async fn non_admin_access_to_admin_api_returns_403(pool: PgPool) {
    let token = generate_access_token(
        Uuid::new_v4(),
        "normal_user".to_string(),
        UserRole::User,
        false,
        JWT_SECRET,
    )
    .expect("应生成测试 token");
    let state = web::Data::new(AppState::new(
        pool,
        JWT_SECRET.to_string(),
        false,
        MemoryStorage::shared(),
        false,
        true,
        true,
        BrandConfig::default(),
        None,
        None,
    ));
    let app = test::init_service(
        App::new().app_data(state).service(
            web::scope("/api")
                .wrap(JwtAuth::new(JWT_SECRET.to_string()))
                .configure(admin_api::config),
        ),
    )
    .await;

    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/admin/dashboard")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let body: serde_json::Value = test::read_body_json(response).await;
    assert_eq!(body["error"], "Forbidden");
    assert!(body["message"].is_string());
}
