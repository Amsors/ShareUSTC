use actix_web::{get, web, HttpResponse, Responder};

use crate::db::AppState;
use crate::models::CurrentUser;
use crate::services::AdminService;

use super::{check_admin, utils::handle_admin_error};

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

/// 配置路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_dashboard).service(get_detailed_stats);
}
