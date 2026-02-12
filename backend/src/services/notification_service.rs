use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    CreateNotificationRequest, Notification, NotificationListQuery, NotificationListResponse,
    NotificationPriority, NotificationResponse, NotificationType, UnreadCountResponse,
};
use crate::services::ResourceError;
use chrono::NaiveDateTime;

/// 通知已读记录
#[derive(Debug, sqlx::FromRow)]
pub struct NotificationRead {
    pub id: Uuid,
    pub notification_id: Uuid,
    pub user_id: Uuid,
    pub read_at: NaiveDateTime,
}

/// 带已读状态的通知（查询结果）
#[derive(Debug, sqlx::FromRow)]
pub struct NotificationWithReadStatus {
    pub id: Uuid,
    pub recipient_id: Option<Uuid>,
    pub title: String,
    pub content: String,
    pub notification_type: String,
    pub priority: String,
    pub is_read: bool,
    pub link_url: Option<String>,
    pub created_at: NaiveDateTime,
}

pub struct NotificationService;

impl NotificationService {
    /// 创建通知
    pub async fn create_notification(
        pool: &PgPool,
        request: CreateNotificationRequest,
    ) -> Result<Notification, ResourceError> {
        let notification = sqlx::query_as::<_, Notification>(
            r#"
            INSERT INTO notifications
                (recipient_id, title, content, notification_type, priority, link_url)
            VALUES
                ($1, $2, $3, $4, $5, $6)
            RETURNING
                id, recipient_id, title, content, notification_type, priority,
                is_read, link_url, created_at
            "#,
        )
        .bind(request.recipient_id)
        .bind(request.title)
        .bind(request.content)
        .bind(request.notification_type.as_str())
        .bind(request.priority.as_str())
        .bind(request.link_url)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            log::error!("[NotificationService] 创建通知失败: {}", e);
            ResourceError::DatabaseError(e.to_string())
        })?;

        Ok(notification)
    }

    /// 获取用户的通知列表
    pub async fn get_notifications(
        pool: &PgPool,
        user_id: Uuid,
        query: NotificationListQuery,
    ) -> Result<NotificationListResponse, ResourceError> {
        let page = query.page.unwrap_or(1).max(1);
        let per_page = query.per_page.unwrap_or(20).min(100);
        let offset = (page - 1) * per_page;

        // 构建查询条件
        let unread_only = query.unread_only.unwrap_or(false);

        // 获取总数（特定用户 + 广播通知）
        let total = if unread_only {
            sqlx::query_scalar::<_, i64>(
                r#"
                SELECT COUNT(*) FROM notifications n
                WHERE (n.recipient_id = $1 OR n.recipient_id IS NULL)
                    AND (
                        -- 定向通知使用原表的 is_read
                        (n.recipient_id IS NOT NULL AND n.is_read = FALSE)
                        OR
                        -- 群发通知使用 notification_reads 表
                        (n.recipient_id IS NULL AND NOT EXISTS (
                            SELECT 1 FROM notification_reads nr
                            WHERE nr.notification_id = n.id AND nr.user_id = $1
                        ))
                    )
                "#,
            )
            .bind(user_id)
            .fetch_one(pool)
            .await
        } else {
            sqlx::query_scalar::<_, i64>(
                r#"
                SELECT COUNT(*) FROM notifications
                WHERE recipient_id = $1 OR recipient_id IS NULL
                "#,
            )
            .bind(user_id)
            .fetch_one(pool)
            .await
        }
        .map_err(|e| ResourceError::DatabaseError(e.to_string()))?;

        // 获取未读总数
        let unread_count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM notifications n
            WHERE (n.recipient_id = $1 OR n.recipient_id IS NULL)
                AND (
                    (n.recipient_id IS NOT NULL AND n.is_read = FALSE)
                    OR
                    (n.recipient_id IS NULL AND NOT EXISTS (
                        SELECT 1 FROM notification_reads nr
                        WHERE nr.notification_id = n.id AND nr.user_id = $1
                    ))
                )
            "#,
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| ResourceError::DatabaseError(e.to_string()))?;

        // 获取通知列表（包含已读状态计算）
        let notifications = sqlx::query_as::<_, NotificationWithReadStatus>(
            r#"
            SELECT
                n.id,
                n.recipient_id,
                n.title,
                n.content,
                n.notification_type,
                n.priority,
                CASE
                    WHEN n.recipient_id IS NOT NULL THEN n.is_read
                    ELSE EXISTS (
                        SELECT 1 FROM notification_reads nr
                        WHERE nr.notification_id = n.id AND nr.user_id = $1
                    )
                END as is_read,
                n.link_url,
                n.created_at
            FROM notifications n
            WHERE n.recipient_id = $1 OR n.recipient_id IS NULL
            ORDER BY n.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(|e| ResourceError::DatabaseError(e.to_string()))?;

        // 如果只需要未读，过滤掉已读的
        let notifications: Vec<NotificationWithReadStatus> = if unread_only {
            notifications.into_iter().filter(|n| !n.is_read).collect()
        } else {
            notifications
        };

        // 计算返回的总数
        let response_total = if unread_only {
            notifications.len() as i64
        } else {
            total
        };

        let notifications: Vec<NotificationResponse> = notifications
            .into_iter()
            .map(|n| NotificationResponse {
                id: n.id,
                recipient_id: n.recipient_id,
                title: n.title,
                content: n.content,
                notification_type: n.notification_type,
                priority: n.priority,
                is_read: n.is_read,
                link_url: n.link_url,
                created_at: n.created_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            })
            .collect();

        Ok(NotificationListResponse {
            notifications,
            total: response_total,
            page,
            per_page,
            unread_count,
        })
    }

    /// 标记单条通知为已读
    pub async fn mark_as_read(
        pool: &PgPool,
        notification_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, ResourceError> {
        // 先查询通知类型
        let notification = sqlx::query_as::<_, Notification>(
            "SELECT * FROM notifications WHERE id = $1"
        )
        .bind(notification_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| ResourceError::DatabaseError(e.to_string()))?;

        let notification = match notification {
            Some(n) => n,
            None => return Ok(false),
        };

        // 检查用户是否有权限查看此通知
        if notification.recipient_id.is_some() && notification.recipient_id != Some(user_id) {
            return Ok(false);
        }

        let rows_affected = if notification.recipient_id.is_some() {
            // 定向通知：更新原表的 is_read
            sqlx::query(
                "UPDATE notifications SET is_read = TRUE WHERE id = $1"
            )
            .bind(notification_id)
            .execute(pool)
            .await
            .map_err(|e| ResourceError::DatabaseError(e.to_string()))?
            .rows_affected()
        } else {
            // 群发通知：插入到 notification_reads 表
            sqlx::query(
                r#"
                INSERT INTO notification_reads (notification_id, user_id)
                VALUES ($1, $2)
                ON CONFLICT (notification_id, user_id) DO NOTHING
                "#
            )
            .bind(notification_id)
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(|e| ResourceError::DatabaseError(e.to_string()))?
            .rows_affected()
        };

        Ok(rows_affected > 0)
    }

    /// 标记所有通知为已读
    pub async fn mark_all_as_read(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<i64, ResourceError> {
        // 1. 标记所有定向通知为已读
        let direct_result = sqlx::query(
            r#"
            UPDATE notifications
            SET is_read = TRUE
            WHERE recipient_id = $1 AND is_read = FALSE
            "#,
        )
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| ResourceError::DatabaseError(e.to_string()))?;

        // 2. 为所有未读的群发通知插入已读记录
        let broadcast_result = sqlx::query(
            r#"
            INSERT INTO notification_reads (notification_id, user_id)
            SELECT n.id, $1
            FROM notifications n
            WHERE n.recipient_id IS NULL
              AND NOT EXISTS (
                  SELECT 1 FROM notification_reads nr
                  WHERE nr.notification_id = n.id AND nr.user_id = $1
              )
            "#,
        )
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| ResourceError::DatabaseError(e.to_string()))?;

        Ok((direct_result.rows_affected() + broadcast_result.rows_affected()) as i64)
    }

    /// 获取未读通知数量
    pub async fn get_unread_count(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<UnreadCountResponse, ResourceError> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM notifications n
            WHERE (n.recipient_id = $1 OR n.recipient_id IS NULL)
                AND (
                    (n.recipient_id IS NOT NULL AND n.is_read = FALSE)
                    OR
                    (n.recipient_id IS NULL AND NOT EXISTS (
                        SELECT 1 FROM notification_reads nr
                        WHERE nr.notification_id = n.id AND nr.user_id = $1
                    ))
                )
            "#,
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| ResourceError::DatabaseError(e.to_string()))?;

        Ok(UnreadCountResponse { count })
    }

    /// 获取高优先级通知（未读的）
    pub async fn get_priority_notifications(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<NotificationResponse>, ResourceError> {
        let notifications = sqlx::query_as::<_, NotificationWithReadStatus>(
            r#"
            SELECT
                n.id,
                n.recipient_id,
                n.title,
                n.content,
                n.notification_type,
                n.priority,
                CASE
                    WHEN n.recipient_id IS NOT NULL THEN n.is_read
                    ELSE EXISTS (
                        SELECT 1 FROM notification_reads nr
                        WHERE nr.notification_id = n.id AND nr.user_id = $1
                    )
                END as is_read,
                n.link_url,
                n.created_at
            FROM notifications n
            WHERE (n.recipient_id = $1 OR n.recipient_id IS NULL)
                AND n.priority = 'high'
                AND (
                    (n.recipient_id IS NOT NULL AND n.is_read = FALSE)
                    OR
                    (n.recipient_id IS NULL AND NOT EXISTS (
                        SELECT 1 FROM notification_reads nr
                        WHERE nr.notification_id = n.id AND nr.user_id = $1
                    ))
                )
            ORDER BY n.created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(|e| ResourceError::DatabaseError(e.to_string()))?;

        let notifications: Vec<NotificationResponse> = notifications
            .into_iter()
            .map(|n| NotificationResponse {
                id: n.id,
                recipient_id: n.recipient_id,
                title: n.title,
                content: n.content,
                notification_type: n.notification_type,
                priority: n.priority,
                is_read: n.is_read,
                link_url: n.link_url,
                created_at: n.created_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
            })
            .collect();

        Ok(notifications)
    }

    /// 关闭（标记已读）高优先级通知
    pub async fn dismiss_priority_notification(
        pool: &PgPool,
        notification_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, ResourceError> {
        // 先查询通知类型
        let notification = sqlx::query_as::<_, Notification>(
            "SELECT * FROM notifications WHERE id = $1 AND priority = 'high'"
        )
        .bind(notification_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| ResourceError::DatabaseError(e.to_string()))?;

        let notification = match notification {
            Some(n) => n,
            None => return Ok(false),
        };

        // 检查用户是否有权限查看此通知
        if notification.recipient_id.is_some() && notification.recipient_id != Some(user_id) {
            return Ok(false);
        }

        let rows_affected = if notification.recipient_id.is_some() {
            // 定向通知：更新原表的 is_read
            sqlx::query(
                "UPDATE notifications SET is_read = TRUE WHERE id = $1"
            )
            .bind(notification_id)
            .execute(pool)
            .await
            .map_err(|e| ResourceError::DatabaseError(e.to_string()))?
            .rows_affected()
        } else {
            // 群发通知：插入到 notification_reads 表
            sqlx::query(
                r#"
                INSERT INTO notification_reads (notification_id, user_id)
                VALUES ($1, $2)
                ON CONFLICT (notification_id, user_id) DO NOTHING
                "#
            )
            .bind(notification_id)
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(|e| ResourceError::DatabaseError(e.to_string()))?
            .rows_affected()
        };

        Ok(rows_affected > 0)
    }

    /// 创建评论通知（资源被评论时通知上传者）
    pub async fn create_comment_notification(
        pool: &PgPool,
        resource_id: Uuid,
        resource_title: &str,
        uploader_id: Uuid,
        commenter_name: &str,
    ) -> Result<(), ResourceError> {
        // 不给自己发通知
        // 注意：这里需要在调用处检查，因为我们不知道评论者ID

        let request = CreateNotificationRequest {
            recipient_id: Some(uploader_id),
            title: "您的资源收到新评论".to_string(),
            content: format!("用户 {} 评论了您的资源《{}》", commenter_name, resource_title),
            notification_type: NotificationType::CommentReply,
            priority: NotificationPriority::Normal,
            link_url: Some(format!("/resource/{}", resource_id)),
        };

        Self::create_notification(pool, request).await?;
        Ok(())
    }

    /// 创建评分通知（资源被评分时通知上传者）
    pub async fn create_rating_notification(
        pool: &PgPool,
        resource_id: Uuid,
        resource_title: &str,
        uploader_id: Uuid,
        rater_name: &str,
    ) -> Result<(), ResourceError> {
        let request = CreateNotificationRequest {
            recipient_id: Some(uploader_id),
            title: "您的资源收到新评分".to_string(),
            content: format!("用户 {} 评分了您的资源《{}》", rater_name, resource_title),
            notification_type: NotificationType::RatingReminder,
            priority: NotificationPriority::Normal,
            link_url: Some(format!("/resource/{}", resource_id)),
        };

        Self::create_notification(pool, request).await?;
        Ok(())
    }
}
