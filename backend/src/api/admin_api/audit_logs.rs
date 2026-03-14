use actix_web::{get, web, HttpResponse, Responder};

use crate::db::AppState;
use crate::models::CurrentUser;
use crate::services::{AdminService, AuditLogQuery};

use super::{check_admin, utils::handle_admin_error};

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

/// 配置路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_audit_logs);
}
