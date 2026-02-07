use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    CreateNotificationRequest, Notification, NotificationListQuery, NotificationListResponse,
    NotificationPriority, NotificationResponse, NotificationType, UnreadCountResponse,
};
use crate::services::ResourceError;

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
                SELECT COUNT(*) FROM notifications
                WHERE (recipient_id = $1 OR recipient_id IS NULL)
                    AND is_read = FALSE
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
            SELECT COUNT(*) FROM notifications
            WHERE (recipient_id = $1 OR recipient_id IS NULL)
                AND is_read = FALSE
            "#,
        )
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| ResourceError::DatabaseError(e.to_string()))?;

        // 获取通知列表
        let notifications = if unread_only {
            sqlx::query_as::<_, Notification>(
                r#"
                SELECT
                    id, recipient_id, title, content, notification_type,
                    priority, is_read, link_url, created_at
                FROM notifications
                WHERE (recipient_id = $1 OR recipient_id IS NULL)
                    AND is_read = FALSE
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(user_id)
            .bind(per_page)
            .bind(offset)
            .fetch_all(pool)
            .await
        } else {
            sqlx::query_as::<_, Notification>(
                r#"
                SELECT
                    id, recipient_id, title, content, notification_type,
                    priority, is_read, link_url, created_at
                FROM notifications
                WHERE recipient_id = $1 OR recipient_id IS NULL
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(user_id)
            .bind(per_page)
            .bind(offset)
            .fetch_all(pool)
            .await
        }
        .map_err(|e| ResourceError::DatabaseError(e.to_string()))?;

        let notifications: Vec<NotificationResponse> = notifications
            .into_iter()
            .map(NotificationResponse::from)
            .collect();

        Ok(NotificationListResponse {
            notifications,
            total,
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
        // 检查通知是否属于该用户（或者是广播通知）
        let result = sqlx::query(
            r#"
            UPDATE notifications
            SET is_read = TRUE
            WHERE id = $1 AND (recipient_id = $2 OR recipient_id IS NULL)
            "#,
        )
        .bind(notification_id)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| ResourceError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    /// 标记所有通知为已读
    pub async fn mark_all_as_read(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<i64, ResourceError> {
        let result = sqlx::query(
            r#"
            UPDATE notifications
            SET is_read = TRUE
            WHERE (recipient_id = $1 OR recipient_id IS NULL) AND is_read = FALSE
            "#,
        )
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| ResourceError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected() as i64)
    }

    /// 获取未读通知数量
    pub async fn get_unread_count(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<UnreadCountResponse, ResourceError> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM notifications
            WHERE (recipient_id = $1 OR recipient_id IS NULL)
                AND is_read = FALSE
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
        let notifications = sqlx::query_as::<_, Notification>(
            r#"
            SELECT
                id, recipient_id, title, content, notification_type,
                priority, is_read, link_url, created_at
            FROM notifications
            WHERE (recipient_id = $1 OR recipient_id IS NULL)
                AND priority = 'high'
                AND is_read = FALSE
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(|e| ResourceError::DatabaseError(e.to_string()))?;

        let notifications: Vec<NotificationResponse> = notifications
            .into_iter()
            .map(NotificationResponse::from)
            .collect();

        Ok(notifications)
    }

    /// 关闭（标记已读）高优先级通知
    pub async fn dismiss_priority_notification(
        pool: &PgPool,
        notification_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, ResourceError> {
        let result = sqlx::query(
            r#"
            UPDATE notifications
            SET is_read = TRUE
            WHERE id = $1
                AND (recipient_id = $2 OR recipient_id IS NULL)
                AND priority = 'high'
            "#,
        )
        .bind(notification_id)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| ResourceError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected() > 0)
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
