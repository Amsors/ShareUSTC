use actix_web::{get, web, HttpResponse, Responder};
use serde::Deserialize;

use crate::db::AppState;
use crate::services::{TeacherError, TeacherService};
use crate::utils::{bad_request, internal_error, not_found};

/// 查询参数：获取教师列表
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetTeachersQuery {
    /// 是否只返回有关联资源的教师
    #[serde(default)]
    with_resources_only: bool,
}

/// 将 TeacherError 转换为 HttpResponse
fn handle_teacher_error(err: TeacherError) -> HttpResponse {
    match err {
        TeacherError::NotFound(msg) => not_found(&msg),
        TeacherError::ValidationError(msg) => bad_request(&msg),
        TeacherError::DatabaseError(msg) => {
            log::error!("[Teacher] 数据库错误 | error={}", msg);
            internal_error("服务器内部错误")
        }
    }
}

/// 获取有效教师列表（公开API）
#[get("/teachers")]
async fn get_teachers(
    data: web::Data<AppState>,
    query: web::Query<GetTeachersQuery>,
) -> impl Responder {
    log::info!(
        "[Teacher] 获取有效教师列表 | with_resources_only={}",
        query.with_resources_only
    );

    match TeacherService::get_active_teachers(&data.pool, query.with_resources_only).await {
        Ok(teachers) => {
            let response: Vec<serde_json::Value> = teachers
                .into_iter()
                .map(|t| {
                    serde_json::json!({
                        "sn": t.sn,
                        "name": t.name,
                        "department": t.department,
                    })
                })
                .collect();
            HttpResponse::Ok().json(response)
        }
        Err(e) => handle_teacher_error(e),
    }
}

/// 配置教师路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_teachers);
}
