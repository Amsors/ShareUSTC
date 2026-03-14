use actix_web::{get, web, HttpResponse, Responder};
use serde::Deserialize;

use crate::db::AppState;
use crate::services::{CourseError, CourseService};
use crate::utils::{bad_request, internal_error, not_found};

/// 查询参数：获取课程列表
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetCoursesQuery {
    /// 是否只返回有关联资源的课程
    #[serde(default)]
    with_resources_only: bool,
}

/// 将 CourseError 转换为 HttpResponse
fn handle_course_error(err: CourseError) -> HttpResponse {
    match err {
        CourseError::NotFound(msg) => not_found(&msg),
        CourseError::ValidationError(msg) => bad_request(&msg),
        CourseError::DatabaseError(msg) => {
            log::error!("[Course] 数据库错误 | error={}", msg);
            internal_error("服务器内部错误")
        }
    }
}

/// 获取有效课程列表（公开API）
#[get("/courses")]
async fn get_courses(
    data: web::Data<AppState>,
    query: web::Query<GetCoursesQuery>,
) -> impl Responder {
    log::info!(
        "[Course] 获取有效课程列表 | with_resources_only={}",
        query.with_resources_only
    );

    match CourseService::get_active_courses(&data.pool, query.with_resources_only).await {
        Ok(courses) => {
            let response: Vec<serde_json::Value> = courses
                .into_iter()
                .map(|c| {
                    serde_json::json!({
                        "sn": c.sn,
                        "name": c.name,
                        "semester": c.semester,
                        "credits": c.credits,
                    })
                })
                .collect();
            HttpResponse::Ok().json(response)
        }
        Err(e) => handle_course_error(e),
    }
}

/// 配置课程路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_courses);
}
