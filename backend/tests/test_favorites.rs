#![allow(clippy::expect_used)]

mod common;

use backend::models::{AddToFavoriteRequest, CreateFavoriteRequest};
use backend::services::{FavoriteError, FavoriteService};
use sqlx::PgPool;

use common::{create_user, seed_resource};

#[sqlx::test(migrations = "./migrations")]
async fn favorite_core_paths_cover_success_ownership_and_conflict(pool: PgPool) {
    let owner = create_user(&pool, "favorite_owner").await;
    let other = create_user(&pool, "favorite_other").await;
    let resource_id = seed_resource(&pool, owner.id, "收藏测试资源").await;

    let favorite = FavoriteService::create_favorite(
        &pool,
        owner.id,
        CreateFavoriteRequest {
            name: "核心资料".to_string(),
        },
    )
    .await
    .expect("收藏夹应创建成功");

    let duplicate_name = FavoriteService::create_favorite(
        &pool,
        owner.id,
        CreateFavoriteRequest {
            name: "核心资料".to_string(),
        },
    )
    .await;
    assert!(matches!(
        duplicate_name,
        Err(FavoriteError::ValidationError(_))
    ));

    FavoriteService::add_resource_to_favorite(
        &pool,
        favorite.id,
        owner.id,
        AddToFavoriteRequest { resource_id },
    )
    .await
    .expect("应可收藏存在的资源");
    let duplicate_resource = FavoriteService::add_resource_to_favorite(
        &pool,
        favorite.id,
        owner.id,
        AddToFavoriteRequest { resource_id },
    )
    .await;
    assert!(matches!(
        duplicate_resource,
        Err(FavoriteError::Conflict(_))
    ));

    let forbidden = FavoriteService::get_favorite_detail(&pool, favorite.id, other.id).await;
    assert!(matches!(forbidden, Err(FavoriteError::NotFound(_))));

    let detail = FavoriteService::get_favorite_detail(&pool, favorite.id, owner.id)
        .await
        .expect("所有者应可读取收藏夹");
    assert_eq!(detail.resource_count, 1);
    assert_eq!(detail.resources[0].id, resource_id);
}
