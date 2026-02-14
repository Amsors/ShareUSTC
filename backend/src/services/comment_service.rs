use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    Comment, CommentListQuery, CommentListResponse, CommentResponse, CreateCommentRequest,
};
use crate::services::{NotificationService, ResourceError};

pub struct CommentService;

impl CommentService {
    /// 创建评论
    pub async fn create_comment(
        pool: &PgPool,
        resource_id: Uuid,
        user_id: Uuid,
        request: CreateCommentRequest,
    ) -> Result<CommentResponse, ResourceError> {
        // 验证评论内容
        let content = request.content.trim();
        if content.is_empty() {
            return Err(ResourceError::ValidationError("评论内容不能为空".to_string()));
        }
        if content.len() > 1000 {
            return Err(ResourceError::ValidationError("评论内容不能超过1000字".to_string()));
        }

        // 验证资源是否存在
        let resource_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM resources WHERE id = $1)"
        )
        .bind(resource_id)
        .fetch_one(pool)
        .await
        .map_err(|e| ResourceError::DatabaseError(e.to_string()))?;

        if !resource_exists {
            return Err(ResourceError::NotFound(format!("资源 {} 不存在", resource_id)));
        }

        log::debug!("[CommentService] 开始创建评论: resource_id={}, user_id={}, content={}", resource_id, user_id, content);

        // 直接插入不使用事务（简化排查）
        let comment = sqlx::query_as::<_, Comment>(
            r#"
            INSERT INTO comments (resource_id, user_id, content)
            VALUES ($1, $2, $3)
            RETURNING id, resource_id, user_id, content, audit_status, created_at, updated_at
            "#,
        )
        .bind(resource_id)
        .bind(user_id)
        .bind(content)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            log::error!("[CommentService] 插入评论失败: {}", e);
            ResourceError::DatabaseError(e.to_string())
        })?;

        log::debug!("[CommentService] 评论插入成功: comment_id={}", comment.id);

        // 获取用户信息
        let (user_name, user_avatar) = match sqlx::query_as::<_, (String, Option<String>)>(
            "SELECT username, avatar_url FROM users WHERE id = $1"
        )
        .bind(user_id)
        .fetch_one(pool)
        .await {
            Ok((name, avatar)) => (name, avatar),
            Err(e) => {
                log::warn!("[CommentService] 获取用户信息失败: {}", e);
                ("未知用户".to_string(), None)
            }
        };

        log::debug!("[CommentService] 评论创建完成: comment_id={}, user_name={}", comment.id, user_name);

        // 发送通知给资源上传者（如果不是评论自己的资源）
        Self::notify_uploader_on_comment(pool, resource_id, user_id, &user_name).await;

        Ok(CommentResponse {
            id: comment.id,
            resource_id: comment.resource_id,
            user_id: comment.user_id,
            user_name,
            user_avatar,
            content: comment.content,
            created_at: comment.created_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
        })
    }

    /// 评论时通知资源上传者
    async fn notify_uploader_on_comment(
        pool: &PgPool,
        resource_id: Uuid,
        commenter_id: Uuid,
        commenter_name: &str,
    ) {
        // 获取资源上传者信息
        let result = sqlx::query_as::<_, (Uuid, String, Option<Uuid>)>(
            "SELECT uploader_id, title, author_id FROM resources WHERE id = $1"
        )
        .bind(resource_id)
        .fetch_optional(pool)
        .await;

        if let Ok(Some((uploader_id, resource_title, author_id))) = result {
            // 优先通知作者（如果存在），否则通知上传者
            let notify_user_id = author_id.unwrap_or(uploader_id);

            // 不给自己发通知
            if notify_user_id != commenter_id {
                if let Err(e) = NotificationService::create_comment_notification(
                    pool,
                    resource_id,
                    &resource_title,
                    notify_user_id,
                    commenter_name,
                ).await {
                    log::warn!("[CommentService] 发送评论通知失败: {}", e);
                }
            }
        }
    }

    /// 获取评论列表
    pub async fn get_comments(
        pool: &PgPool,
        resource_id: Uuid,
        query: CommentListQuery,
    ) -> Result<CommentListResponse, ResourceError> {
        let page = query.page.unwrap_or(1).max(1);
        let per_page = query.per_page.unwrap_or(20).min(100);
        let offset = (page - 1) * per_page;

        // 获取评论总数
        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM comments WHERE resource_id = $1 AND audit_status = 'approved'"
        )
        .bind(resource_id)
        .fetch_one(pool)
        .await?;

        // 获取评论列表
        let rows = sqlx::query!(
            r#"
            SELECT
                c.id,
                c.resource_id,
                c.user_id,
                c.content,
                c.created_at,
                u.username as user_name
            FROM comments c
            JOIN users u ON c.user_id = u.id
            WHERE c.resource_id = $1 AND c.audit_status = 'approved'
            ORDER BY c.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            resource_id,
            per_page,
            offset
        )
        .fetch_all(pool)
        .await?;

        let comments = rows
            .into_iter()
            .map(|row| CommentResponse {
                id: row.id,
                resource_id: row.resource_id,
                user_id: row.user_id,
                user_name: row.user_name,
                user_avatar: None,
                content: row.content,
                created_at: row.created_at.map(|dt| dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
                    .unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
            })
            .collect();

        Ok(CommentListResponse {
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
        user_id: Uuid,
        is_admin: bool,
    ) -> Result<bool, ResourceError> {
        // 检查评论是否存在且属于该用户（或用户是管理员）
        let comment = sqlx::query_as::<_, Comment>(
            "SELECT * FROM comments WHERE id = $1"
        )
        .bind(comment_id)
        .fetch_optional(pool)
        .await?;

        let comment = match comment {
            Some(c) => c,
            None => return Ok(false),
        };

        // 检查权限
        if comment.user_id != user_id && !is_admin {
            return Ok(false);
        }

        // 删除评论
        sqlx::query("DELETE FROM comments WHERE id = $1")
            .bind(comment_id)
            .execute(pool)
            .await?;

        Ok(true)
    }
}
