use actix_web::{get, web, HttpResponse};
use serde::Deserialize;

use crate::db::AppState;
use crate::services::CourseService;

/// 查询参数：获取课程列表
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetCoursesQuery {
    /// 是否只返回有关联资源的课程
    #[serde(default)]
    with_resources_only: bool,
}

/// 获取有效课程列表（公开API）
#[get("/courses")]
async fn get_courses(
    data: web::Data<AppState>,
    query: web::Query<GetCoursesQuery>,
) -> Result<HttpResponse, crate::services::CourseError> {
    log::info!(
        "[Course] 获取有效课程列表 | with_resources_only={}",
        query.with_resources_only
    );

    let courses = CourseService::get_active_courses(&data.pool, query.with_resources_only).await?;
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
    Ok(HttpResponse::Ok().json(response))
}

/// 配置课程路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_courses);
}
