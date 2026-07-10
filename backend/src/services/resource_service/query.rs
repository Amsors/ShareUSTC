//! 资源查询相关功能

use crate::models::resource::*;
use crate::services::resource_service::utils::{add_resource_type_condition, calc_avg};
use crate::services::ResourceError;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// 获取资源详情
pub async fn get_resource_detail(
    pool: &PgPool,
    resource_id: Uuid,
) -> Result<ResourceDetailResponse, ResourceError> {
    // 获取资源信息
    let resource: Resource = sqlx::query_as::<_, Resource>("SELECT * FROM resources WHERE id = $1")
        .bind(resource_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| ResourceError::NotFound(format!("资源 {} 不存在", resource_id)))?;

    // 获取统计信息
    let stats: ResourceStats =
        sqlx::query_as::<_, ResourceStats>("SELECT * FROM resource_stats WHERE resource_id = $1")
            .bind(resource_id)
            .fetch_one(pool)
            .await?;

    // 获取上传者名称
    let uploader_name: Option<String> =
        sqlx::query_scalar("SELECT username FROM users WHERE id = $1")
            .bind(resource.uploader_id)
            .fetch_optional(pool)
            .await?;

    // 转换标签
    let tags: Option<Vec<String>> = resource
        .tags
        .as_ref()
        .and_then(|t| serde_json::from_value::<Vec<String>>(t.clone()).ok());

    // 获取关联的教师列表
    let teachers: Vec<super::TeacherInfo> = sqlx::query_as::<_, super::TeacherInfo>(
        r#"
        SELECT t.sn, t.name, t.department
        FROM teachers t
        INNER JOIN resource_teachers rt ON t.sn = rt.teacher_sn
        WHERE rt.resource_id = $1 AND t.is_active = true
        ORDER BY t.sn ASC
        "#,
    )
    .bind(resource_id)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        log::warn!(
            "[Resource] 获取关联教师失败 | resource_id={}, error={}",
            resource_id,
            e
        );
        e
    })
    .unwrap_or_default();

    // 获取关联的课程列表
    let courses: Vec<super::CourseInfo> = sqlx::query_as::<_, super::CourseInfo>(
        r#"
        SELECT c.sn, c.name, c.semester, c.credits
        FROM courses c
        INNER JOIN resource_courses rc ON c.sn = rc.course_sn
        WHERE rc.resource_id = $1 AND c.is_active = true
        ORDER BY c.sn ASC
        "#,
    )
    .bind(resource_id)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        log::warn!(
            "[Resource] 获取关联课程失败 | resource_id={}, error={}",
            resource_id,
            e
        );
        e
    })
    .unwrap_or_default();

    // 获取关联的资源列表（该资源主动关联的其他资源）
    let related_resources: Vec<super::RelatedResourceInfo> =
        sqlx::query_as::<_, super::RelatedResourceInfo>(
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
                "[Resource] 获取关联资源失败 | resource_id={}, error={}",
                resource_id,
                e
            );
            e
        })
        .unwrap_or_default();

    Ok(ResourceDetailResponse {
        id: resource.id,
        title: resource.title,
        author_id: resource.author_id,
        uploader_id: resource.uploader_id,
        course_name: resource.course_name,
        resource_type: resource.resource_type,
        category: resource.category,
        tags,
        description: resource.description,
        file_size: resource.file_size,
        audit_status: resource.audit_status,
        created_at: resource.created_at,
        updated_at: resource.updated_at,
        stats: ResourceStatsResponse {
            views: stats.views,
            downloads: stats.downloads,
            likes: stats.likes,
            avg_difficulty: stats.avg_difficulty(),
            avg_overall_quality: stats.avg_overall_quality(),
            avg_answer_quality: stats.avg_answer_quality(),
            avg_format_quality: stats.avg_format_quality(),
            avg_detail_level: stats.avg_detail_level(),
            rating_count: stats.rating_count(),
        },
        uploader_name,
        teachers,
        courses,
        related_resources,
        storage_type: resource
            .storage_type
            .clone()
            .unwrap_or_else(|| "local".to_string()),
    })
}

/// 获取资源列表
/// 使用 QueryBuilder 构建动态查询，避免字符串拼接
pub async fn get_resource_list(
    pool: &PgPool,
    query: &ResourceListQuery,
) -> Result<ResourceListResponse, ResourceError> {
    let page = query.get_page();
    let per_page = query.get_per_page();
    let offset = (page - 1) * per_page;

    // 构建排序
    let sort_by = match query.sort_by.as_deref() {
        Some("downloads") => "rs.downloads",
        Some("likes") => "rs.likes",
        // 按总体质量平均分排序（当 count > 0 时计算，否则视为 0）
        Some("rating") => "CASE WHEN rs.overall_quality_count > 0 THEN rs.overall_quality_total::FLOAT / rs.overall_quality_count ELSE 0 END",
        Some("title") => "r.title",
        _ => "r.created_at",
    };
    let sort_order = match query.sort_order.as_deref() {
        Some("asc") => "ASC",
        _ => "DESC",
    };

    // 判断是否需要关联表
    let need_teacher_join = !query.teacher_sns.is_empty();
    let need_course_join = !query.course_sns.is_empty();

    // 使用 QueryBuilder 构建 COUNT 查询
    let mut count_builder = sqlx::QueryBuilder::new(
        "SELECT COUNT(DISTINCT r.id) FROM resources r WHERE r.audit_status = 'approved'",
    );

    // 添加关联表 JOIN
    if need_teacher_join {
        count_builder.push(" AND EXISTS (SELECT 1 FROM resource_teachers rt WHERE rt.resource_id = r.id AND rt.teacher_sn = ANY(");
        count_builder.push_bind(&query.teacher_sns);
        count_builder.push("))");
    }

    if need_course_join {
        count_builder.push(" AND EXISTS (SELECT 1 FROM resource_courses rc WHERE rc.resource_id = r.id AND rc.course_sn = ANY(");
        count_builder.push_bind(&query.course_sns);
        count_builder.push("))");
    }

    // 处理资源类型筛选（支持合并类型）
    add_resource_type_condition(&mut count_builder, query.resource_type.as_deref());

    // 处理分类筛选
    if let Some(ref category) = query.category {
        count_builder.push(" AND r.category = ");
        count_builder.push_bind(category);
    }

    let total: i64 = count_builder.build_query_scalar().fetch_one(pool).await?;

    // 使用 QueryBuilder 构建列表查询
    let mut list_builder = sqlx::QueryBuilder::new(
        r#"
        SELECT r.*, rs.views, rs.downloads, rs.likes,
               rs.difficulty_total, rs.difficulty_count,
               rs.overall_quality_total, rs.overall_quality_count,
               rs.answer_quality_total, rs.answer_quality_count,
               rs.format_quality_total, rs.format_quality_count,
               rs.detail_level_total, rs.detail_level_count,
               u.username as uploader_name
        FROM resources r
        LEFT JOIN resource_stats rs ON r.id = rs.resource_id
        LEFT JOIN users u ON r.uploader_id = u.id
        WHERE r.audit_status = 'approved'
        "#,
    );

    // 添加关联表筛选条件
    if need_teacher_join {
        list_builder.push(" AND EXISTS (SELECT 1 FROM resource_teachers rt WHERE rt.resource_id = r.id AND rt.teacher_sn = ANY(");
        list_builder.push_bind(&query.teacher_sns);
        list_builder.push("))");
    }

    if need_course_join {
        list_builder.push(" AND EXISTS (SELECT 1 FROM resource_courses rc WHERE rc.resource_id = r.id AND rc.course_sn = ANY(");
        list_builder.push_bind(&query.course_sns);
        list_builder.push("))");
    }

    // 处理资源类型筛选
    add_resource_type_condition(&mut list_builder, query.resource_type.as_deref());

    // 处理分类筛选
    if let Some(ref category) = query.category {
        list_builder.push(" AND r.category = ");
        list_builder.push_bind(category);
    }

    // 添加排序和分页
    list_builder.push(format!(" ORDER BY {} {}", sort_by, sort_order));
    list_builder.push(" LIMIT ");
    list_builder.push_bind(per_page as i64);
    list_builder.push(" OFFSET ");
    list_builder.push_bind(offset as i64);

    let rows = list_builder.build().fetch_all(pool).await?;

    let resources = map_rows_to_resources(rows)?;

    Ok(ResourceListResponse {
        resources,
        total,
        page,
        per_page,
    })
}

/// 搜索资源
/// 使用 QueryBuilder 构建动态查询，避免字符串拼接
pub async fn search_resources(
    pool: &PgPool,
    query: &ResourceSearchQuery,
) -> Result<ResourceListResponse, ResourceError> {
    let page = query.get_page();
    let per_page = query.get_per_page();
    let offset = (page - 1) * per_page;

    let search_pattern = format!("%{}%", query.q);

    // 判断是否需要关联表
    let need_teacher_join = !query.teacher_sns.is_empty();
    let need_course_join = !query.course_sns.is_empty();

    // 使用 QueryBuilder 构建 COUNT 查询
    let mut count_builder = sqlx::QueryBuilder::new(
        "SELECT COUNT(DISTINCT r.id) FROM resources r WHERE r.audit_status = 'approved' AND (r.title ILIKE "
    );
    count_builder.push_bind(&search_pattern);
    count_builder.push(" OR r.course_name ILIKE ");
    count_builder.push_bind(&search_pattern);
    count_builder.push(")");

    // 添加关联表筛选条件
    if need_teacher_join {
        count_builder.push(" AND EXISTS (SELECT 1 FROM resource_teachers rt WHERE rt.resource_id = r.id AND rt.teacher_sn = ANY(");
        count_builder.push_bind(&query.teacher_sns);
        count_builder.push("))");
    }

    if need_course_join {
        count_builder.push(" AND EXISTS (SELECT 1 FROM resource_courses rc WHERE rc.resource_id = r.id AND rc.course_sn = ANY(");
        count_builder.push_bind(&query.course_sns);
        count_builder.push("))");
    }

    // 处理资源类型筛选（支持合并类型）
    add_resource_type_condition(&mut count_builder, query.resource_type.as_deref());

    // 处理分类筛选
    if let Some(ref category) = query.category {
        count_builder.push(" AND r.category = ");
        count_builder.push_bind(category);
    }

    let total: i64 = count_builder.build_query_scalar().fetch_one(pool).await?;

    // 使用 QueryBuilder 构建搜索查询
    let mut search_builder = sqlx::QueryBuilder::new(
        r#"
        SELECT r.*, rs.views, rs.downloads, rs.likes,
               rs.difficulty_total, rs.difficulty_count,
               rs.overall_quality_total, rs.overall_quality_count,
               rs.answer_quality_total, rs.answer_quality_count,
               rs.format_quality_total, rs.format_quality_count,
               rs.detail_level_total, rs.detail_level_count,
               u.username as uploader_name
        FROM resources r
        LEFT JOIN resource_stats rs ON r.id = rs.resource_id
        LEFT JOIN users u ON r.uploader_id = u.id
        WHERE r.audit_status = 'approved' AND (r.title ILIKE
        "#,
    );
    search_builder.push_bind(&search_pattern);
    search_builder.push(" OR r.course_name ILIKE ");
    search_builder.push_bind(&search_pattern);
    search_builder.push(")");

    // 添加关联表筛选条件
    if need_teacher_join {
        search_builder.push(" AND EXISTS (SELECT 1 FROM resource_teachers rt WHERE rt.resource_id = r.id AND rt.teacher_sn = ANY(");
        search_builder.push_bind(&query.teacher_sns);
        search_builder.push("))");
    }

    if need_course_join {
        search_builder.push(" AND EXISTS (SELECT 1 FROM resource_courses rc WHERE rc.resource_id = r.id AND rc.course_sn = ANY(");
        search_builder.push_bind(&query.course_sns);
        search_builder.push("))");
    }

    // 处理资源类型筛选
    add_resource_type_condition(&mut search_builder, query.resource_type.as_deref());

    // 处理分类筛选
    if let Some(ref category) = query.category {
        search_builder.push(" AND r.category = ");
        search_builder.push_bind(category);
    }

    // 添加排序和分页
    search_builder.push(" ORDER BY r.created_at DESC LIMIT ");
    search_builder.push_bind(per_page as i64);
    search_builder.push(" OFFSET ");
    search_builder.push_bind(offset as i64);

    let rows = search_builder.build().fetch_all(pool).await?;

    let resources = map_rows_to_resources(rows)?;

    Ok(ResourceListResponse {
        resources,
        total,
        page,
        per_page,
    })
}

/// 获取用户上传的资源列表
pub async fn get_user_resources(
    pool: &PgPool,
    user_id: Uuid,
    page: i32,
    per_page: i32,
) -> Result<ResourceListResponse, ResourceError> {
    let offset = (page - 1) * per_page;

    // 获取总数
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM resources WHERE uploader_id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await?;

    // 获取资源列表
    let rows = sqlx::query(
        r#"
        SELECT r.*, rs.views, rs.downloads, rs.likes,
               rs.difficulty_total, rs.difficulty_count,
               rs.overall_quality_total, rs.overall_quality_count,
               rs.answer_quality_total, rs.answer_quality_count,
               rs.format_quality_total, rs.format_quality_count,
               rs.detail_level_total, rs.detail_level_count,
               u.username as uploader_name
        FROM resources r
        LEFT JOIN resource_stats rs ON r.id = rs.resource_id
        LEFT JOIN users u ON r.uploader_id = u.id
        WHERE r.uploader_id = $1
        ORDER BY r.created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(user_id)
    .bind(per_page as i64)
    .bind(offset as i64)
    .fetch_all(pool)
    .await?;

    let resources = map_rows_to_resources(rows)?;

    Ok(ResourceListResponse {
        resources,
        total,
        page,
        per_page,
    })
}

/// 获取热门资源列表
/// 按浏览量降序排序（主要），下载量次之
/// 返回所有资源（包括待审核的），只要浏览量>0或按创建时间排序
pub async fn get_hot_resources(
    pool: &PgPool,
    limit: i32,
) -> Result<Vec<HotResourceItem>, ResourceError> {
    let limit = limit.clamp(1, 20);

    log::info!("获取热门资源，限制数量: {}", limit);

    // 先检查资源总数
    let total_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM resources")
        .fetch_one(pool)
        .await
        .unwrap_or(0);
    log::info!("数据库中共有 {} 条资源", total_count);

    // 先尝试获取有浏览量的资源
    let rows = sqlx::query(
        r#"
        SELECT 
            r.id,
            r.title,
            r.course_name,
            r.resource_type,
            COALESCE(rs.downloads, 0) as downloads,
            COALESCE(rs.views, 0) as views,
            COALESCE(rs.likes, 0) as likes
        FROM resources r
        LEFT JOIN resource_stats rs ON r.id = rs.resource_id
        ORDER BY COALESCE(rs.views, 0) DESC, COALESCE(rs.downloads, 0) DESC, r.created_at DESC
        LIMIT $1
        "#,
    )
    .bind(limit as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        log::error!("获取热门资源查询失败: {}", e);
        ResourceError::Database(e)
    })?;

    log::info!("获取到 {} 条热门资源", rows.len());

    let mut resources = Vec::new();
    for row in rows {
        resources.push(HotResourceItem {
            id: row.try_get("id")?,
            title: row.try_get("title")?,
            course_name: row.try_get("course_name").ok(),
            resource_type: row.try_get("resource_type")?,
            downloads: row.try_get::<i32, _>("downloads").unwrap_or(0),
            views: row.try_get::<i32, _>("views").unwrap_or(0),
            likes: row.try_get::<i32, _>("likes").unwrap_or(0),
        });
    }

    Ok(resources)
}

/// 获取资源总数
pub async fn get_resource_count(pool: &PgPool) -> Result<i64, ResourceError> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM resources")
        .fetch_one(pool)
        .await?;

    Ok(count)
}

/// 根据文件哈希查询资源列表
///
/// 用于上传前检查是否已存在相同内容的资源
/// 只返回审核通过的资源
pub async fn find_by_file_hash(
    pool: &PgPool,
    file_hash: &str,
) -> Result<Vec<ResourceListItem>, ResourceError> {
    // 使用 ILIKE 来支持大小写不敏感的查询（哈希可能是大写或小写）
    let records = sqlx::query_as::<
        _,
        (
            Uuid,
            String,
            Option<String>,
            String,
            String,
            Option<serde_json::Value>,
            String,
            chrono::NaiveDateTime,
            i32,
            i32,
            i32,
            Option<String>,
        ),
    >(
        r#"
        SELECT
            r.id, r.title, r.course_name, r.resource_type, r.category,
            r.tags, r.audit_status, r.created_at,
            COALESCE(rs.views, 0) as views,
            COALESCE(rs.downloads, 0) as downloads,
            COALESCE(rs.likes, 0) as likes,
            u.username as uploader_name
        FROM resources r
        LEFT JOIN resource_stats rs ON r.id = rs.resource_id
        LEFT JOIN users u ON r.uploader_id = u.id
        WHERE LOWER(r.file_hash) = LOWER($1)
            AND r.audit_status = 'approved'
        ORDER BY r.created_at DESC
        LIMIT 10
        "#,
    )
    .bind(file_hash)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        log::error!("[ResourceService] 根据哈希查询资源失败 | error={}", e);
        ResourceError::Database(e)
    })?;

    let items = records
        .into_iter()
        .map(
            |(
                id,
                title,
                course_name,
                resource_type,
                category,
                tags,
                audit_status,
                created_at,
                views,
                downloads,
                likes,
                uploader_name,
            )| {
                // 解析标签
                let tags_vec = tags.and_then(|v| serde_json::from_value::<Vec<String>>(v).ok());

                ResourceListItem {
                    id,
                    title,
                    course_name,
                    resource_type,
                    category,
                    tags: tags_vec,
                    audit_status,
                    created_at,
                    stats: ResourceStatsResponse {
                        views,
                        downloads,
                        likes,
                        avg_difficulty: None,
                        avg_overall_quality: None,
                        avg_answer_quality: None,
                        avg_format_quality: None,
                        avg_detail_level: None,
                        rating_count: 0,
                    },
                    uploader_name,
                    storage_type: "local".to_string(), // 简化为local，实际需要根据记录返回
                }
            },
        )
        .collect();

    Ok(items)
}

/// 辅助方法：将查询结果行映射为 ResourceListItem
fn map_rows_to_resources(
    rows: Vec<sqlx::postgres::PgRow>,
) -> Result<Vec<ResourceListItem>, ResourceError> {
    let mut resources = Vec::new();
    for row in rows {
        let tags_json: Option<serde_json::Value> = row.try_get("tags").ok();
        let tags: Option<Vec<String>> =
            tags_json.and_then(|t| serde_json::from_value::<Vec<String>>(t).ok());

        // 计算各维度的平均分
        let avg_difficulty = calc_avg(
            row.try_get::<i32, _>("difficulty_total").ok(),
            row.try_get::<i32, _>("difficulty_count").ok(),
        );
        let avg_overall_quality = calc_avg(
            row.try_get::<i32, _>("overall_quality_total").ok(),
            row.try_get::<i32, _>("overall_quality_count").ok(),
        );
        let avg_answer_quality = calc_avg(
            row.try_get::<i32, _>("answer_quality_total").ok(),
            row.try_get::<i32, _>("answer_quality_count").ok(),
        );
        let avg_format_quality = calc_avg(
            row.try_get::<i32, _>("format_quality_total").ok(),
            row.try_get::<i32, _>("format_quality_count").ok(),
        );
        let avg_detail_level = calc_avg(
            row.try_get::<i32, _>("detail_level_total").ok(),
            row.try_get::<i32, _>("detail_level_count").ok(),
        );

        // 评分人数取各维度中的最大值
        let rating_count: i32 = [
            row.try_get::<i32, _>("difficulty_count").unwrap_or(0),
            row.try_get::<i32, _>("overall_quality_count").unwrap_or(0),
            row.try_get::<i32, _>("answer_quality_count").unwrap_or(0),
            row.try_get::<i32, _>("format_quality_count").unwrap_or(0),
            row.try_get::<i32, _>("detail_level_count").unwrap_or(0),
        ]
        .iter()
        .max()
        .copied()
        .unwrap_or(0);

        resources.push(ResourceListItem {
            id: row.try_get("id")?,
            title: row.try_get("title")?,
            course_name: row.try_get("course_name").ok(),
            resource_type: row.try_get("resource_type")?,
            category: row.try_get("category")?,
            tags,
            audit_status: row.try_get("audit_status")?,
            created_at: row.try_get("created_at")?,
            stats: ResourceStatsResponse {
                views: row.try_get::<i32, _>("views").unwrap_or(0),
                downloads: row.try_get::<i32, _>("downloads").unwrap_or(0),
                likes: row.try_get::<i32, _>("likes").unwrap_or(0),
                avg_difficulty,
                avg_overall_quality,
                avg_answer_quality,
                avg_format_quality,
                avg_detail_level,
                rating_count,
            },
            uploader_name: row.try_get("uploader_name").ok(),
            storage_type: row
                .try_get::<Option<String>, _>("storage_type")
                .ok()
                .flatten()
                .unwrap_or_else(|| "local".to_string()),
        });
    }
    Ok(resources)
}
