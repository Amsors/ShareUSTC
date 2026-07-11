use actix_web::{get, put, web, HttpRequest, HttpResponse};
use uuid::Uuid;

use crate::db::AppState;
use crate::models::{
    CurrentUser, UpdateResourceDescriptionRequest, UpdateResourceRelationsRequest,
};
use crate::services::{AuditLogService, ResourceError, ResourceService};
use crate::utils::{bad_request, forbidden};

/// 获取资源的关联资源列表
#[get("/resources/{resource_id}/relations")]
pub async fn get_resource_relations(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ResourceError> {
    let resource_id = path.into_inner();

    let resources = ResourceService::get_related_resources(&state.pool, resource_id).await?;
    Ok(HttpResponse::Ok().json(resources))
}

/// 更新资源关联信息
#[put("/resources/{resource_id}/relations")]
pub async fn update_resource_relations(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    request: web::Json<UpdateResourceRelationsRequest>,
    req: HttpRequest,
) -> Result<HttpResponse, ResourceError> {
    let resource_id = path.into_inner();

    // 获取资源信息（用于权限检查和审计日志）
    let resource_detail = ResourceService::get_resource_detail(&state.pool, resource_id).await?;

    // 检查权限（上传者或管理员可以修改）
    if resource_detail.uploader_id != user.id && user.role != crate::models::UserRole::Admin {
        return Ok(forbidden("只有资源上传者或管理员可以修改关联信息"));
    }

    ResourceService::update_resource_relations(
        &state.pool,
        resource_id,
        request.teacher_sns.clone(),
        request.course_sns.clone(),
        request.related_resource_ids.clone(),
    )
    .await?;

    // 记录审计日志
    let ip_address = req.peer_addr().map(|addr| addr.ip().to_string());
    if let Err(e) = AuditLogService::log_update_resource(
        &state.pool,
        user.id,
        resource_id,
        &resource_detail.title,
        ip_address.as_deref(),
    )
    .await
    {
        log::warn!(
            "[Audit] 记录资源关联更新日志失败 | resource_id={}, error={}",
            resource_id,
            e
        );
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "关联信息更新成功"
    })))
}

/// 更新资源描述
#[put("/resources/{resource_id}/description")]
pub async fn update_resource_description(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    request: web::Json<UpdateResourceDescriptionRequest>,
    req: HttpRequest,
) -> Result<HttpResponse, ResourceError> {
    let resource_id = path.into_inner();

    // 验证请求
    if let Err(msg) = request.validate() {
        return Ok(bad_request(&msg));
    }

    // 获取资源信息（用于审计日志）
    let resource_detail = ResourceService::get_resource_detail(&state.pool, resource_id).await?;

    ResourceService::update_resource_description(
        &state.pool,
        &user,
        resource_id,
        request.description.clone(),
    )
    .await?;

    // 记录审计日志
    let ip_address = req.peer_addr().map(|addr| addr.ip().to_string());
    if let Err(e) = AuditLogService::log_update_resource(
        &state.pool,
        user.id,
        resource_id,
        &resource_detail.title,
        ip_address.as_deref(),
    )
    .await
    {
        log::warn!(
            "[Audit] 记录资源描述更新日志失败 | resource_id={}, error={}",
            resource_id,
            e
        );
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "资源描述更新成功"
    })))
}
