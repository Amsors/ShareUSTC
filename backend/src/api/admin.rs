use actix_multipart::Multipart;
use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};
use calamine::Reader;
use futures_util::StreamExt;
use uuid::Uuid;

use crate::db::AppState;
use crate::models::CurrentUser;
use crate::models::{
    BatchDeleteCoursesRequest, BatchDeleteTeachersRequest, BatchImportCourseItem,
    BatchImportCoursesRequest, BatchImportTeacherItem, BatchImportTeachersRequest, CourseListQuery,
    CreateCourseRequest, CreateTeacherRequest, TeacherListQuery, UpdateCourseRequest,
    UpdateCourseStatusRequest, UpdateTeacherRequest, UpdateTeacherStatusRequest,
};
use crate::services::{
    AdminError, AdminService, AuditLogQuery, AuditLogService, AuditResourceRequest, CourseError,
    CourseService, FavoriteService, ResourceError, ResourceService, TeacherError, TeacherService,
    UpdateUserStatusRequest,
};
use crate::utils::{bad_request, conflict, forbidden, internal_error, no_content, not_found};

/// 检查用户是否是管理员
fn check_admin(current_user: &CurrentUser) -> Result<(), AdminError> {
    if !matches!(current_user.role, crate::models::UserRole::Admin) {
        return Err(AdminError::Forbidden("需要管理员权限".to_string()));
    }
    Ok(())
}

/// 将AdminError转换为HttpResponse
/// 使用正确的 HTTP 状态码
fn handle_admin_error(err: AdminError) -> HttpResponse {
    match err {
        AdminError::NotFound(msg) => not_found(&msg),
        AdminError::ValidationError(msg) => bad_request(&msg),
        AdminError::Forbidden(msg) => forbidden(&msg),
        AdminError::DatabaseError(msg) => {
            log::error!("[Admin] 数据库错误 | error={}", msg);
            internal_error("服务器内部错误")
        }
    }
}

/// 将TeacherError转换为HttpResponse
fn handle_teacher_error(err: TeacherError) -> HttpResponse {
    match err {
        TeacherError::NotFound(msg) => not_found(&msg),
        TeacherError::ValidationError(msg) => bad_request(&msg),
        TeacherError::DatabaseError(msg) => {
            log::error!("[Admin] 教师服务数据库错误 | error={}", msg);
            internal_error("服务器内部错误")
        }
    }
}

/// 将CourseError转换为HttpResponse
fn handle_course_error(err: CourseError) -> HttpResponse {
    match err {
        CourseError::NotFound(msg) => not_found(&msg),
        CourseError::ValidationError(msg) => bad_request(&msg),
        CourseError::DatabaseError(msg) => {
            log::error!("[Admin] 课程服务数据库错误 | error={}", msg);
            internal_error("服务器内部错误")
        }
    }
}

/// 将ResourceError转换为HttpResponse
fn handle_resource_error(err: ResourceError) -> HttpResponse {
    match err {
        ResourceError::NotFound(msg) => not_found(&msg),
        ResourceError::ValidationError(msg) => bad_request(&msg),
        ResourceError::Unauthorized(msg) => forbidden(&msg),
        ResourceError::Conflict(msg) => conflict(&msg),
        _ => {
            log::error!("[Admin] 资源服务错误 | error={}", err);
            internal_error("服务器内部错误")
        }
    }
}

/// 获取仪表盘统计数据
#[get("/admin/dashboard")]
async fn get_dashboard(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
) -> impl Responder {
    let user = current_user.into_inner();
    log::info!("[Admin] 获取仪表盘数据 | admin_id={}", user.id);

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    match AdminService::get_dashboard_stats(&data.pool).await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => handle_admin_error(e),
    }
}

/// 获取用户列表
#[get("/admin/users")]
async fn get_user_list(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let user = current_user.into_inner();
    log::info!("[Admin] 获取用户列表 | admin_id={}", user.id);

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    let page = query
        .get("page")
        .and_then(|p| p.parse::<i32>().ok())
        .unwrap_or(1);
    let per_page = query
        .get("perPage")
        .and_then(|p| p.parse::<i32>().ok())
        .unwrap_or(20);

    match AdminService::get_user_list(&data.pool, page, per_page).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => handle_admin_error(e),
    }
}

/// 更新用户状态（禁用/启用）
#[put("/admin/users/{user_id}/status")]
async fn update_user_status(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    req: web::Json<UpdateUserStatusRequest>,
    http_req: HttpRequest,
) -> impl Responder {
    let user = current_user.into_inner();

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    let user_id = path.into_inner();
    log::info!(
        "[Admin] 更新用户状态 | admin_id={}, target_user_id={}, is_active={}",
        user.id,
        user_id,
        req.is_active
    );

    // 禁止禁用自己
    if user_id == user.id {
        log::warn!("[Admin] 管理员尝试禁用自己 | admin_id={}", user.id);
        return bad_request("不能禁用自己的账号");
    }

    match AdminService::update_user_status(&data.pool, user_id, req.is_active).await {
        Ok(_) => {
            log::info!(
                "[Admin] 用户状态更新成功 | admin_id={}, target_user_id={}",
                user.id,
                user_id
            );

            // 记录审计日志
            let ip_address = http_req.peer_addr().map(|addr| addr.ip().to_string());
            if let Err(e) = AuditLogService::log_update_user_status(
                &data.pool,
                user.id,
                user_id,
                req.is_active,
                ip_address.as_deref(),
            )
            .await
            {
                log::warn!(
                    "[Audit] 记录更新用户状态日志失败 | admin_id={}, target_user_id={}, error={}",
                    user.id,
                    user_id,
                    e
                );
            }

            HttpResponse::Ok().json(serde_json::json!({
                "message": "用户状态已更新"
            }))
        }
        Err(e) => handle_admin_error(e),
    }
}

/// 获取用户实名信息
#[get("/admin/users/{user_id}/real-info")]
async fn get_user_real_info(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let user = current_user.into_inner();

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    let user_id = path.into_inner();
    log::info!(
        "[Admin] 管理员获取用户实名信息 | admin_id={}, target_user_id={}",
        user.id,
        user_id
    );

    match AdminService::get_user_real_info(&data.pool, user_id).await {
        Ok(real_info) => {
            log::info!(
                "[Admin] 获取用户实名信息成功 | admin_id={}, target_user_id={}",
                user.id,
                user_id
            );
            HttpResponse::Ok().json(real_info)
        }
        Err(e) => {
            log::warn!(
                "[Admin] 获取用户实名信息失败 | admin_id={}, target_user_id={}, error={}",
                user.id,
                user_id,
                e
            );
            handle_admin_error(e)
        }
    }
}

/// 获取待审核资源列表
#[get("/admin/resources/pending")]
async fn get_pending_resources(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let user = current_user.into_inner();
    log::info!("[Admin] 获取待审核资源列表 | admin_id={}", user.id);

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    let page = query
        .get("page")
        .and_then(|p| p.parse::<i32>().ok())
        .unwrap_or(1);
    let per_page = query
        .get("perPage")
        .and_then(|p| p.parse::<i32>().ok())
        .unwrap_or(20);

    match AdminService::get_pending_resources(&data.pool, page, per_page).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => handle_admin_error(e),
    }
}

/// 审核资源
#[put("/admin/resources/{resource_id}/audit")]
async fn audit_resource(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    req: web::Json<AuditResourceRequest>,
) -> impl Responder {
    let user = current_user.into_inner();

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    let resource_id = path.into_inner();
    log::info!(
        "[Admin] 审核资源 | admin_id={}, resource_id={}, status={}",
        user.id,
        resource_id,
        req.status
    );

    match AdminService::audit_resource(
        &data.pool,
        resource_id,
        req.status.clone(),
        req.reason.clone(),
    )
    .await
    {
        Ok(_) => {
            log::info!(
                "[Admin] 资源审核完成 | admin_id={}, resource_id={}",
                user.id,
                resource_id
            );
            HttpResponse::Ok().json(serde_json::json!({
                "message": "资源审核完成"
            }))
        }
        Err(e) => handle_admin_error(e),
    }
}

/// 获取评论列表
#[get("/admin/comments")]
async fn get_comment_list(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let user = current_user.into_inner();
    log::info!("[Admin] 获取评论列表 | admin_id={}", user.id);

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    let page = query
        .get("page")
        .and_then(|p| p.parse::<i32>().ok())
        .unwrap_or(1);
    let per_page = query
        .get("perPage")
        .and_then(|p| p.parse::<i32>().ok())
        .unwrap_or(20);
    let audit_status = query.get("auditStatus").cloned();

    match AdminService::get_comment_list(&data.pool, page, per_page, audit_status).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => handle_admin_error(e),
    }
}

/// 删除评论
#[delete("/admin/comments/{comment_id}")]
async fn delete_comment(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> impl Responder {
    let user = current_user.into_inner();

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    let comment_id = path.into_inner();
    log::info!(
        "[Admin] 删除评论 | admin_id={}, comment_id={}",
        user.id,
        comment_id
    );

    match AdminService::delete_comment(&data.pool, comment_id).await {
        Ok(_) => {
            log::info!(
                "[Admin] 评论删除成功 | admin_id={}, comment_id={}",
                user.id,
                comment_id
            );

            // 记录审计日志
            let ip_address = req.peer_addr().map(|addr| addr.ip().to_string());
            if let Err(e) = AuditLogService::log_delete_comment(
                &data.pool,
                user.id,
                comment_id,
                true, // is_admin
                ip_address.as_deref(),
            )
            .await
            {
                log::warn!(
                    "[Audit] 记录删除评论日志失败 | admin_id={}, comment_id={}, error={}",
                    user.id,
                    comment_id,
                    e
                );
            }

            no_content()
        }
        Err(e) => handle_admin_error(e),
    }
}

/// 审核评论
#[put("/admin/comments/{comment_id}/audit")]
async fn audit_comment(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    req: web::Json<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let user = current_user.into_inner();

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    let comment_id = path.into_inner();
    let status = req.get("status").cloned().unwrap_or_default();
    log::info!(
        "[Admin] 审核评论 | admin_id={}, comment_id={}, status={}",
        user.id,
        comment_id,
        status
    );

    match AdminService::audit_comment(&data.pool, comment_id, status).await {
        Ok(_) => {
            log::info!(
                "[Admin] 评论审核完成 | admin_id={}, comment_id={}",
                user.id,
                comment_id
            );
            HttpResponse::Ok().json(serde_json::json!({
                "message": "评论审核完成"
            }))
        }
        Err(e) => handle_admin_error(e),
    }
}

/// 发送系统通知
#[post("/admin/notifications")]
async fn send_notification(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    req: web::Json<crate::services::SendNotificationRequest>,
    http_req: HttpRequest,
) -> impl Responder {
    let user = current_user.into_inner();
    log::info!(
        "[Admin] 发送系统通知 | admin_id={}, title={}",
        user.id,
        req.title
    );

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    // 提前保存需要的数据
    let title = req.title.clone();

    // 获取接收者数量
    let recipient_count = if req.target == "all" {
        // 广播给所有用户，获取用户总数
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE is_active = true")
            .fetch_one(&data.pool)
            .await
            .unwrap_or(0) as i32
    } else {
        // 指定用户
        1
    };

    match AdminService::send_notification(&data.pool, req.into_inner()).await {
        Ok(_) => {
            log::info!("[Admin] 系统通知发送成功 | admin_id={}", user.id);

            // 记录审计日志
            let ip_address = http_req.peer_addr().map(|addr| addr.ip().to_string());
            if let Err(e) = AuditLogService::log_send_notification(
                &data.pool,
                user.id,
                &title,
                recipient_count,
                ip_address.as_deref(),
            )
            .await
            {
                log::warn!(
                    "[Audit] 记录发送通知日志失败 | admin_id={}, error={}",
                    user.id,
                    e
                );
            }

            HttpResponse::Created().json(serde_json::json!({
                "message": "通知发送成功"
            }))
        }
        Err(e) => handle_admin_error(e),
    }
}

/// 获取详细统计数据
#[get("/admin/stats/detailed")]
async fn get_detailed_stats(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
) -> impl Responder {
    let user = current_user.into_inner();
    log::info!("[Admin] 获取详细统计数据 | admin_id={}", user.id);

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    match AdminService::get_detailed_stats(&data.pool).await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => handle_admin_error(e),
    }
}

/// 获取操作日志列表
#[get("/admin/audit-logs")]
async fn get_audit_logs(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    query: web::Query<AuditLogQuery>,
) -> impl Responder {
    let user = current_user.into_inner();
    log::info!("[Admin] 获取审计日志 | admin_id={}", user.id);

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    let query_params = AuditLogQuery {
        page: query.page,
        per_page: query.per_page,
        action: query.action.clone(),
        user_id: query.user_id,
        start_date: query.start_date.clone(),
        end_date: query.end_date.clone(),
    };

    match AdminService::get_audit_logs(&data.pool, query_params).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => handle_admin_error(e),
    }
}

/// ==================== 教师管理接口 ====================

/// 获取教师列表（管理员）
#[get("/admin/teachers")]
async fn get_teacher_list(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    query: web::Query<TeacherListQuery>,
) -> impl Responder {
    let user = current_user.into_inner();
    log::info!("[Admin] 获取教师列表 | admin_id={}", user.id);

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    match TeacherService::get_teacher_list(&data.pool, query.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => handle_teacher_error(e),
    }
}

/// 添加教师
#[post("/admin/teachers")]
async fn create_teacher(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    req: web::Json<CreateTeacherRequest>,
) -> impl Responder {
    let user = current_user.into_inner();
    log::info!("[Admin] 添加教师 | admin_id={}", user.id);

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    match TeacherService::create_teacher(&data.pool, req.into_inner()).await {
        Ok(teacher) => HttpResponse::Created().json(teacher),
        Err(e) => handle_teacher_error(e),
    }
}

/// 更新教师信息
#[put("/admin/teachers/{sn}")]
async fn update_teacher(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<i64>,
    req: web::Json<UpdateTeacherRequest>,
) -> impl Responder {
    let user = current_user.into_inner();
    let sn = path.into_inner();
    log::info!(
        "[Admin] 更新教师信息 | admin_id={}, teacher_sn={}",
        user.id,
        sn
    );

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    match TeacherService::update_teacher(&data.pool, sn, req.into_inner()).await {
        Ok(teacher) => HttpResponse::Ok().json(teacher),
        Err(e) => handle_teacher_error(e),
    }
}

/// 更新教师状态
#[put("/admin/teachers/{sn}/status")]
async fn update_teacher_status(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<i64>,
    req: web::Json<UpdateTeacherStatusRequest>,
) -> impl Responder {
    let user = current_user.into_inner();
    let sn = path.into_inner();
    log::info!(
        "[Admin] 更新教师状态 | admin_id={}, teacher_sn={}, is_active={}",
        user.id,
        sn,
        req.is_active
    );

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    match TeacherService::update_teacher_status(&data.pool, sn, req.into_inner()).await {
        Ok(teacher) => HttpResponse::Ok().json(teacher),
        Err(e) => handle_teacher_error(e),
    }
}

/// 删除教师
#[delete("/admin/teachers/{sn}")]
async fn delete_teacher(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<i64>,
) -> impl Responder {
    let user = current_user.into_inner();
    let sn = path.into_inner();
    log::info!("[Admin] 删除教师 | admin_id={}, teacher_sn={}", user.id, sn);

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    match TeacherService::delete_teacher(&data.pool, sn).await {
        Ok(_) => no_content(),
        Err(e) => handle_teacher_error(e),
    }
}

/// ==================== 课程管理接口 ====================

/// 获取课程列表（管理员）
#[get("/admin/courses")]
async fn get_course_list(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    query: web::Query<CourseListQuery>,
) -> impl Responder {
    let user = current_user.into_inner();
    log::info!("[Admin] 获取课程列表 | admin_id={}", user.id);

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    match CourseService::get_course_list(&data.pool, query.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => handle_course_error(e),
    }
}

/// 添加课程
#[post("/admin/courses")]
async fn create_course(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    req: web::Json<CreateCourseRequest>,
) -> impl Responder {
    let user = current_user.into_inner();
    log::info!("[Admin] 添加课程 | admin_id={}", user.id);

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    match CourseService::create_course(&data.pool, req.into_inner()).await {
        Ok(course) => HttpResponse::Created().json(course),
        Err(e) => handle_course_error(e),
    }
}

/// 更新课程信息
#[put("/admin/courses/{sn}")]
async fn update_course(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<i64>,
    req: web::Json<UpdateCourseRequest>,
) -> impl Responder {
    let user = current_user.into_inner();
    let sn = path.into_inner();
    log::info!(
        "[Admin] 更新课程信息 | admin_id={}, course_sn={}",
        user.id,
        sn
    );

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    match CourseService::update_course(&data.pool, sn, req.into_inner()).await {
        Ok(course) => HttpResponse::Ok().json(course),
        Err(e) => handle_course_error(e),
    }
}

/// 更新课程状态
#[put("/admin/courses/{sn}/status")]
async fn update_course_status(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<i64>,
    req: web::Json<UpdateCourseStatusRequest>,
) -> impl Responder {
    let user = current_user.into_inner();
    let sn = path.into_inner();
    log::info!(
        "[Admin] 更新课程状态 | admin_id={}, course_sn={}, is_active={}",
        user.id,
        sn,
        req.is_active
    );

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    match CourseService::update_course_status(&data.pool, sn, req.into_inner()).await {
        Ok(course) => HttpResponse::Ok().json(course),
        Err(e) => handle_course_error(e),
    }
}

/// 删除课程
#[delete("/admin/courses/{sn}")]
async fn delete_course(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<i64>,
) -> impl Responder {
    let user = current_user.into_inner();
    let sn = path.into_inner();
    log::info!("[Admin] 删除课程 | admin_id={}, course_sn={}", user.id, sn);

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    match CourseService::delete_course(&data.pool, sn).await {
        Ok(_) => no_content(),
        Err(e) => handle_course_error(e),
    }
}

/// 批量导入教师
#[post("/admin/teachers/batch-import")]
async fn batch_import_teachers(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    req: web::Json<BatchImportTeachersRequest>,
) -> impl Responder {
    let user = current_user.into_inner();
    log::info!("[Admin] 批量导入教师 | admin_id={}, count={}", user.id, req.teachers.len());

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    if req.teachers.is_empty() {
        return bad_request("导入数据不能为空");
    }

    match TeacherService::batch_import_teachers(&data.pool, req.teachers.clone()).await {
        Ok(result) => {
            log::info!(
                "[Admin] 批量导入教师完成 | admin_id={}, success={}, fail={}",
                user.id,
                result.success_count,
                result.fail_count
            );
            HttpResponse::Ok().json(result)
        }
        Err(e) => handle_teacher_error(e),
    }
}

/// 批量导入课程
#[post("/admin/courses/batch-import")]
async fn batch_import_courses(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    req: web::Json<BatchImportCoursesRequest>,
) -> impl Responder {
    let user = current_user.into_inner();
    log::info!("[Admin] 批量导入课程 | admin_id={}, count={}", user.id, req.courses.len());

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    if req.courses.is_empty() {
        return bad_request("导入数据不能为空");
    }

    match CourseService::batch_import_courses(&data.pool, req.courses.clone()).await {
        Ok(result) => {
            log::info!(
                "[Admin] 批量导入课程完成 | admin_id={}, success={}, fail={}",
                user.id,
                result.success_count,
                result.fail_count
            );
            HttpResponse::Ok().json(result)
        }
        Err(e) => handle_course_error(e),
    }
}

/// 解析文件内容为教师数据
fn parse_teachers_from_bytes(
    data: &[u8],
    file_type: &str,
) -> Result<Vec<BatchImportTeacherItem>, String> {
    match file_type {
        "json" => {
            let teachers: Vec<BatchImportTeacherItem> =
                serde_json::from_slice(data).map_err(|e| format!("JSON解析错误: {}", e))?;
            Ok(teachers)
        }
        "csv" => {
            let mut rdr = csv::Reader::from_reader(data);
            let mut teachers = Vec::new();
            for (idx, result) in rdr.records().enumerate() {
                let record = result.map_err(|e| format!("CSV第{}行解析错误: {}", idx + 1, e))?;
                let name = record
                    .get(0)
                    .ok_or_else(|| format!("CSV第{}行: 缺少姓名", idx + 1))?
                    .trim()
                    .to_string();
                let department = record.get(1).map(|s| s.trim().to_string());
                let department = if department.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                    None
                } else {
                    department
                };
                teachers.push(BatchImportTeacherItem { name, department });
            }
            Ok(teachers)
        }
        "xlsx" => {
            let mut teachers = Vec::new();
            let cursor = std::io::Cursor::new(data);
            let mut workbook: calamine::Xlsx<std::io::Cursor<&[u8]>> = calamine::Xlsx::new(cursor)
                .map_err(|e| format!("Excel文件解析错误: {:?}", e))?;
            let range = workbook
                .worksheet_range_at(0)
                .ok_or("无法读取Excel第一个工作表")?
                .map_err(|e| format!("Excel读取错误: {:?}", e))?;

            for (idx, row) in range.rows().enumerate().skip(1) {
                // 跳过标题行
                let name_cell = row
                    .get(0)
                    .ok_or_else(|| format!("Excel第{}行: 缺少姓名", idx + 1))?;
                let name = name_cell.to_string().trim().to_string();
                let department: Option<String> = row.get(1).map(|c| c.to_string().trim().to_string());
                let department = if department.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                    None
                } else {
                    department
                };
                teachers.push(BatchImportTeacherItem { name, department });
            }
            Ok(teachers)
        }
        _ => Err("不支持的文件格式".to_string()),
    }
}

/// 解析文件内容为课程数据
fn parse_courses_from_bytes(
    data: &[u8],
    file_type: &str,
) -> Result<Vec<BatchImportCourseItem>, String> {
    match file_type {
        "json" => {
            let courses: Vec<BatchImportCourseItem> =
                serde_json::from_slice(data).map_err(|e| format!("JSON解析错误: {}", e))?;
            Ok(courses)
        }
        "csv" => {
            let mut rdr = csv::Reader::from_reader(data);
            let mut courses = Vec::new();
            for (idx, result) in rdr.records().enumerate() {
                let record = result.map_err(|e| format!("CSV第{}行解析错误: {}", idx + 1, e))?;
                let name = record
                    .get(0)
                    .ok_or_else(|| format!("CSV第{}行: 缺少课程名称", idx + 1))?
                    .trim()
                    .to_string();
                let semester = record.get(1).map(|s| s.trim().to_string());
                let semester = if semester.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                    None
                } else {
                    semester
                };
                let credits = record
                    .get(2)
                    .and_then(|s| s.trim().parse::<f64>().ok())
                    .filter(|&c| c > 0.0);
                courses.push(BatchImportCourseItem {
                    name,
                    semester,
                    credits,
                });
            }
            Ok(courses)
        }
        "xlsx" => {
            let mut courses = Vec::new();
            let cursor = std::io::Cursor::new(data);
            let mut workbook: calamine::Xlsx<std::io::Cursor<&[u8]>> = calamine::Xlsx::new(cursor)
                .map_err(|e| format!("Excel文件解析错误: {:?}", e))?;
            let range = workbook
                .worksheet_range_at(0)
                .ok_or("无法读取Excel第一个工作表")?
                .map_err(|e| format!("Excel读取错误: {:?}", e))?;

            for (idx, row) in range.rows().enumerate().skip(1) {
                // 跳过标题行
                let name_cell = row
                    .get(0)
                    .ok_or_else(|| format!("Excel第{}行: 缺少课程名称", idx + 1))?;
                let name = name_cell.to_string().trim().to_string();
                let semester: Option<String> = row.get(1).map(|c| c.to_string().trim().to_string());
                let semester = if semester.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                    None
                } else {
                    semester
                };
                let credits = row
                    .get(2)
                    .and_then(|c| c.to_string().trim().parse::<f64>().ok())
                    .filter(|&c| c > 0.0);
                courses.push(BatchImportCourseItem {
                    name,
                    semester,
                    credits,
                });
            }
            Ok(courses)
        }
        _ => Err("不支持的文件格式".to_string()),
    }
}

/// 从文件批量导入教师
#[post("/admin/teachers/batch-import-file")]
async fn batch_import_teachers_from_file(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    mut payload: Multipart,
) -> impl Responder {
    let user = current_user.into_inner();
    log::info!("[Admin] 开始从文件批量导入教师 | admin_id={}", user.id);

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    let mut file_data: Vec<u8> = Vec::new();
    let mut file_type: String = String::new();

    // 读取上传的文件
    while let Some(Ok(mut field)) = payload.next().await {
        let content_disposition = field.content_disposition();
        let name = content_disposition
            .get_name()
            .unwrap_or_default()
            .to_string();

        if name == "file" {
            // 从文件名推断文件类型
            if let Some(filename) = content_disposition.get_filename() {
                file_type = if filename.ends_with(".json") {
                    "json".to_string()
                } else if filename.ends_with(".csv") {
                    "csv".to_string()
                } else if filename.ends_with(".xlsx") {
                    "xlsx".to_string()
                } else {
                    return bad_request("不支持的文件格式，请上传 .json, .csv 或 .xlsx 文件");
                };
            }

            // 读取文件内容
            while let Some(chunk) = field.next().await {
                match chunk {
                    Ok(bytes) => file_data.extend_from_slice(&bytes),
                    Err(e) => {
                        log::error!("[Admin] 读取文件失败 | error={}", e);
                        return bad_request("文件读取失败");
                    }
                }
            }
        }
    }

    if file_data.is_empty() {
        return bad_request("未上传文件或文件为空");
    }

    if file_type.is_empty() {
        return bad_request("无法识别文件类型");
    }

    // 解析文件内容
    let teachers = match parse_teachers_from_bytes(&file_data, &file_type) {
        Ok(teachers) => teachers,
        Err(e) => {
            return bad_request(&e);
        }
    };

    if teachers.is_empty() {
        return bad_request("文件中没有有效的教师数据");
    }

    log::info!(
        "[Admin] 文件解析成功，开始导入 | admin_id={}, count={}",
        user.id,
        teachers.len()
    );

    // 调用批量导入服务
    match TeacherService::batch_import_teachers(&data.pool, teachers).await {
        Ok(result) => {
            log::info!(
                "[Admin] 批量导入教师完成 | admin_id={}, success={}, fail={}",
                user.id,
                result.success_count,
                result.fail_count
            );
            HttpResponse::Ok().json(result)
        }
        Err(e) => handle_teacher_error(e),
    }
}

/// 从文件批量导入课程
#[post("/admin/courses/batch-import-file")]
async fn batch_import_courses_from_file(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    mut payload: Multipart,
) -> impl Responder {
    let user = current_user.into_inner();
    log::info!("[Admin] 开始从文件批量导入课程 | admin_id={}", user.id);

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    let mut file_data: Vec<u8> = Vec::new();
    let mut file_type: String = String::new();

    // 读取上传的文件
    while let Some(Ok(mut field)) = payload.next().await {
        let content_disposition = field.content_disposition();
        let name = content_disposition
            .get_name()
            .unwrap_or_default()
            .to_string();

        if name == "file" {
            // 从文件名推断文件类型
            if let Some(filename) = content_disposition.get_filename() {
                file_type = if filename.ends_with(".json") {
                    "json".to_string()
                } else if filename.ends_with(".csv") {
                    "csv".to_string()
                } else if filename.ends_with(".xlsx") {
                    "xlsx".to_string()
                } else {
                    return bad_request("不支持的文件格式，请上传 .json, .csv 或 .xlsx 文件");
                };
            }

            // 读取文件内容
            while let Some(chunk) = field.next().await {
                match chunk {
                    Ok(bytes) => file_data.extend_from_slice(&bytes),
                    Err(e) => {
                        log::error!("[Admin] 读取文件失败 | error={}", e);
                        return bad_request("文件读取失败");
                    }
                }
            }
        }
    }

    if file_data.is_empty() {
        return bad_request("未上传文件或文件为空");
    }

    if file_type.is_empty() {
        return bad_request("无法识别文件类型");
    }

    // 解析文件内容
    let courses = match parse_courses_from_bytes(&file_data, &file_type) {
        Ok(courses) => courses,
        Err(e) => {
            return bad_request(&e);
        }
    };

    if courses.is_empty() {
        return bad_request("文件中没有有效的课程数据");
    }

    log::info!(
        "[Admin] 文件解析成功，开始导入 | admin_id={}, count={}",
        user.id,
        courses.len()
    );

    // 调用批量导入服务
    match CourseService::batch_import_courses(&data.pool, courses).await {
        Ok(result) => {
            log::info!(
                "[Admin] 批量导入课程完成 | admin_id={}, success={}, fail={}",
                user.id,
                result.success_count,
                result.fail_count
            );
            HttpResponse::Ok().json(result)
        }
        Err(e) => handle_course_error(e),
    }
}

/// 批量删除教师
#[post("/admin/teachers/batch-delete")]
async fn batch_delete_teachers(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    req: web::Json<BatchDeleteTeachersRequest>,
) -> impl Responder {
    let user = current_user.into_inner();
    log::info!(
        "[Admin] 批量删除教师 | admin_id={}, sns={}",
        user.id,
        req.sns
    );

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    if req.sns.trim().is_empty() {
        return bad_request("编号列表不能为空");
    }

    match TeacherService::batch_delete_teachers(&data.pool, &req.sns).await {
        Ok(result) => {
            log::info!(
                "[Admin] 批量删除教师完成 | admin_id={}, success={}, not_found={}, fail={}",
                user.id,
                result.success_count,
                result.not_found_count,
                result.fail_count
            );
            HttpResponse::Ok().json(result)
        }
        Err(e) => handle_teacher_error(e),
    }
}

/// 批量删除课程
#[post("/admin/courses/batch-delete")]
async fn batch_delete_courses(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    req: web::Json<BatchDeleteCoursesRequest>,
) -> impl Responder {
    let user = current_user.into_inner();
    log::info!(
        "[Admin] 批量删除课程 | admin_id={}, sns={}",
        user.id,
        req.sns
    );

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    if req.sns.trim().is_empty() {
        return bad_request("编号列表不能为空");
    }

    match CourseService::batch_delete_courses(&data.pool, &req.sns).await {
        Ok(result) => {
            log::info!(
                "[Admin] 批量删除课程完成 | admin_id={}, success={}, not_found={}, fail={}",
                user.id,
                result.success_count,
                result.not_found_count,
                result.fail_count
            );
            HttpResponse::Ok().json(result)
        }
        Err(e) => handle_course_error(e),
    }
}

/// ==================== 资料管理接口 ====================

/// 获取所有资源列表（支持关键词搜索）
#[get("/admin/resources/all")]
async fn get_all_resources(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let user = current_user.into_inner();
    log::info!("[Admin] 获取所有资源列表 | admin_id={}", user.id);

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    let page = query
        .get("page")
        .and_then(|p| p.parse::<i32>().ok())
        .unwrap_or(1);
    let per_page = query
        .get("perPage")
        .and_then(|p| p.parse::<i32>().ok())
        .unwrap_or(20);
    let keyword = query.get("keyword").cloned();

    match AdminService::get_all_resources(&data.pool, page, per_page, keyword).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => handle_admin_error(e),
    }
}

/// 管理员删除资源
#[delete("/admin/resources/{resource_id}")]
async fn admin_delete_resource(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> impl Responder {
    let user = current_user.into_inner();

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    let resource_id = path.into_inner();
    log::info!(
        "[Admin] 管理员删除资源 | admin_id={}, resource_id={}",
        user.id,
        resource_id
    );

    match ResourceService::delete_resource(&data.pool, &user, &data.storage, resource_id).await {
        Ok(title) => {
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

            no_content()
        }
        Err(e) => handle_resource_error(e),
    }
}

/// 管理员重新计算资源哈希
#[post("/admin/resources/{resource_id}/recalculate-hash")]
async fn admin_recalculate_resource_hash(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> impl Responder {
    let user = current_user.into_inner();

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    let resource_id = path.into_inner();
    log::info!(
        "[Admin] 管理员重新计算资源hash | admin_id={}, resource_id={}",
        user.id, resource_id
    );

    match AdminService::recalculate_resource_hash(&data.pool, &data.storage, resource_id).await {
        Ok(result) => {
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
                    user.id, resource_id, e
                );
            }

            HttpResponse::Ok().json(result)
        }
        Err(e) => {
            log::error!(
                "[Admin] 资源hash重新计算失败 | admin_id={}, resource_id={}, error={}",
                user.id, resource_id, e
            );
            handle_admin_error(e)
        }
    }
}

/// 获取管理员的收藏夹列表
#[get("/admin/favorites")]
async fn get_admin_favorites(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
) -> impl Responder {
    let user = current_user.into_inner();
    log::info!("[Admin] 获取收藏夹列表 | admin_id={}", user.id);

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    match FavoriteService::get_user_favorites(&data.pool, user.id).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => handle_resource_error(e),
    }
}

/// 删除收藏夹内的所有资源
#[delete("/admin/favorites/{favorite_id}/resources")]
async fn delete_all_favorite_resources(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> impl Responder {
    let user = current_user.into_inner();

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    let favorite_id = path.into_inner();
    log::info!(
        "[Admin] 删除收藏夹内所有资源 | admin_id={}, favorite_id={}",
        user.id,
        favorite_id
    );

    match AdminService::delete_all_favorite_resources(&data.pool, &user, &data.storage, favorite_id).await {
        Ok(result) => {
            log::info!(
                "[Admin] 收藏夹内资源删除成功 | admin_id={}, favorite_id={}, deleted_count={}",
                user.id,
                favorite_id,
                result.deleted_count
            );

            // 记录审计日志
            let ip_address = req.peer_addr().map(|addr| addr.ip().to_string());
            if let Err(e) = AuditLogService::log_action(
                &data.pool,
                user.id,
                "delete_favorite_resources",
                Some("favorite"),
                Some(favorite_id),
                Some(serde_json::json!({
                    "deleted_count": result.deleted_count,
                    "favorite_name": result.favorite_name
                })),
                ip_address.as_deref(),
            )
            .await
            {
                log::warn!(
                    "[Audit] 记录删除收藏夹资源日志失败 | admin_id={}, favorite_id={}, error={}",
                    user.id,
                    favorite_id,
                    e
                );
            }

            HttpResponse::Ok().json(result)
        }
        Err(e) => handle_admin_error(e),
    }
}

/// 检测重复资源（根据文件hash）
#[get("/admin/duplicate-resources")]
async fn check_duplicate_resources(
    data: web::Data<AppState>,
    current_user: web::ReqData<CurrentUser>,
) -> impl Responder {
    let user = current_user.into_inner();
    log::info!("[Admin] 检测重复资源 | admin_id={}", user.id);

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    match AdminService::check_duplicate_resources(&data.pool).await {
        Ok(result) => {
            log::info!(
                "[Admin] 重复资源检测完成 | admin_id={}, groups={}, duplicates={}",
                user.id, result.total_groups, result.total_duplicate_resources
            );
            HttpResponse::Ok().json(result)
        }
        Err(e) => handle_admin_error(e),
    }
}

/// 配置管理后台路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_dashboard)
        .service(get_user_list)
        .service(update_user_status)
        .service(get_user_real_info)
        .service(get_pending_resources)
        .service(audit_resource)
        .service(get_comment_list)
        .service(delete_comment)
        .service(audit_comment)
        .service(send_notification)
        .service(get_detailed_stats)
        .service(get_audit_logs)
        // 教师管理
        .service(get_teacher_list)
        .service(create_teacher)
        .service(update_teacher)
        .service(update_teacher_status)
        .service(delete_teacher)
        // 课程管理
        .service(get_course_list)
        .service(create_course)
        .service(update_course)
        .service(update_course_status)
        .service(delete_course)
        // 批量导入
        .service(batch_import_teachers)
        .service(batch_import_courses)
        // 从文件批量导入
        .service(batch_import_teachers_from_file)
        .service(batch_import_courses_from_file)
        // 批量删除
        .service(batch_delete_teachers)
        .service(batch_delete_courses)
        // 资料管理
        .service(get_all_resources)
        .service(admin_delete_resource)
        .service(admin_recalculate_resource_hash)
        .service(get_admin_favorites)
        .service(delete_all_favorite_resources)
        // 重复资源检测
        .service(check_duplicate_resources);
}
