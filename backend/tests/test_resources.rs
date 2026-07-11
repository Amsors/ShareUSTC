#![allow(clippy::expect_used)]

mod common;

use backend::models::UserRole;
use backend::services::{ResourceError, ResourceService};
use sqlx::PgPool;
use uuid::Uuid;

use common::{create_user, upload_request, MemoryStorage};

#[sqlx::test(migrations = "./migrations")]
async fn resource_upload_query_delete_enforces_ownership(pool: PgPool) {
    let owner = create_user(&pool, "resource_owner").await;
    let other = create_user(&pool, "resource_other").await;
    let storage = MemoryStorage::shared();

    let uploaded = ResourceService::upload_resource(
        &pool,
        &owner,
        &storage,
        upload_request("资源成功路径"),
        "lecture.pdf",
        b"%PDF-stage6".to_vec(),
        Some("application/pdf"),
    )
    .await
    .expect("资源应上传成功");

    let detail = ResourceService::get_resource_detail(&pool, uploaded.id)
        .await
        .expect("应查到已上传资源");
    assert_eq!(detail.title, "资源成功路径");
    assert_eq!(detail.uploader_id, owner.id);

    let forbidden = ResourceService::delete_resource(&pool, &other, &storage, uploaded.id).await;
    assert!(matches!(forbidden, Err(ResourceError::Unauthorized(_))));
    assert!(ResourceService::get_resource_detail(&pool, uploaded.id)
        .await
        .is_ok());

    ResourceService::delete_resource(&pool, &owner, &storage, uploaded.id)
        .await
        .expect("上传者应可删除资源");
    assert!(matches!(
        ResourceService::get_resource_detail(&pool, uploaded.id).await,
        Err(ResourceError::NotFound(_))
    ));
}

#[sqlx::test(migrations = "./migrations")]
async fn resource_upload_rolls_back_when_stats_insert_fails(pool: PgPool) {
    let owner = create_user(&pool, "rollback_owner").await;
    let storage = MemoryStorage::shared();
    sqlx::query(
        r#"CREATE FUNCTION reject_resource_stats() RETURNS trigger AS $$
           BEGIN RAISE EXCEPTION 'forced stats failure'; END; $$ LANGUAGE plpgsql"#,
    )
    .execute(&pool)
    .await
    .expect("应创建故障注入函数");
    sqlx::query(
        "CREATE TRIGGER reject_resource_stats BEFORE INSERT ON resource_stats FOR EACH ROW EXECUTE FUNCTION reject_resource_stats()",
    )
    .execute(&pool)
    .await
    .expect("应创建故障注入触发器");

    let result = ResourceService::upload_resource(
        &pool,
        &owner,
        &storage,
        upload_request("回滚测试资源"),
        "rollback.pdf",
        b"%PDF-rollback".to_vec(),
        Some("application/pdf"),
    )
    .await;
    assert!(matches!(result, Err(ResourceError::Database(_))));

    let resource_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM resources WHERE uploader_id = $1")
            .bind(owner.id)
            .fetch_one(&pool)
            .await
            .expect("应能查询回滚结果");
    assert_eq!(resource_count, 0, "统计插入失败时资源记录必须回滚");

    let missing = ResourceService::get_resource_detail(&pool, Uuid::new_v4()).await;
    assert!(matches!(missing, Err(ResourceError::NotFound(_))));
    assert_eq!(owner.role, UserRole::User);
}
