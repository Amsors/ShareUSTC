use sqlx::PgPool;
use uuid::Uuid;

/// 审计日志服务
pub struct AuditLogService;

/// 审计日志操作类型
#[derive(Debug, Clone)]
pub enum AuditAction {
    Login,
    Logout,
    Register,
    UploadResource,
    DownloadResource,
    DeleteResource,
    UpdateResource,
    CreateComment,
    DeleteComment,
    RateResource,
    LikeResource,
    CreateFavorite,
    UpdateProfile,
    AdminAction,
}

impl ToString for AuditAction {
    fn to_string(&self) -> String {
        match self {
            AuditAction::Login => "login".to_string(),
            AuditAction::Logout => "logout".to_string(),
            AuditAction::Register => "register".to_string(),
            AuditAction::UploadResource => "upload_resource".to_string(),
            AuditAction::DownloadResource => "download_resource".to_string(),
            AuditAction::DeleteResource => "delete_resource".to_string(),
            AuditAction::UpdateResource => "update_resource".to_string(),
            AuditAction::CreateComment => "create_comment".to_string(),
            AuditAction::DeleteComment => "delete_comment".to_string(),
            AuditAction::RateResource => "rate_resource".to_string(),
            AuditAction::LikeResource => "like_resource".to_string(),
            AuditAction::CreateFavorite => "create_favorite".to_string(),
            AuditAction::UpdateProfile => "update_profile".to_string(),
            AuditAction::AdminAction => "admin_action".to_string(),
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
    ) -> Result<(), sqlx::Error> {
        let action_str = action.to_string();

        sqlx::query(
            r#"
            INSERT INTO audit_logs
                (user_id, action, target_type, target_id, details, ip_address)
            VALUES
                ($1, $2, $3, $4, $5, $6::inet)
            "#
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
    ) -> Result<(), sqlx::Error> {
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
    ) -> Result<(), sqlx::Error> {
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
    ) -> Result<(), sqlx::Error> {
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
    ) -> Result<(), sqlx::Error> {
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
    ) -> Result<(), sqlx::Error> {
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
    ) -> Result<(), sqlx::Error> {
        let details = serde_json::json!({
            "title": resource_title,
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
}
