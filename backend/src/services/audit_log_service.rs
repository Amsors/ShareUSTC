use sqlx::PgPool;
use uuid::Uuid;

/// 审计日志服务错误类型
#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),
}

impl actix_web::ResponseError for AuditError {
    fn error_response(&self) -> actix_web::HttpResponse {
        match self {
            AuditError::Database(e) => {
                log::error!("[Audit] 数据库错误 | error={}", e);
                crate::utils::internal_error("服务器内部错误")
            }
        }
    }
}

/// 审计日志服务
pub struct AuditLogService;

/// 审计日志操作类型
#[derive(Debug, Clone)]
pub enum AuditAction {
    Login,
    Register,
    UploadResource,
    DownloadResource,
    DeleteResource,
    UpdateResource,
    CreateComment,
    DeleteComment,
    RateResource,
    LikeResource,
    UnlikeResource,
    CreateFavorite,
    UpdateProfile,
    AdminAction,
    PackDownload, // 打包下载收藏夹
}

impl ToString for AuditAction {
    fn to_string(&self) -> String {
        match self {
            AuditAction::Login => "login".to_string(),
            AuditAction::Register => "register".to_string(),
            AuditAction::UploadResource => "upload_resource".to_string(),
            AuditAction::DownloadResource => "download_resource".to_string(),
            AuditAction::DeleteResource => "delete_resource".to_string(),
            AuditAction::UpdateResource => "update_resource".to_string(),
            AuditAction::CreateComment => "create_comment".to_string(),
            AuditAction::DeleteComment => "delete_comment".to_string(),
            AuditAction::RateResource => "rate_resource".to_string(),
            AuditAction::LikeResource => "like_resource".to_string(),
            AuditAction::UnlikeResource => "unlike_resource".to_string(),
            AuditAction::CreateFavorite => "create_favorite".to_string(),
            AuditAction::UpdateProfile => "update_profile".to_string(),
            AuditAction::AdminAction => "admin_action".to_string(),
            AuditAction::PackDownload => "pack_download".to_string(),
        }
    }
}

impl AuditLogService {
    /// 记录审计日志
    pub async fn log(
        pool: &PgPool,
        user_id: Option<Uuid>,
        action: AuditAction,
        target_type: Option<&str>,
        target_id: Option<Uuid>,
        details: Option<serde_json::Value>,
        ip_address: Option<&str>,
    ) -> Result<(), AuditError> {
        let action_str = action.to_string();

        sqlx::query(
            r#"
            INSERT INTO audit_logs
                (user_id, action, target_type, target_id, details, ip_address)
            VALUES
                ($1, $2, $3, $4, $5, $6::inet)
            "#,
        )
        .bind(user_id)
        .bind(action_str)
        .bind(target_type)
        .bind(target_id)
        .bind(details)
        .bind(ip_address)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// 记录登录日志
    pub async fn log_login(
        pool: &PgPool,
        user_id: Uuid,
        username: &str,
        ip_address: Option<&str>,
    ) -> Result<(), AuditError> {
        let details = serde_json::json!({
            "username": username,
        });

        Self::log(
            pool,
            Some(user_id),
            AuditAction::Login,
            Some("user"),
            Some(user_id),
            Some(details),
            ip_address,
        )
        .await
    }

    /// 记录注册日志
    pub async fn log_register(
        pool: &PgPool,
        user_id: Uuid,
        username: &str,
        ip_address: Option<&str>,
    ) -> Result<(), AuditError> {
        let details = serde_json::json!({
            "username": username,
        });

        Self::log(
            pool,
            Some(user_id),
            AuditAction::Register,
            Some("user"),
            Some(user_id),
            Some(details),
            ip_address,
        )
        .await
    }

    /// 记录资源上传日志
    pub async fn log_upload_resource(
        pool: &PgPool,
        user_id: Uuid,
        resource_id: Uuid,
        resource_title: &str,
        resource_type: &str,
        ip_address: Option<&str>,
    ) -> Result<(), AuditError> {
        let details = serde_json::json!({
            "title": resource_title,
            "resource_type": resource_type,
        });

        Self::log(
            pool,
            Some(user_id),
            AuditAction::UploadResource,
            Some("resource"),
            Some(resource_id),
            Some(details),
            ip_address,
        )
        .await
    }

    /// 记录资源下载日志
    pub async fn log_download_resource(
        pool: &PgPool,
        user_id: Option<Uuid>,
        resource_id: Uuid,
        resource_title: &str,
        ip_address: Option<&str>,
    ) -> Result<(), AuditError> {
        let details = serde_json::json!({
            "title": resource_title,
        });

        Self::log(
            pool,
            user_id,
            AuditAction::DownloadResource,
            Some("resource"),
            Some(resource_id),
            Some(details),
            ip_address,
        )
        .await
    }

    /// 记录资源删除日志
    pub async fn log_delete_resource(
        pool: &PgPool,
        user_id: Uuid,
        resource_id: Uuid,
        resource_title: &str,
        ip_address: Option<&str>,
    ) -> Result<(), AuditError> {
        let details = serde_json::json!({
            "title": resource_title,
        });

        Self::log(
            pool,
            Some(user_id),
            AuditAction::DeleteResource,
            Some("resource"),
            Some(resource_id),
            Some(details),
            ip_address,
        )
        .await
    }

    /// 记录资源更新日志
    pub async fn log_update_resource(
        pool: &PgPool,
        user_id: Uuid,
        resource_id: Uuid,
        resource_title: &str,
        ip_address: Option<&str>,
    ) -> Result<(), AuditError> {
        let details = serde_json::json!({
            "title": resource_title,
            "updated_at": chrono::Local::now().to_rfc3339(),
        });

        Self::log(
            pool,
            Some(user_id),
            AuditAction::UpdateResource,
            Some("resource"),
            Some(resource_id),
            Some(details),
            ip_address,
        )
        .await
    }

    /// 记录创建收藏夹日志
    pub async fn log_create_favorite(
        pool: &PgPool,
        user_id: Uuid,
        favorite_id: Uuid,
        favorite_name: &str,
        ip_address: Option<&str>,
    ) -> Result<(), AuditError> {
        let details = serde_json::json!({
            "name": favorite_name,
        });

        Self::log(
            pool,
            Some(user_id),
            AuditAction::CreateFavorite,
            Some("favorite"),
            Some(favorite_id),
            Some(details),
            ip_address,
        )
        .await
    }

    /// 记录打包下载日志
    pub async fn log_pack_download(
        pool: &PgPool,
        user_id: Uuid,
        favorite_id: Uuid,
        favorite_name: &str,
        download_size: i64,
        resource_count: usize,
        ip_address: Option<&str>,
    ) -> Result<(), AuditError> {
        let details = serde_json::json!({
            "favorite_name": favorite_name,
            "download_size": download_size,
            "resource_count": resource_count,
        });

        Self::log(
            pool,
            Some(user_id),
            AuditAction::PackDownload,
            Some("favorite"),
            Some(favorite_id),
            Some(details),
            ip_address,
        )
        .await
    }

    /// 记录更新个人主页日志
    pub async fn log_update_profile(
        pool: &PgPool,
        user_id: Uuid,
        username: &str,
        ip_address: Option<&str>,
    ) -> Result<(), AuditError> {
        let details = serde_json::json!({
            "username": username,
            "updated_at": chrono::Local::now().to_rfc3339(),
        });

        Self::log(
            pool,
            Some(user_id),
            AuditAction::UpdateProfile,
            Some("user"),
            Some(user_id),
            Some(details),
            ip_address,
        )
        .await
    }

    /// 记录评分资源日志
    pub async fn log_rate_resource(
        pool: &PgPool,
        user_id: Uuid,
        resource_id: Uuid,
        resource_title: &str,
        overall_quality: i32,
        ip_address: Option<&str>,
    ) -> Result<(), AuditError> {
        let details = serde_json::json!({
            "title": resource_title,
            "overall_quality": overall_quality,
        });

        Self::log(
            pool,
            Some(user_id),
            AuditAction::RateResource,
            Some("resource"),
            Some(resource_id),
            Some(details),
            ip_address,
        )
        .await
    }

    /// 记录点赞资源日志
    pub async fn log_like_resource(
        pool: &PgPool,
        user_id: Uuid,
        resource_id: Uuid,
        resource_title: &str,
        is_like: bool, // true: 点赞, false: 取消点赞
        ip_address: Option<&str>,
    ) -> Result<(), AuditError> {
        let action = if is_like {
            AuditAction::LikeResource
        } else {
            AuditAction::UnlikeResource
        };
        let details = serde_json::json!({
            "title": resource_title,
            "action": if is_like { "like" } else { "unlike" },
        });

        Self::log(
            pool,
            Some(user_id),
            action,
            Some("resource"),
            Some(resource_id),
            Some(details),
            ip_address,
        )
        .await
    }

    /// 记录发表评论日志
    pub async fn log_create_comment(
        pool: &PgPool,
        user_id: Uuid,
        _resource_id: Uuid,
        comment_id: Uuid,
        resource_title: &str,
        ip_address: Option<&str>,
    ) -> Result<(), AuditError> {
        let details = serde_json::json!({
            "resource_title": resource_title,
        });

        Self::log(
            pool,
            Some(user_id),
            AuditAction::CreateComment,
            Some("comment"),
            Some(comment_id),
            Some(details),
            ip_address,
        )
        .await
    }

    /// 记录删除评论日志
    pub async fn log_delete_comment(
        pool: &PgPool,
        user_id: Uuid,
        comment_id: Uuid,
        is_admin: bool,
        ip_address: Option<&str>,
    ) -> Result<(), AuditError> {
        let details = serde_json::json!({
            "is_admin": is_admin,
            "deleted_by": if is_admin { "admin" } else { "user" },
        });

        Self::log(
            pool,
            Some(user_id),
            AuditAction::DeleteComment,
            Some("comment"),
            Some(comment_id),
            Some(details),
            ip_address,
        )
        .await
    }

    /// 记录发送通知日志（管理员）
    pub async fn log_send_notification(
        pool: &PgPool,
        admin_id: Uuid,
        title: &str,
        recipient_count: i32,
        ip_address: Option<&str>,
    ) -> Result<(), AuditError> {
        let details = serde_json::json!({
            "title": title,
            "recipient_count": recipient_count,
        });

        Self::log(
            pool,
            Some(admin_id),
            AuditAction::AdminAction,
            Some("notification"),
            None,
            Some(details),
            ip_address,
        )
        .await
    }

    /// 记录更新用户状态日志（管理员禁用/启用用户）
    pub async fn log_update_user_status(
        pool: &PgPool,
        admin_id: Uuid,
        target_user_id: Uuid,
        is_active: bool,
        ip_address: Option<&str>,
    ) -> Result<(), AuditError> {
        let details = serde_json::json!({
            "target_user_id": target_user_id,
            "action": if is_active { "enable" } else { "disable" },
            "is_active": is_active,
        });

        Self::log(
            pool,
            Some(admin_id),
            AuditAction::AdminAction,
            Some("user"),
            Some(target_user_id),
            Some(details),
            ip_address,
        )
        .await
    }

    /// 记录通用操作日志
    pub async fn log_action(
        pool: &PgPool,
        user_id: Uuid,
        action: &str,
        target_type: Option<&str>,
        target_id: Option<Uuid>,
        details: Option<serde_json::Value>,
        ip_address: Option<&str>,
    ) -> Result<(), AuditError> {
        let action_enum = match action {
            "delete_favorite_resources" => AuditAction::AdminAction,
            _ => AuditAction::AdminAction,
        };

        Self::log(
            pool,
            Some(user_id),
            action_enum,
            target_type,
            target_id,
            details,
            ip_address,
        )
        .await
    }
}
