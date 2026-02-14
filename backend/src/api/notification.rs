use actix_web::{get, put, web, HttpResponse, Responder};
use uuid::Uuid;

use crate::db::AppState;
use crate::models::{CurrentUser, NotificationListQuery};
use crate::services::NotificationService;
use crate::utils::{not_found, internal_error};

/// 获取通知列表
#[get("/notifications")]
pub async fn get_notifications(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    query: web::Query<NotificationListQuery>,
) -> impl Responder {
    match NotificationService::get_notifications(&state.pool, user.id, query.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            log::warn!("获取通知列表失败: {}", e);
            internal_error("获取通知列表失败")
        }
    }
}

/// 标记单条通知为已读
#[put("/notifications/{notification_id}/read")]
pub async fn mark_as_read(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let notification_id = path.into_inner();

    match NotificationService::mark_as_read(&state.pool, notification_id, user.id).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => not_found("通知不存在或无权访问"),
        Err(e) => {
            log::warn!("标记通知已读失败: {}", e);
            internal_error("操作失败")
        }
    }
}

/// 标记所有通知为已读
#[put("/notifications/read-all")]
pub async fn mark_all_as_read(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
) -> impl Responder {
    match NotificationService::mark_all_as_read(&state.pool, user.id).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({
            "markedCount": count
        })),
        Err(e) => {
            log::warn!("标记全部已读失败: {}", e);
            internal_error("操作失败")
        }
    }
}

/// 获取未读通知数量
#[get("/notifications/unread-count")]
pub async fn get_unread_count(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
) -> impl Responder {
    match NotificationService::get_unread_count(&state.pool, user.id).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            log::warn!("获取未读数量失败: {}", e);
            internal_error("获取失败")
        }
    }
}

/// 获取高优先级通知
#[get("/notifications/priority")]
pub async fn get_priority_notifications(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
) -> impl Responder {
    match NotificationService::get_priority_notifications(&state.pool, user.id).await {
        Ok(notifications) => HttpResponse::Ok().json(notifications),
        Err(e) => {
            log::warn!("获取高优先级通知失败: {}", e);
            internal_error("获取失败")
        }
    }
}

/// 关闭（标记已读）高优先级通知
#[put("/notifications/priority/{notification_id}/dismiss")]
pub async fn dismiss_priority_notification(
    state: web::Data<AppState>,
    user: web::ReqData<CurrentUser>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let notification_id = path.into_inner();

    match NotificationService::dismiss_priority_notification(&state.pool, notification_id, user.id).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => not_found("通知不存在或无权访问"),
        Err(e) => {
            log::warn!("关闭高优先级通知失败: {}", e);
            internal_error("操作失败")
        }
    }
}

/// 配置通知路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_notifications)
        .service(mark_as_read)
        .service(mark_all_as_read)
        .service(get_unread_count)
        .service(get_priority_notifications)
        .service(dismiss_priority_notification);
}
