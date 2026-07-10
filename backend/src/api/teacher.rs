use actix_web::{get, web, HttpResponse};
use serde::Deserialize;

use crate::db::AppState;
use crate::services::TeacherService;

/// 查询参数：获取教师列表
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetTeachersQuery {
    /// 是否只返回有关联资源的教师
    #[serde(default)]
    with_resources_only: bool,
}

/// 获取有效教师列表（公开API）
#[get("/teachers")]
async fn get_teachers(
    data: web::Data<AppState>,
    query: web::Query<GetTeachersQuery>,
) -> Result<HttpResponse, crate::services::TeacherError> {
    log::info!(
        "[Teacher] 获取有效教师列表 | with_resources_only={}",
        query.with_resources_only
    );

    let teachers =
        TeacherService::get_active_teachers(&data.pool, query.with_resources_only).await?;
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
    Ok(HttpResponse::Ok().json(response))
}

/// 配置教师路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_teachers);
}
