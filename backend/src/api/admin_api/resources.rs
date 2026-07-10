use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};
use uuid::Uuid;

use crate::db::AppState;
use crate::models::CurrentUser;
use crate::services::{AdminService, AuditLogService, AuditResourceRequest, ResourceService};

use super::check_admin;

/// 获取待审核资源列表
#[get("/admin/resources/pending")]
async fn get_pending_resources(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let user = current_user.into_inner();
    log::info!("[Admin] 获取待审核资源列表 | admin_id={}", user.id);

    check_admin(&user)?;

    let page = query
        .get("page")
        .and_then(|p| p.parse::<i32>().ok())
        .unwrap_or(1);
    let per_page = query
        .get("perPage")
        .and_then(|p| p.parse::<i32>().ok())
        .unwrap_or(20);

    let response = AdminService::get_pending_resources(&data.pool, page, per_page).await?;
    Ok(HttpResponse::Ok().json(response))
}

/// 审核资源
#[put("/admin/resources/{resource_id}/audit")]
async fn audit_resource(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    req: web::Json<AuditResourceRequest>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let user = current_user.into_inner();

    check_admin(&user)?;

    let resource_id = path.into_inner();
    log::info!(
        "[Admin] 审核资源 | admin_id={}, resource_id={}, status={}",
        user.id,
        resource_id,
        req.status
    );

    AdminService::audit_resource(
        &data.pool,
        resource_id,
        req.status.clone(),
        req.reason.clone(),
    )
    .await?;
    log::info!(
        "[Admin] 资源审核完成 | admin_id={}, resource_id={}",
        user.id,
        resource_id
    );
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "资源审核完成"
    })))
}

/// 获取所有资源列表（支持关键词搜索）
#[get("/admin/resources/all")]
async fn get_all_resources(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let user = current_user.into_inner();
    log::info!("[Admin] 获取所有资源列表 | admin_id={}", user.id);

    check_admin(&user)?;

    let page = query
        .get("page")
        .and_then(|p| p.parse::<i32>().ok())
        .unwrap_or(1);
    let per_page = query
        .get("perPage")
        .and_then(|p| p.parse::<i32>().ok())
        .unwrap_or(20);
    let keyword = query.get("keyword").cloned();

    let response = AdminService::get_all_resources(&data.pool, page, per_page, keyword).await?;
    Ok(HttpResponse::Ok().json(response))
}

/// 管理员删除资源
#[delete("/admin/resources/{resource_id}")]
async fn admin_delete_resource(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let user = current_user.into_inner();

    check_admin(&user)?;

    let resource_id = path.into_inner();
    log::info!(
        "[Admin] 管理员删除资源 | admin_id={}, resource_id={}",
        user.id,
        resource_id
    );

    let title =
        ResourceService::delete_resource(&data.pool, &user, &data.storage, resource_id).await?;
    log::info!(
        "[Admin] 资源删除成功 | admin_id={}, resource_id={}, title={}",
        user.id,
        resource_id,
        title
    );

    // 记录审计日志
    let ip_address = req.peer_addr().map(|addr| addr.ip().to_string());
    if let Err(e) = AuditLogService::log_delete_resource(
        &data.pool,
        user.id,
        resource_id,
        &title,
        ip_address.as_deref(),
    )
    .await
    {
        log::warn!(
            "[Audit] 记录删除资源日志失败 | admin_id={}, resource_id={}, error={}",
            user.id,
            resource_id,
            e
        );
    }

    Ok(HttpResponse::NoContent().finish())
}

/// 管理员重新计算资源哈希
#[post("/admin/resources/{resource_id}/recalculate-hash")]
async fn admin_recalculate_resource_hash(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let user = current_user.into_inner();

    check_admin(&user)?;

    let resource_id = path.into_inner();
    log::info!(
        "[Admin] 管理员重新计算资源hash | admin_id={}, resource_id={}",
        user.id,
        resource_id
    );

    let result =
        AdminService::recalculate_resource_hash(&data.pool, &data.storage, resource_id).await?;
    log::info!(
        "[Admin] 资源hash重新计算成功 | admin_id={}, resource_id={}, new_hash={}",
        user.id,
        resource_id,
        &result.new_hash[..16.min(result.new_hash.len())]
    );

    // 记录审计日志
    let ip_address = req.peer_addr().map(|addr| addr.ip().to_string());
    if let Err(e) = AuditLogService::log_action(
        &data.pool,
        user.id,
        "recalculate_resource_hash",
        Some("resource"),
        Some(resource_id),
        Some(serde_json::json!({
            "old_hash": result.old_hash,
            "new_hash": result.new_hash,
            "file_size": result.file_size,
        })),
        ip_address.as_deref(),
    )
    .await
    {
        log::warn!(
            "[Audit] 记录重新计算hash日志失败 | admin_id={}, resource_id={}, error={}",
            user.id,
            resource_id,
            e
        );
    }

    Ok(HttpResponse::Ok().json(result))
}

/// 检测重复资源（根据文件hash）
#[get("/admin/duplicate-resources")]
async fn check_duplicate_resources(
    data: web::Data<AppState>,
    current_user: web::ReqData<CurrentUser>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let user = current_user.into_inner();
    log::info!("[Admin] 检测重复资源 | admin_id={}", user.id);

    check_admin(&user)?;

    let result = AdminService::check_duplicate_resources(&data.pool).await?;
    log::info!(
        "[Admin] 重复资源检测完成 | admin_id={}, groups={}, duplicates={}",
        user.id,
        result.total_groups,
        result.total_duplicate_resources
    );
    Ok(HttpResponse::Ok().json(result))
}

/// 配置路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_pending_resources)
        .service(audit_resource)
        .service(get_all_resources)
        .service(admin_delete_resource)
        .service(admin_recalculate_resource_hash)
        .service(check_duplicate_resources);
}
