use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// 管理员服务错误类型
#[derive(Debug)]
pub enum AdminError {
    DatabaseError(String),
    NotFound(String),
    ValidationError(String),
    Forbidden(String),
}

impl std::fmt::Display for AdminError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdminError::DatabaseError(msg) => write!(f, "数据库错误: {}", msg),
            AdminError::NotFound(msg) => write!(f, "未找到: {}", msg),
            AdminError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
            AdminError::Forbidden(msg) => write!(f, "权限不足: {}", msg),
        }
    }
}

impl std::error::Error for AdminError {}

/// 仪表盘统计数据
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardStats {
    pub total_users: i64,
    pub total_resources: i64,
    pub total_downloads: i64,
    pub pending_resources: i64,
    pub pending_comments: i64,
    pub today_new_users: i64,
    pub today_new_resources: i64,
}

/// 管理员用户列表项
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AdminUserListItem {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub role: String,
    pub is_verified: bool,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
}

/// 用户列表响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminUserListResponse {
    pub users: Vec<AdminUserListItem>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

/// 用户状态更新请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserStatusRequest {
    pub is_active: bool,
}

/// 待审核资源列表项
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PendingResourceItem {
    pub id: Uuid,
    pub title: String,
    pub course_name: Option<String>,
    pub resource_type: String,
    pub category: String,
    pub uploader_id: Uuid,
    pub uploader_name: Option<String>,
    pub ai_reject_reason: Option<String>,
    pub created_at: NaiveDateTime,
}

/// 待审核资源列表响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingResourceListResponse {
    pub resources: Vec<PendingResourceItem>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

/// 资源审核请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditResourceRequest {
    pub status: String, // approved, rejected
    pub reason: Option<String>,
}

/// 管理员评论列表项
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AdminCommentItem {
    pub id: Uuid,
    pub resource_id: Uuid,
    pub resource_title: Option<String>,
    pub user_id: Uuid,
    pub user_name: Option<String>,
    pub content: String,
    pub audit_status: String,
    pub created_at: NaiveDateTime,
}

/// 评论列表响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminCommentListResponse {
    pub comments: Vec<AdminCommentItem>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

/// 管理员服务
pub struct AdminService;

impl AdminService {
    /// 获取仪表盘统计数据
    pub async fn get_dashboard_stats(pool: &PgPool) -> Result<DashboardStats, AdminError> {
        // 用户总数
        let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE is_active = true")
            .fetch_one(pool)
            .await
            .map_err(|e| AdminError::DatabaseError(e.to_string()))?;

        // 资源总数
        let total_resources: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM resources")
            .fetch_one(pool)
            .await
            .map_err(|e| AdminError::DatabaseError(e.to_string()))?;

        // 总下载量
        let total_downloads: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM download_logs")
            .fetch_one(pool)
            .await
            .map_err(|e| AdminError::DatabaseError(e.to_string()))?;

        // 待审核资源数
        let pending_resources: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM resources WHERE audit_status = 'pending'"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AdminError::DatabaseError(e.to_string()))?;

        // 待审核评论数
        let pending_comments: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM comments WHERE audit_status = 'pending'"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AdminError::DatabaseError(e.to_string()))?;

        // 今日新增用户
        let today_new_users: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM users WHERE DATE(created_at) = CURRENT_DATE"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AdminError::DatabaseError(e.to_string()))?;

        // 今日新增资源
        let today_new_resources: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM resources WHERE DATE(created_at) = CURRENT_DATE"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AdminError::DatabaseError(e.to_string()))?;

        Ok(DashboardStats {
            total_users,
            total_resources,
            total_downloads,
            pending_resources,
            pending_comments,
            today_new_users,
            today_new_resources,
        })
    }

    /// 获取用户列表
    pub async fn get_user_list(
        pool: &PgPool,
        page: i32,
        per_page: i32,
    ) -> Result<AdminUserListResponse, AdminError> {
        let offset = (page - 1) * per_page;

        // 获取用户列表
        let users: Vec<AdminUserListItem> = sqlx::query_as(
            r#"
            SELECT
                u.id,
                u.username,
                u.email,
                u.role,
                u.is_verified,
                u.is_active,
                u.created_at
            FROM users u
            ORDER BY u.created_at DESC
            LIMIT $1 OFFSET $2
            "#
        )
        .bind(per_page as i64)
        .bind(offset as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| AdminError::DatabaseError(e.to_string()))?;

        // 获取总数
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(pool)
            .await
            .map_err(|e| AdminError::DatabaseError(e.to_string()))?;

        Ok(AdminUserListResponse {
            users,
            total,
            page,
            per_page,
        })
    }

    /// 更新用户状态（禁用/启用）
    pub async fn update_user_status(
        pool: &PgPool,
        user_id: Uuid,
        is_active: bool,
    ) -> Result<(), AdminError> {
        let result = sqlx::query(
            "UPDATE users SET is_active = $1, updated_at = NOW() WHERE id = $2"
        )
        .bind(is_active)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| AdminError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AdminError::NotFound("用户不存在".to_string()));
        }

        Ok(())
    }

    /// 获取待审核资源列表
    pub async fn get_pending_resources(
        pool: &PgPool,
        page: i32,
        per_page: i32,
    ) -> Result<PendingResourceListResponse, AdminError> {
        let offset = (page - 1) * per_page;

        // 获取待审核资源
        let resources: Vec<PendingResourceItem> = sqlx::query_as(
            r#"
            SELECT
                r.id,
                r.title,
                r.course_name,
                r.resource_type,
                r.category,
                r.uploader_id,
                u.username as uploader_name,
                r.ai_reject_reason,
                r.created_at
            FROM resources r
            JOIN users u ON r.uploader_id = u.id
            WHERE r.audit_status = 'pending'
            ORDER BY r.created_at DESC
            LIMIT $1 OFFSET $2
            "#
        )
        .bind(per_page as i64)
        .bind(offset as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| AdminError::DatabaseError(e.to_string()))?;

        // 获取总数
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM resources WHERE audit_status = 'pending'"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AdminError::DatabaseError(e.to_string()))?;

        Ok(PendingResourceListResponse {
            resources,
            total,
            page,
            per_page,
        })
    }

    /// 审核资源
    pub async fn audit_resource(
        pool: &PgPool,
        resource_id: Uuid,
        status: String,
        reason: Option<String>,
    ) -> Result<(), AdminError> {
        // 验证状态值
        if status != "approved" && status != "rejected" {
            return Err(AdminError::ValidationError(
                "状态必须是 approved 或 rejected".to_string()
            ));
        }

        let result = sqlx::query(
            r#"
            UPDATE resources
            SET audit_status = $1,
                ai_reject_reason = $2,
                updated_at = NOW()
            WHERE id = $3
            "#
        )
        .bind(&status)
        .bind(reason)
        .bind(resource_id)
        .execute(pool)
        .await
        .map_err(|e| AdminError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AdminError::NotFound("资源不存在".to_string()));
        }

        log::info!("资源审核完成: id={}, status={}", resource_id, status);
        Ok(())
    }

    /// 获取评论列表
    pub async fn get_comment_list(
        pool: &PgPool,
        page: i32,
        per_page: i32,
        audit_status: Option<String>,
    ) -> Result<AdminCommentListResponse, AdminError> {
        let offset = (page - 1) * per_page;

        let mut query = String::from(
            r#"
            SELECT
                c.id,
                c.resource_id,
                r.title as resource_title,
                c.user_id,
                u.username as user_name,
                c.content,
                c.audit_status,
                c.created_at
            FROM comments c
            JOIN users u ON c.user_id = u.id
            JOIN resources r ON c.resource_id = r.id
            WHERE 1=1
            "#
        );

        let mut count_query = String::from(
            "SELECT COUNT(*) FROM comments c WHERE 1=1"
        );

        // 添加审核状态筛选
        if let Some(ref _status) = audit_status {
            query.push_str(" AND c.audit_status = $3");
            count_query.push_str(" AND c.audit_status = $1");
        }

        query.push_str(" ORDER BY c.created_at DESC LIMIT $1 OFFSET $2");

        // 执行查询
        let comments: Vec<AdminCommentItem> = if let Some(ref status) = audit_status {
            sqlx::query_as(&query)
                .bind(per_page as i64)
                .bind(offset as i64)
                .bind(status)
                .fetch_all(pool)
                .await
                .map_err(|e| AdminError::DatabaseError(e.to_string()))?
        } else {
            sqlx::query_as(&query)
                .bind(per_page as i64)
                .bind(offset as i64)
                .fetch_all(pool)
                .await
                .map_err(|e| AdminError::DatabaseError(e.to_string()))?
        };

        // 获取总数
        let total: i64 = if let Some(ref status) = audit_status {
            sqlx::query_scalar(&count_query)
                .bind(status)
                .fetch_one(pool)
                .await
                .map_err(|e| AdminError::DatabaseError(e.to_string()))?
        } else {
            sqlx::query_scalar(&count_query)
                .fetch_one(pool)
                .await
                .map_err(|e| AdminError::DatabaseError(e.to_string()))?
        };

        Ok(AdminCommentListResponse {
            comments,
            total,
            page,
            per_page,
        })
    }

    /// 删除评论
    pub async fn delete_comment(
        pool: &PgPool,
        comment_id: Uuid,
    ) -> Result<(), AdminError> {
        let result = sqlx::query("DELETE FROM comments WHERE id = $1")
            .bind(comment_id)
            .execute(pool)
            .await
            .map_err(|e| AdminError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AdminError::NotFound("评论不存在".to_string()));
        }

        log::info!("评论已删除: id={}", comment_id);
        Ok(())
    }

    /// 审核评论
    pub async fn audit_comment(
        pool: &PgPool,
        comment_id: Uuid,
        status: String,
    ) -> Result<(), AdminError> {
        if status != "approved" && status != "rejected" {
            return Err(AdminError::ValidationError(
                "状态必须是 approved 或 rejected".to_string()
            ));
        }

        let result = sqlx::query(
            "UPDATE comments SET audit_status = $1 WHERE id = $2"
        )
        .bind(&status)
        .bind(comment_id)
        .execute(pool)
        .await
        .map_err(|e| AdminError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AdminError::NotFound("评论不存在".to_string()));
        }

        log::info!("评论审核完成: id={}, status={}", comment_id, status);
        Ok(())
    }
}
