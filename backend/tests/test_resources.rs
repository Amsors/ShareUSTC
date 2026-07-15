#![allow(clippy::expect_used)]

mod common;

use backend::models::{ResourceListQuery, UserRole};
use backend::services::{ResourceError, ResourceService};
use sqlx::PgPool;
use uuid::Uuid;

use common::{create_user, upload_request, MemoryStorage};

#[sqlx::test(migrations = "./migrations")]
async fn resource_list_supports_multi_select_and_returns_category_counts(pool: PgPool) {
    let owner = create_user(&pool, "resource_filter_owner").await;
    let lecture_id = common::seed_resource(&pool, owner.id, "PDF 讲义").await;
    let paper_id = common::seed_resource(&pool, owner.id, "PDF 试卷").await;
    let note_id = common::seed_resource(&pool, owner.id, "PPT 笔记").await;

    sqlx::query("UPDATE resources SET category = 'past_paper' WHERE id = $1")
        .bind(paper_id)
        .execute(&pool)
        .await
        .expect("应更新试卷分类");
    sqlx::query("UPDATE resources SET category = 'note', resource_type = 'ppt' WHERE id = $1")
        .bind(note_id)
        .execute(&pool)
        .await
        .expect("应更新笔记类型与分类");

    let response = ResourceService::get_resource_list(
        &pool,
        &ResourceListQuery {
            page: Some(1),
            per_page: Some(100),
            resource_type: None,
            category: None,
            resource_types: vec!["pdf".to_string()],
            categories: vec!["lecture".to_string(), "past_paper".to_string()],
            sort_by: None,
            sort_order: None,
            teacher_sns: vec![],
            course_sns: vec![],
        },
    )
    .await
    .expect("多选筛选应成功");

    assert_eq!(response.total, 2);
    assert_eq!(response.resources.len(), 2);
    assert_eq!(response.category_counts.get("lecture"), Some(&1));
    assert_eq!(response.category_counts.get("past_paper"), Some(&1));
    assert_eq!(response.category_counts.get("note"), None);
    assert!(response
        .resources
        .iter()
        .any(|resource| resource.id == lecture_id));
}

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
