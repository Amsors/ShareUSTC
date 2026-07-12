#![allow(clippy::expect_used)]

mod common;

use actix_web::{http::StatusCode, test, web, App};
use backend::api;
use backend::config::BrandConfig;
use backend::db::AppState;
use backend::models::{LoginRequest, RegisterRequest};
use backend::services::{AuthError, AuthService};
use sqlx::PgPool;

use common::{test_app_config, MemoryStorage, JWT_SECRET};

#[sqlx::test(migrations = "./migrations")]
async fn auth_register_login_refresh_logout_and_failures(pool: PgPool) {
    let register = RegisterRequest {
        username: "auth_user".to_string(),
        password: "correct-password".to_string(),
        email: Some("auth@example.com".to_string()),
    };
    let registered = AuthService::register(&pool, JWT_SECRET, register, true, &[])
        .await
        .expect("注册应成功");

    let duplicate = AuthService::register(
        &pool,
        JWT_SECRET,
        RegisterRequest {
            username: "auth_user".to_string(),
            password: "correct-password".to_string(),
            email: Some("other@example.com".to_string()),
        },
        true,
        &[],
    )
    .await;
    assert!(matches!(duplicate, Err(AuthError::UserExists(_))));

    let wrong_password = AuthService::login(
        &pool,
        JWT_SECRET,
        LoginRequest {
            username: "auth_user".to_string(),
            password: "wrong-password".to_string(),
        },
    )
    .await;
    assert!(matches!(
        wrong_password,
        Err(AuthError::InvalidCredentials(_))
    ));

    let logged_in = AuthService::login(
        &pool,
        JWT_SECRET,
        LoginRequest {
            username: "auth_user".to_string(),
            password: "correct-password".to_string(),
        },
    )
    .await
    .expect("正确凭证应登录成功");
    let refreshed = AuthService::refresh_token(&pool, JWT_SECRET, logged_in.tokens.refresh_token)
        .await
        .expect("refresh token 应可换取新 token");
    assert!(!refreshed.access_token.is_empty());
    assert_eq!(registered.user.id, logged_in.user.id);

    let state = web::Data::new(AppState::new(
        pool,
        test_app_config(),
        JWT_SECRET.to_string(),
        false,
        MemoryStorage::shared(),
        true,
        true,
        true,
        BrandConfig::default(),
        None,
        None,
    ));
    let app = test::init_service(
        App::new()
            .app_data(state)
            .service(web::scope("/api").configure(api::auth::config)),
    )
    .await;
    let response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/auth/logout")
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let cookies: Vec<_> = response.response().cookies().collect();
    assert_eq!(cookies.len(), 2);
    assert!(cookies.iter().all(|cookie| cookie.value().is_empty()));
}
