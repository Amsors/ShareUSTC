use actix_web::{post, web, HttpRequest, HttpResponse, Responder};

use crate::db::AppState;
use crate::models::CurrentUser;
use crate::services::{AdminService, AuditLogService, NotificationService};

use super::{
    check_admin,
    utils::{handle_admin_error, handle_resource_error},
};

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

    // 获取接收者数量（用户总数查询移入 service 层；查询失败应冒泡为 500，不再吞掉）
    let recipient_count = if req.target == "all" {
        // 广播给所有用户，获取用户总数
        match NotificationService::count_active_users(&data.pool).await {
            Ok(count) => count as i32,
            Err(e) => return handle_resource_error(e),
        }
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

/// 配置路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(send_notification);
}
