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
    pub sn: Option<i64>,
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
    /// 同步管理员权限
    /// 根据环境变量 ADMIN_USERNAMES 的配置，同步数据库中的管理员权限：
    /// 1. 配置文件中存在的用户 -> 赋予管理员权限
    /// 2. 配置文件中不存在的用户（但数据库中是管理员）-> 取消管理员权限
    pub async fn sync_admin_roles(pool: &PgPool, admin_usernames: &[String]) -> Result<(usize, usize), AdminError> {
        let admin_set: std::collections::HashSet<&str> = admin_usernames.iter().map(|s| s.as_str()).collect();

        // 1. 将配置中的管理员用户设置为 admin
        let mut granted_count = 0usize;
        for username in admin_usernames {
            if username.is_empty() {
                continue;
            }
            let result = sqlx::query(
                "UPDATE users SET role = 'admin', updated_at = NOW() WHERE username = $1 AND role != 'admin'"
            )
            .bind(username)
            .execute(pool)
            .await
            .map_err(|e| AdminError::DatabaseError(format!("更新管理员权限失败: {}", e)))?;

            if result.rows_affected() > 0 {
                log::info!("已为用户 '{}' 赋予管理员权限", username);
                granted_count += 1;
            }
        }

        // 2. 获取所有当前是 admin 的用户
        let current_admins: Vec<(String,)> = sqlx::query_as("SELECT username FROM users WHERE role = 'admin'")
            .fetch_all(pool)
            .await
            .map_err(|e| AdminError::DatabaseError(format!("查询当前管理员失败: {}", e)))?;

        // 3. 取消不在配置中的管理员权限
        let mut revoked_count = 0usize;
        for (username,) in current_admins {
            if !admin_set.contains(username.as_str()) {
                let result = sqlx::query(
                    "UPDATE users SET role = 'user', updated_at = NOW() WHERE username = $1"
                )
                .bind(&username)
                .execute(pool)
                .await
                .map_err(|e| AdminError::DatabaseError(format!("取消管理员权限失败: {}", e)))?;

                if result.rows_affected() > 0 {
                    log::info!("已取消用户 '{}' 的管理员权限", username);
                    revoked_count += 1;
                }
            }
        }

        log::info!("管理员权限同步完成: 赋予 {} 个, 取消 {} 个", granted_count, revoked_count);
        Ok((granted_count, revoked_count))
    }
}

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
                u.sn,
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

    /// 发送系统通知
    pub async fn send_notification(
        pool: &PgPool,
        request: SendNotificationRequest,
    ) -> Result<(), AdminError> {
        // 验证请求
        if request.title.trim().is_empty() {
            return Err(AdminError::ValidationError("通知标题不能为空".to_string()));
        }
        if request.content.trim().is_empty() {
            return Err(AdminError::ValidationError("通知内容不能为空".to_string()));
        }

        let target = request.get_target()?;

        match target {
            NotificationTarget::All => {
                // 群发通知 - recipient_id 为 NULL 表示全员通知
                sqlx::query(
                    r#"
                    INSERT INTO notifications
                        (recipient_id, title, content, notification_type, priority, link_url)
                    VALUES
                        (NULL, $1, $2, $3, $4, $5)
                    "#
                )
                .bind(&request.title)
                .bind(&request.content)
                .bind(&request.notification_type)
                .bind(&request.priority)
                .bind(request.link_url)
                .execute(pool)
                .await
                .map_err(|e| AdminError::DatabaseError(e.to_string()))?;
            }
            NotificationTarget::Specific(user_id) => {
                // 检查用户是否存在
                let user_exists: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)"
                )
                .bind(user_id)
                .fetch_one(pool)
                .await
                .map_err(|e| AdminError::DatabaseError(e.to_string()))?;

                if !user_exists {
                    return Err(AdminError::NotFound("指定用户不存在".to_string()));
                }

                // 定向发送
                sqlx::query(
                    r#"
                    INSERT INTO notifications
                        (recipient_id, title, content, notification_type, priority, link_url)
                    VALUES
                        ($1, $2, $3, $4, $5, $6)
                    "#
                )
                .bind(user_id)
                .bind(&request.title)
                .bind(&request.content)
                .bind(&request.notification_type)
                .bind(&request.priority)
                .bind(request.link_url)
                .execute(pool)
                .await
                .map_err(|e| AdminError::DatabaseError(e.to_string()))?;
            }
        }

        log::info!("通知发送成功: target={:?}, type={}", request.target, request.notification_type);
        Ok(())
    }

    /// 获取详细统计数据
    pub async fn get_detailed_stats(pool: &PgPool) -> Result<DetailedStats, AdminError> {
        // 用户统计
        let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE is_active = true")
            .fetch_one(pool)
            .await
            .map_err(|e| AdminError::DatabaseError(e.to_string()))?;

        let new_users_today: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM users WHERE DATE(created_at) = CURRENT_DATE"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AdminError::DatabaseError(e.to_string()))?;

        let new_users_week: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM users WHERE created_at >= CURRENT_DATE - INTERVAL '7 days'"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AdminError::DatabaseError(e.to_string()))?;

        let new_users_month: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM users WHERE created_at >= CURRENT_DATE - INTERVAL '30 days'"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AdminError::DatabaseError(e.to_string()))?;

        // 资源统计
        let total_resources: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM resources")
            .fetch_one(pool)
            .await
            .map_err(|e| AdminError::DatabaseError(e.to_string()))?;

        let pending_resources: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM resources WHERE audit_status = 'pending'"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AdminError::DatabaseError(e.to_string()))?;

        let approved_resources: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM resources WHERE audit_status = 'approved'"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AdminError::DatabaseError(e.to_string()))?;

        let rejected_resources: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM resources WHERE audit_status = 'rejected'"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AdminError::DatabaseError(e.to_string()))?;

        // 资源类型分布
        let resource_type_distribution: Vec<ResourceTypeStat> = sqlx::query_as(
            r#"
            SELECT resource_type as type, COUNT(*) as count
            FROM resources
            GROUP BY resource_type
            ORDER BY count DESC
            "#
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AdminError::DatabaseError(e.to_string()))?;

        // 下载统计
        let total_downloads: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM download_logs")
            .fetch_one(pool)
            .await
            .map_err(|e| AdminError::DatabaseError(e.to_string()))?;

        let downloads_today: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM download_logs WHERE DATE(downloaded_at) = CURRENT_DATE"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AdminError::DatabaseError(e.to_string()))?;

        let downloads_week: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM download_logs WHERE downloaded_at >= CURRENT_DATE - INTERVAL '7 days'"
        )
        .fetch_one(pool)
        .await
        .map_err(|e| AdminError::DatabaseError(e.to_string()))?;

        // 热门资源排行（前10）
        let top_resources: Vec<TopResource> = sqlx::query_as(
            r#"
            SELECT
                r.id,
                r.title,
                COUNT(dl.id) as download_count
            FROM resources r
            LEFT JOIN download_logs dl ON r.id = dl.resource_id
            GROUP BY r.id, r.title
            ORDER BY download_count DESC
            LIMIT 10
            "#
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AdminError::DatabaseError(e.to_string()))?;

        // 互动统计
        let total_comments: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM comments")
            .fetch_one(pool)
            .await
            .map_err(|e| AdminError::DatabaseError(e.to_string()))?;

        let total_ratings: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ratings")
            .fetch_one(pool)
            .await
            .map_err(|e| AdminError::DatabaseError(e.to_string()))?;

        let total_likes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM likes")
            .fetch_one(pool)
            .await
            .map_err(|e| AdminError::DatabaseError(e.to_string()))?;

        // 评分分布 - 使用5个维度的平均值
        let rating_distribution: Vec<RatingDistribution> = sqlx::query_as(
            r#"
            SELECT
                CASE
                    WHEN (difficulty + overall_quality + answer_quality + format_quality + detail_level) / 5.0 >= 9 THEN 'excellent'
                    WHEN (difficulty + overall_quality + answer_quality + format_quality + detail_level) / 5.0 >= 7 THEN 'good'
                    WHEN (difficulty + overall_quality + answer_quality + format_quality + detail_level) / 5.0 >= 5 THEN 'average'
                    WHEN (difficulty + overall_quality + answer_quality + format_quality + detail_level) / 5.0 >= 3 THEN 'poor'
                    ELSE 'bad'
                END as rating_range,
                COUNT(*) as count
            FROM ratings
            GROUP BY rating_range
            ORDER BY rating_range
            "#
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AdminError::DatabaseError(e.to_string()))?;

        Ok(DetailedStats {
            user_stats: UserStats {
                total_users,
                new_users_today,
                new_users_week,
                new_users_month,
            },
            resource_stats: ResourceStats {
                total_resources,
                pending_resources,
                approved_resources,
                rejected_resources,
                type_distribution: resource_type_distribution,
            },
            download_stats: DownloadStats {
                total_downloads,
                downloads_today,
                downloads_week,
                top_resources,
            },
            interaction_stats: InteractionStats {
                total_comments,
                total_ratings,
                total_likes,
                rating_distribution,
            },
        })
    }

    /// 获取操作日志列表
    pub async fn get_audit_logs(
        pool: &PgPool,
        query: AuditLogQuery,
    ) -> Result<AuditLogListResponse, AdminError> {
        let page = query.page.unwrap_or(1).max(1);
        let per_page = query.per_page.unwrap_or(20).min(100);
        let offset = (page - 1) * per_page;

        // 获取总数（使用参数化查询）
        let total: i64 = if let Some(ref action) = query.action {
            if let Some(user_id) = query.user_id {
                if let Some(ref start_date) = query.start_date {
                    if let Some(ref end_date) = query.end_date {
                        // action + user_id + start_date + end_date
                        sqlx::query_scalar(
                            r#"
                            SELECT COUNT(*) FROM audit_logs
                            WHERE action = $1 AND user_id = $2
                                AND created_at >= $3 AND created_at <= $4
                            "#
                        )
                        .bind(action)
                        .bind(user_id)
                        .bind(start_date)
                        .bind(end_date)
                        .fetch_one(pool)
                        .await
                    } else {
                        // action + user_id + start_date
                        sqlx::query_scalar(
                            r#"
                            SELECT COUNT(*) FROM audit_logs
                            WHERE action = $1 AND user_id = $2 AND created_at >= $3
                            "#
                        )
                        .bind(action)
                        .bind(user_id)
                        .bind(start_date)
                        .fetch_one(pool)
                        .await
                    }
                } else if let Some(ref end_date) = query.end_date {
                    // action + user_id + end_date
                    sqlx::query_scalar(
                        r#"
                        SELECT COUNT(*) FROM audit_logs
                        WHERE action = $1 AND user_id = $2 AND created_at <= $3
                        "#
                    )
                    .bind(action)
                    .bind(user_id)
                    .bind(end_date)
                    .fetch_one(pool)
                    .await
                } else {
                    // action + user_id
                    sqlx::query_scalar(
                        "SELECT COUNT(*) FROM audit_logs WHERE action = $1 AND user_id = $2"
                    )
                    .bind(action)
                    .bind(user_id)
                    .fetch_one(pool)
                    .await
                }
            } else if let Some(ref start_date) = query.start_date {
                if let Some(ref end_date) = query.end_date {
                    // action + start_date + end_date
                    sqlx::query_scalar(
                        r#"
                        SELECT COUNT(*) FROM audit_logs
                        WHERE action = $1 AND created_at >= $2 AND created_at <= $3
                        "#
                    )
                    .bind(action)
                    .bind(start_date)
                    .bind(end_date)
                    .fetch_one(pool)
                    .await
                } else {
                    // action + start_date
                    sqlx::query_scalar(
                        "SELECT COUNT(*) FROM audit_logs WHERE action = $1 AND created_at >= $2"
                    )
                    .bind(action)
                    .bind(start_date)
                    .fetch_one(pool)
                    .await
                }
            } else if let Some(ref end_date) = query.end_date {
                // action + end_date
                sqlx::query_scalar(
                    "SELECT COUNT(*) FROM audit_logs WHERE action = $1 AND created_at <= $2"
                )
                .bind(action)
                .bind(end_date)
                .fetch_one(pool)
                .await
            } else {
                // action only
                sqlx::query_scalar("SELECT COUNT(*) FROM audit_logs WHERE action = $1")
                    .bind(action)
                    .fetch_one(pool)
                    .await
            }
        } else if let Some(user_id) = query.user_id {
            if let Some(ref start_date) = query.start_date {
                if let Some(ref end_date) = query.end_date {
                    // user_id + start_date + end_date
                    sqlx::query_scalar(
                        r#"
                        SELECT COUNT(*) FROM audit_logs
                        WHERE user_id = $1 AND created_at >= $2 AND created_at <= $3
                        "#
                    )
                    .bind(user_id)
                    .bind(start_date)
                    .bind(end_date)
                    .fetch_one(pool)
                    .await
                } else {
                    // user_id + start_date
                    sqlx::query_scalar(
                        "SELECT COUNT(*) FROM audit_logs WHERE user_id = $1 AND created_at >= $2"
                    )
                    .bind(user_id)
                    .bind(start_date)
                    .fetch_one(pool)
                    .await
                }
            } else if let Some(ref end_date) = query.end_date {
                // user_id + end_date
                sqlx::query_scalar(
                    "SELECT COUNT(*) FROM audit_logs WHERE user_id = $1 AND created_at <= $2"
                )
                .bind(user_id)
                .bind(end_date)
                .fetch_one(pool)
                .await
            } else {
                // user_id only
                sqlx::query_scalar("SELECT COUNT(*) FROM audit_logs WHERE user_id = $1")
                    .bind(user_id)
                    .fetch_one(pool)
                    .await
            }
        } else if let Some(ref start_date) = query.start_date {
            if let Some(ref end_date) = query.end_date {
                // start_date + end_date
                sqlx::query_scalar(
                    "SELECT COUNT(*) FROM audit_logs WHERE created_at >= $1 AND created_at <= $2"
                )
                .bind(start_date)
                .bind(end_date)
                .fetch_one(pool)
                .await
            } else {
                // start_date only
                sqlx::query_scalar("SELECT COUNT(*) FROM audit_logs WHERE created_at >= $1")
                    .bind(start_date)
                    .fetch_one(pool)
                    .await
            }
        } else if let Some(ref end_date) = query.end_date {
            // end_date only
            sqlx::query_scalar("SELECT COUNT(*) FROM audit_logs WHERE created_at <= $1")
                .bind(end_date)
                .fetch_one(pool)
                .await
        } else {
            // no filter
            sqlx::query_scalar("SELECT COUNT(*) FROM audit_logs")
                .fetch_one(pool)
                .await
        }
        .map_err(|e| AdminError::DatabaseError(format!("查询总数失败: {}", e)))?;

        // 获取日志列表（使用参数化查询）
        let logs: Vec<AuditLogItem> = if let Some(ref action) = query.action {
            if let Some(user_id) = query.user_id {
                if let Some(ref start_date) = query.start_date {
                    if let Some(ref end_date) = query.end_date {
                        // action + user_id + start_date + end_date
                        sqlx::query_as(
                            r#"
                            SELECT
                                al.id, al.user_id, u.username as user_name,
                                al.action, al.target_type, al.target_id,
                                al.details, al.ip_address::text, al.created_at
                            FROM audit_logs al
                            LEFT JOIN users u ON al.user_id = u.id
                            WHERE al.action = $1 AND al.user_id = $2
                                AND al.created_at >= $3 AND al.created_at <= $4
                            ORDER BY al.created_at DESC
                            LIMIT $5 OFFSET $6
                            "#
                        )
                        .bind(action).bind(user_id).bind(start_date).bind(end_date)
                        .bind(per_page as i64).bind(offset as i64)
                        .fetch_all(pool).await
                    } else {
                        // action + user_id + start_date
                        sqlx::query_as(
                            r#"
                            SELECT
                                al.id, al.user_id, u.username as user_name,
                                al.action, al.target_type, al.target_id,
                                al.details, al.ip_address::text, al.created_at
                            FROM audit_logs al
                            LEFT JOIN users u ON al.user_id = u.id
                            WHERE al.action = $1 AND al.user_id = $2 AND al.created_at >= $3
                            ORDER BY al.created_at DESC
                            LIMIT $4 OFFSET $5
                            "#
                        )
                        .bind(action).bind(user_id).bind(start_date)
                        .bind(per_page as i64).bind(offset as i64)
                        .fetch_all(pool).await
                    }
                } else if let Some(ref end_date) = query.end_date {
                    // action + user_id + end_date
                    sqlx::query_as(
                        r#"
                        SELECT
                            al.id, al.user_id, u.username as user_name,
                            al.action, al.target_type, al.target_id,
                            al.details, al.ip_address::text, al.created_at
                        FROM audit_logs al
                        LEFT JOIN users u ON al.user_id = u.id
                        WHERE al.action = $1 AND al.user_id = $2 AND al.created_at <= $3
                        ORDER BY al.created_at DESC
                        LIMIT $4 OFFSET $5
                        "#
                    )
                    .bind(action).bind(user_id).bind(end_date)
                    .bind(per_page as i64).bind(offset as i64)
                    .fetch_all(pool).await
                } else {
                    // action + user_id
                    sqlx::query_as(
                        r#"
                        SELECT
                            al.id, al.user_id, u.username as user_name,
                            al.action, al.target_type, al.target_id,
                            al.details, al.ip_address::text, al.created_at
                        FROM audit_logs al
                        LEFT JOIN users u ON al.user_id = u.id
                        WHERE al.action = $1 AND al.user_id = $2
                        ORDER BY al.created_at DESC
                        LIMIT $3 OFFSET $4
                        "#
                    )
                    .bind(action).bind(user_id)
                    .bind(per_page as i64).bind(offset as i64)
                    .fetch_all(pool).await
                }
            } else if let Some(ref start_date) = query.start_date {
                if let Some(ref end_date) = query.end_date {
                    // action + start_date + end_date
                    sqlx::query_as(
                        r#"
                        SELECT
                            al.id, al.user_id, u.username as user_name,
                            al.action, al.target_type, al.target_id,
                            al.details, al.ip_address::text, al.created_at
                        FROM audit_logs al
                        LEFT JOIN users u ON al.user_id = u.id
                        WHERE al.action = $1 AND al.created_at >= $2 AND al.created_at <= $3
                        ORDER BY al.created_at DESC
                        LIMIT $4 OFFSET $5
                        "#
                    )
                    .bind(action).bind(start_date).bind(end_date)
                    .bind(per_page as i64).bind(offset as i64)
                    .fetch_all(pool).await
                } else {
                    // action + start_date
                    sqlx::query_as(
                        r#"
                        SELECT
                            al.id, al.user_id, u.username as user_name,
                            al.action, al.target_type, al.target_id,
                            al.details, al.ip_address::text, al.created_at
                        FROM audit_logs al
                        LEFT JOIN users u ON al.user_id = u.id
                        WHERE al.action = $1 AND al.created_at >= $2
                        ORDER BY al.created_at DESC
                        LIMIT $3 OFFSET $4
                        "#
                    )
                    .bind(action).bind(start_date)
                    .bind(per_page as i64).bind(offset as i64)
                    .fetch_all(pool).await
                }
            } else if let Some(ref end_date) = query.end_date {
                // action + end_date
                sqlx::query_as(
                    r#"
                    SELECT
                        al.id, al.user_id, u.username as user_name,
                        al.action, al.target_type, al.target_id,
                        al.details, al.ip_address::text, al.created_at
                    FROM audit_logs al
                    LEFT JOIN users u ON al.user_id = u.id
                    WHERE al.action = $1 AND al.created_at <= $2
                    ORDER BY al.created_at DESC
                    LIMIT $3 OFFSET $4
                    "#
                )
                .bind(action).bind(end_date)
                .bind(per_page as i64).bind(offset as i64)
                .fetch_all(pool).await
            } else {
                // action only
                sqlx::query_as(
                    r#"
                    SELECT
                        al.id, al.user_id, u.username as user_name,
                        al.action, al.target_type, al.target_id,
                        al.details, al.ip_address::text, al.created_at
                    FROM audit_logs al
                    LEFT JOIN users u ON al.user_id = u.id
                    WHERE al.action = $1
                    ORDER BY al.created_at DESC
                    LIMIT $2 OFFSET $3
                    "#
                )
                .bind(action)
                .bind(per_page as i64).bind(offset as i64)
                .fetch_all(pool).await
            }
        } else if let Some(user_id) = query.user_id {
            if let Some(ref start_date) = query.start_date {
                if let Some(ref end_date) = query.end_date {
                    // user_id + start_date + end_date
                    sqlx::query_as(
                        r#"
                        SELECT
                            al.id, al.user_id, u.username as user_name,
                            al.action, al.target_type, al.target_id,
                            al.details, al.ip_address::text, al.created_at
                        FROM audit_logs al
                        LEFT JOIN users u ON al.user_id = u.id
                        WHERE al.user_id = $1 AND al.created_at >= $2 AND al.created_at <= $3
                        ORDER BY al.created_at DESC
                        LIMIT $4 OFFSET $5
                        "#
                    )
                    .bind(user_id).bind(start_date).bind(end_date)
                    .bind(per_page as i64).bind(offset as i64)
                    .fetch_all(pool).await
                } else {
                    // user_id + start_date
                    sqlx::query_as(
                        r#"
                        SELECT
                            al.id, al.user_id, u.username as user_name,
                            al.action, al.target_type, al.target_id,
                            al.details, al.ip_address::text, al.created_at
                        FROM audit_logs al
                        LEFT JOIN users u ON al.user_id = u.id
                        WHERE al.user_id = $1 AND al.created_at >= $2
                        ORDER BY al.created_at DESC
                        LIMIT $3 OFFSET $4
                        "#
                    )
                    .bind(user_id).bind(start_date)
                    .bind(per_page as i64).bind(offset as i64)
                    .fetch_all(pool).await
                }
            } else if let Some(ref end_date) = query.end_date {
                // user_id + end_date
                sqlx::query_as(
                    r#"
                    SELECT
                        al.id, al.user_id, u.username as user_name,
                        al.action, al.target_type, al.target_id,
                        al.details, al.ip_address::text, al.created_at
                    FROM audit_logs al
                    LEFT JOIN users u ON al.user_id = u.id
                    WHERE al.user_id = $1 AND al.created_at <= $2
                    ORDER BY al.created_at DESC
                    LIMIT $3 OFFSET $4
                    "#
                )
                .bind(user_id).bind(end_date)
                .bind(per_page as i64).bind(offset as i64)
                .fetch_all(pool).await
            } else {
                // user_id only
                sqlx::query_as(
                    r#"
                    SELECT
                        al.id, al.user_id, u.username as user_name,
                        al.action, al.target_type, al.target_id,
                        al.details, al.ip_address::text, al.created_at
                    FROM audit_logs al
                    LEFT JOIN users u ON al.user_id = u.id
                        WHERE al.user_id = $1
                    ORDER BY al.created_at DESC
                    LIMIT $2 OFFSET $3
                    "#
                )
                .bind(user_id)
                .bind(per_page as i64).bind(offset as i64)
                .fetch_all(pool).await
            }
        } else if let Some(ref start_date) = query.start_date {
            if let Some(ref end_date) = query.end_date {
                // start_date + end_date
                sqlx::query_as(
                    r#"
                    SELECT
                        al.id, al.user_id, u.username as user_name,
                        al.action, al.target_type, al.target_id,
                        al.details, al.ip_address::text, al.created_at
                    FROM audit_logs al
                    LEFT JOIN users u ON al.user_id = u.id
                    WHERE al.created_at >= $1 AND al.created_at <= $2
                    ORDER BY al.created_at DESC
                    LIMIT $3 OFFSET $4
                    "#
                )
                .bind(start_date).bind(end_date)
                .bind(per_page as i64).bind(offset as i64)
                .fetch_all(pool).await
            } else {
                // start_date only
                sqlx::query_as(
                    r#"
                    SELECT
                        al.id, al.user_id, u.username as user_name,
                        al.action, al.target_type, al.target_id,
                        al.details, al.ip_address::text, al.created_at
                    FROM audit_logs al
                    LEFT JOIN users u ON al.user_id = u.id
                    WHERE al.created_at >= $1
                    ORDER BY al.created_at DESC
                    LIMIT $2 OFFSET $3
                    "#
                )
                .bind(start_date)
                .bind(per_page as i64).bind(offset as i64)
                .fetch_all(pool).await
            }
        } else if let Some(ref end_date) = query.end_date {
            // end_date only
            sqlx::query_as(
                r#"
                SELECT
                    al.id, al.user_id, u.username as user_name,
                    al.action, al.target_type, al.target_id,
                    al.details, al.ip_address::text, al.created_at
                FROM audit_logs al
                LEFT JOIN users u ON al.user_id = u.id
                WHERE al.created_at <= $1
                ORDER BY al.created_at DESC
                LIMIT $2 OFFSET $3
                "#
            )
            .bind(end_date)
            .bind(per_page as i64).bind(offset as i64)
            .fetch_all(pool).await
        } else {
            // no filter
            sqlx::query_as(
                r#"
                SELECT
                    al.id, al.user_id, u.username as user_name,
                    al.action, al.target_type, al.target_id,
                    al.details, al.ip_address::text, al.created_at
                FROM audit_logs al
                LEFT JOIN users u ON al.user_id = u.id
                ORDER BY al.created_at DESC
                LIMIT $1 OFFSET $2
                "#
            )
            .bind(per_page as i64).bind(offset as i64)
            .fetch_all(pool).await
        }
        .map_err(|e| AdminError::DatabaseError(format!("查询日志列表失败: {}", e)))?;

        // 转换为响应格式（处理日期序列化）
        let logs: Vec<AuditLogItemResponse> = logs.into_iter().map(AuditLogItemResponse::from).collect();

        Ok(AuditLogListResponse {
            logs,
            total,
            page,
            per_page,
        })
    }
}

/// 通知目标枚举
#[derive(Debug, Clone)]
pub enum NotificationTarget {
    All,           // 所有用户
    Specific(Uuid), // 特定用户
}

/// 发送通知请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendNotificationRequest {
    pub target: String, // "all" 或 "specific"
    pub user_id: Option<Uuid>, // 当 target 为 specific 时使用
    pub title: String,
    pub content: String,
    pub notification_type: String, // system, admin_message
    pub priority: String,          // normal, high
    pub link_url: Option<String>,
}

impl SendNotificationRequest {
    /// 获取通知目标
    pub fn get_target(&self) -> Result<NotificationTarget, AdminError> {
        match self.target.as_str() {
            "all" => Ok(NotificationTarget::All),
            "specific" => {
                self.user_id
                    .ok_or_else(|| AdminError::ValidationError("指定用户时必须提供 user_id".to_string()))
                    .map(NotificationTarget::Specific)
            }
            _ => Err(AdminError::ValidationError("target 必须是 all 或 specific".to_string())),
        }
    }
}

/// 用户统计
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserStats {
    pub total_users: i64,
    pub new_users_today: i64,
    pub new_users_week: i64,
    pub new_users_month: i64,
}

/// 资源类型统计
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ResourceTypeStat {
    #[sqlx(rename = "type")]
    pub resource_type: String,
    pub count: i64,
}

/// 资源统计
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceStats {
    pub total_resources: i64,
    pub pending_resources: i64,
    pub approved_resources: i64,
    pub rejected_resources: i64,
    pub type_distribution: Vec<ResourceTypeStat>,
}

/// 热门资源
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TopResource {
    pub id: Uuid,
    pub title: String,
    pub download_count: i64,
}

/// 下载统计
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadStats {
    pub total_downloads: i64,
    pub downloads_today: i64,
    pub downloads_week: i64,
    pub top_resources: Vec<TopResource>,
}

/// 评分分布
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RatingDistribution {
    pub rating_range: String,
    pub count: i64,
}

/// 互动统计
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InteractionStats {
    pub total_comments: i64,
    pub total_ratings: i64,
    pub total_likes: i64,
    pub rating_distribution: Vec<RatingDistribution>,
}

/// 详细统计数据
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetailedStats {
    pub user_stats: UserStats,
    pub resource_stats: ResourceStats,
    pub download_stats: DownloadStats,
    pub interaction_stats: InteractionStats,
}

/// 操作日志查询参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub action: Option<String>,
    pub user_id: Option<Uuid>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

/// 操作日志列表项
#[derive(Debug, sqlx::FromRow)]
pub struct AuditLogItem {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub user_name: Option<String>,
    pub action: String,
    pub target_type: Option<String>,
    pub target_id: Option<Uuid>,
    pub details: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub created_at: NaiveDateTime,
}

/// 操作日志响应项（用于序列化）
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogItemResponse {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub user_name: Option<String>,
    pub action: String,
    pub target_type: Option<String>,
    pub target_id: Option<Uuid>,
    pub details: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub created_at: String,
}

impl From<AuditLogItem> for AuditLogItemResponse {
    fn from(item: AuditLogItem) -> Self {
        Self {
            id: item.id,
            user_id: item.user_id,
            user_name: item.user_name,
            action: item.action,
            target_type: item.target_type,
            target_id: item.target_id,
            details: item.details,
            ip_address: item.ip_address,
            created_at: item.created_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
        }
    }
}

/// 操作日志列表响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogListResponse {
    pub logs: Vec<AuditLogItemResponse>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}
