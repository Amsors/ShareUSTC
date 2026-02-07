use crate::models::resource::AiAuditResult;

#[derive(Debug)]
pub enum AiError {
    #[allow(dead_code)]
    ApiError(String),
    #[allow(dead_code)]
    TimeoutError(String),
}

impl std::fmt::Display for AiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiError::ApiError(msg) => write!(f, "AI API 错误: {}", msg),
            AiError::TimeoutError(msg) => write!(f, "AI 请求超时: {}", msg),
        }
    }
}

impl std::error::Error for AiError {}

pub struct AiService;

impl AiService {
    /// 审核资源内容
    ///
    /// 当前阶段：默认返回通过
    /// 后续阶段：接入真实 AI API
    pub async fn audit_resource(
        _title: &str,
        _description: Option<&str>,
        _file_data: Option<&[u8]>,
    ) -> Result<AiAuditResult, AiError> {
        // TODO: 接入真实 AI 审核服务
        // 当前阶段默认返回通过

        log::debug!("AI 审核资源: 标题={}, 描述={:?}", _title, _description);

        // 模拟 AI 审核（直接通过）
        Ok(AiAuditResult {
            passed: true,
            reason: None,
            accuracy_score: Some(0.95), // 模拟高准确度评分
        })
    }

    /// 审核评论内容（预留接口）
    ///
    /// 当前阶段：默认返回通过
    #[allow(dead_code)]
    pub async fn audit_comment(_content: &str) -> Result<AiAuditResult, AiError> {
        // TODO: 接入真实 AI 评论审核服务
        // 当前阶段默认返回通过

        log::debug!("AI 审核评论: 内容长度={}", _content.len());

        Ok(AiAuditResult {
            passed: true,
            reason: None,
            accuracy_score: None,
        })
    }

    /// 推荐资源（预留接口）
    ///
    /// 当前阶段：返回空列表
    #[allow(dead_code)]
    pub async fn recommend_resources(
        _user_id: uuid::Uuid,
        _course_name: Option<&str>,
        _limit: i32,
    ) -> Result<Vec<uuid::Uuid>, AiError> {
        // TODO: 接入真实 AI 推荐服务
        // 当前阶段返回空列表

        log::debug!("AI 推荐资源: 用户ID={}", _user_id);

        Ok(vec![])
    }

    /// 获取 AI 不通过原因（用于人工审核参考）（预留接口）
    ///
    /// 当前阶段：返回固定文本
    #[allow(dead_code)]
    pub fn get_rejection_reason(_file_data: Option<&[u8]>) -> String {
        // TODO: 接入真实 AI 原因分析
        // 当前阶段返回通用原因
        "AI 审核检测到此资源可能包含不合适的内容，建议检查资源内容是否符合社区规范。".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_resource_passes() {
        let result = AiService::audit_resource("测试资源", None, None).await;
        assert!(result.is_ok());

        let audit_result = result.unwrap();
        assert!(audit_result.passed);
        assert!(audit_result.accuracy_score.is_some());
    }

    #[tokio::test]
    async fn test_audit_comment_passes() {
        let result = AiService::audit_comment("这是一条测试评论").await;
        assert!(result.is_ok());

        let audit_result = result.unwrap();
        assert!(audit_result.passed);
    }

    #[tokio::test]
    async fn test_recommend_resources_empty() {
        let user_id = uuid::Uuid::new_v4();
        let result = AiService::recommend_resources(user_id, None, 10).await;
        assert!(result.is_ok());

        let recommendations = result.unwrap();
        assert!(recommendations.is_empty());
    }
}
