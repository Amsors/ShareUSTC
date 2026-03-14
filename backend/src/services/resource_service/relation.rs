//! 资源关联相关功能

use crate::models::resource::RelatedResourceInfo;
use crate::services::ResourceError;
use sqlx::PgPool;
use uuid::Uuid;

/// 搜索可关联的资源
/// 用于在上传资源时搜索要关联的其他资源
/// 排除当前资源（如果提供了 exclude_id）
/// 只返回已通过审核的资源
pub async fn search_resources_for_relation(
    pool: &PgPool,
    query: &str,
    exclude_id: Option<Uuid>,
    limit: i32,
) -> Result<Vec<RelatedResourceInfo>, ResourceError> {
    let limit = limit.max(1).min(20);
    let search_pattern = format!("%{}%", query);

    let mut builder = sqlx::QueryBuilder::new(
        r#"
        SELECT r.id, r.title, r.resource_type, r.category, r.created_at
        FROM resources r
        WHERE r.audit_status = 'approved'
        AND (r.title ILIKE
        "#,
    );
    builder.push_bind(&search_pattern);
    builder.push(" OR r.id::text = ");
    builder.push_bind(query); // 支持通过UUID精确搜索
    builder.push(")");

    // 排除指定资源（避免自关联或已关联的）
    if let Some(exclude) = exclude_id {
        builder.push(" AND r.id != ");
        builder.push_bind(exclude);
    }

    // 添加排序和限制
    builder.push(" ORDER BY r.created_at DESC LIMIT ");
    builder.push_bind(limit as i64);

    let resources = builder
        .build_query_as::<RelatedResourceInfo>()
        .fetch_all(pool)
        .await
        .map_err(|e| {
            log::warn!(
                "[Resource] 搜索可关联资源失败 | query={}, error={}",
                query,
                e
            );
            ResourceError::DatabaseError(e.to_string())
        })?;

    Ok(resources)
}

/// 获取资源的关联资源列表
/// 返回该资源主动关联的其他资源列表
pub async fn get_related_resources(
    pool: &PgPool,
    resource_id: Uuid,
) -> Result<Vec<RelatedResourceInfo>, ResourceError> {
    let resources = sqlx::query_as::<_, RelatedResourceInfo>(
        r#"
        SELECT r.id, r.title, r.resource_type, r.category, r.created_at
        FROM resources r
        INNER JOIN resource_relations rr ON r.id = rr.target_resource_id
        WHERE rr.source_resource_id = $1 AND r.audit_status = 'approved'
        ORDER BY rr.created_at DESC
        "#,
    )
    .bind(resource_id)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        log::warn!(
            "[Resource] 获取关联资源列表失败 | resource_id={}, error={}",
            resource_id,
            e
        );
        ResourceError::DatabaseError(e.to_string())
    })?;

    Ok(resources)
}
