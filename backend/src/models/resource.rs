use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 资源类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    /// 网页 Markdown
    WebMarkdown,
    /// PPT 文件
    Ppt,
    /// PPTX 文件
    Pptx,
    /// Word 文档
    Doc,
    /// Word 文档 (新格式)
    Docx,
    /// PDF 文件
    Pdf,
    /// 文本文件
    Txt,
    /// JPEG 图片
    Jpeg,
    /// JPG 图片
    Jpg,
    /// PNG 图片
    Png,
    /// ZIP 压缩包（源文件）
    Zip,
    /// 其他类型
    Other,
}

impl Default for ResourceType {
    fn default() -> Self {
        ResourceType::Other
    }
}

impl ToString for ResourceType {
    fn to_string(&self) -> String {
        match self {
            ResourceType::WebMarkdown => "web_markdown".to_string(),
            ResourceType::Ppt => "ppt".to_string(),
            ResourceType::Pptx => "pptx".to_string(),
            ResourceType::Doc => "doc".to_string(),
            ResourceType::Docx => "docx".to_string(),
            ResourceType::Pdf => "pdf".to_string(),
            ResourceType::Txt => "txt".to_string(),
            ResourceType::Jpeg => "jpeg".to_string(),
            ResourceType::Jpg => "jpg".to_string(),
            ResourceType::Png => "png".to_string(),
            ResourceType::Zip => "zip".to_string(),
            ResourceType::Other => "other".to_string(),
        }
    }
}

impl ResourceType {
    /// 从文件扩展名推断资源类型
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "md" | "markdown" => ResourceType::WebMarkdown,
            "ppt" => ResourceType::Ppt,
            "pptx" => ResourceType::Pptx,
            "doc" => ResourceType::Doc,
            "docx" => ResourceType::Docx,
            "pdf" => ResourceType::Pdf,
            "txt" => ResourceType::Txt,
            "jpeg" => ResourceType::Jpeg,
            "jpg" => ResourceType::Jpg,
            "png" => ResourceType::Png,
            "zip" => ResourceType::Zip,
            _ => ResourceType::Other,
        }
    }

    /// 获取支持的文件扩展名列表
    pub fn supported_extensions() -> Vec<&'static str> {
        vec![
            "md", "markdown", "ppt", "pptx", "doc", "docx",
            "pdf", "txt", "jpeg", "jpg", "png", "zip",
        ]
    }

    /// 检查是否支持预览（预留接口）
    #[allow(dead_code)]
    pub fn is_previewable(&self) -> bool {
        matches!(
            self,
            ResourceType::WebMarkdown
                | ResourceType::Pdf
                | ResourceType::Txt
                | ResourceType::Jpeg
                | ResourceType::Jpg
                | ResourceType::Png
        )
    }

    /// 获取 MIME 类型（预留接口）
    #[allow(dead_code)]
    pub fn mime_type(&self) -> &'static str {
        match self {
            ResourceType::WebMarkdown => "text/markdown",
            ResourceType::Ppt => "application/vnd.ms-powerpoint",
            ResourceType::Pptx => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            ResourceType::Doc => "application/msword",
            ResourceType::Docx => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            ResourceType::Pdf => "application/pdf",
            ResourceType::Txt => "text/plain",
            ResourceType::Jpeg => "image/jpeg",
            ResourceType::Jpg => "image/jpeg",
            ResourceType::Png => "image/png",
            ResourceType::Zip => "application/zip",
            ResourceType::Other => "application/octet-stream",
        }
    }
}

/// 资源分类枚举
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ResourceCategory {
    /// 考试成绩分布
    ExamResult,
    /// 学习心得
    LearningNote,
    /// 往年试卷
    PastPaper,
    /// 笔记
    Note,
    /// 复习提纲
    ReviewOutline,
    /// 讲义
    Lecture,
    /// 其他
    Other,
}

impl Default for ResourceCategory {
    fn default() -> Self {
        ResourceCategory::Other
    }
}

impl ToString for ResourceCategory {
    fn to_string(&self) -> String {
        match self {
            ResourceCategory::ExamResult => "exam_result".to_string(),
            ResourceCategory::LearningNote => "learning_note".to_string(),
            ResourceCategory::PastPaper => "past_paper".to_string(),
            ResourceCategory::Note => "note".to_string(),
            ResourceCategory::ReviewOutline => "review_outline".to_string(),
            ResourceCategory::Lecture => "lecture".to_string(),
            ResourceCategory::Other => "other".to_string(),
        }
    }
}

/// 审核状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AuditStatus {
    /// 待审核
    Pending,
    /// 已通过
    Approved,
    /// 已拒绝
    Rejected,
}

impl Default for AuditStatus {
    fn default() -> Self {
        AuditStatus::Pending
    }
}

impl ToString for AuditStatus {
    fn to_string(&self) -> String {
        match self {
            AuditStatus::Pending => "pending".to_string(),
            AuditStatus::Approved => "approved".to_string(),
            AuditStatus::Rejected => "rejected".to_string(),
        }
    }
}

/// 资源结构体（对应数据库 resources 表）
#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    pub id: Uuid,
    pub title: String,
    pub author_id: Option<Uuid>,
    pub uploader_id: Uuid,
    pub course_name: Option<String>,
    pub resource_type: String,
    pub category: String,
    pub tags: Option<serde_json::Value>,
    pub file_path: String,
    pub source_file_path: Option<String>,
    pub file_hash: Option<String>,
    pub file_size: Option<i64>,
    pub content_accuracy: Option<f64>,
    pub audit_status: String,
    pub ai_reject_reason: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// 资源统计信息（对应数据库 resource_stats 表）
#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceStats {
    pub resource_id: Uuid,
    pub views: i32,
    pub downloads: i32,
    pub likes: i32,
    // 各维度评分统计：总得分(total)和评分次数(count)
    // 数据库使用 INTEGER (INT4)
    pub difficulty_total: i32,
    pub difficulty_count: i32,
    pub overall_quality_total: i32,
    pub overall_quality_count: i32,
    pub answer_quality_total: i32,
    pub answer_quality_count: i32,
    pub format_quality_total: i32,
    pub format_quality_count: i32,
    pub detail_level_total: i32,
    pub detail_level_count: i32,
}

impl ResourceStats {
    /// 计算难度平均分
    pub fn avg_difficulty(&self) -> Option<f64> {
        if self.difficulty_count > 0 {
            Some(self.difficulty_total as f64 / self.difficulty_count as f64)
        } else {
            None
        }
    }

    /// 计算总体质量平均分
    pub fn avg_overall_quality(&self) -> Option<f64> {
        if self.overall_quality_count > 0 {
            Some(self.overall_quality_total as f64 / self.overall_quality_count as f64)
        } else {
            None
        }
    }

    /// 计算参考答案质量平均分
    pub fn avg_answer_quality(&self) -> Option<f64> {
        if self.answer_quality_count > 0 {
            Some(self.answer_quality_total as f64 / self.answer_quality_count as f64)
        } else {
            None
        }
    }

    /// 计算格式质量平均分
    pub fn avg_format_quality(&self) -> Option<f64> {
        if self.format_quality_count > 0 {
            Some(self.format_quality_total as f64 / self.format_quality_count as f64)
        } else {
            None
        }
    }

    /// 计算知识点详细程度平均分
    pub fn avg_detail_level(&self) -> Option<f64> {
        if self.detail_level_count > 0 {
            Some(self.detail_level_total as f64 / self.detail_level_count as f64)
        } else {
            None
        }
    }

    /// 获取评分人数（取各维度中的最大值）
    pub fn rating_count(&self) -> i32 {
        [
            self.difficulty_count,
            self.overall_quality_count,
            self.answer_quality_count,
            self.format_quality_count,
            self.detail_level_count,
        ]
        .iter()
        .max()
        .copied()
        .unwrap_or(0)
    }
}

/// 资源上传请求 DTO
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadResourceRequest {
    pub title: String,
    pub course_name: Option<String>,
    // 前端传入的资源类型，后端实际从文件扩展名推断（保留用于API兼容性）
    #[allow(dead_code)]
    pub resource_type: ResourceType,
    pub category: ResourceCategory,
    pub tags: Option<Vec<String>>,
    pub description: Option<String>,
}

impl UploadResourceRequest {
    /// 验证上传请求
    pub fn validate(&self) -> Result<(), String> {
        // 标题验证
        if self.title.trim().is_empty() {
            return Err("资源标题不能为空".to_string());
        }
        if self.title.len() > 255 {
            return Err("资源标题不能超过255个字符".to_string());
        }

        // 标签验证（如果提供）
        if let Some(tags) = &self.tags {
            if tags.len() > 10 {
                return Err("标签数量不能超过10个".to_string());
            }
            for tag in tags {
                if tag.len() > 50 {
                    return Err("单个标签不能超过50个字符".to_string());
                }
            }
        }

        Ok(())
    }
}

/// 资源上传响应 DTO
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadResourceResponse {
    pub id: Uuid,
    pub title: String,
    pub resource_type: String,
    pub audit_status: String,
    pub ai_message: Option<String>,
    pub created_at: NaiveDateTime,
}

/// 资源详情响应 DTO
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceDetailResponse {
    pub id: Uuid,
    pub title: String,
    pub author_id: Option<Uuid>,
    pub uploader_id: Uuid,
    pub course_name: Option<String>,
    pub resource_type: String,
    pub category: String,
    pub tags: Option<Vec<String>>,
    pub description: Option<String>,
    pub file_size: Option<i64>,
    pub audit_status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub stats: ResourceStatsResponse,
    pub uploader_name: Option<String>,
}

/// 资源统计响应 DTO
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceStatsResponse {
    pub views: i32,
    pub downloads: i32,
    pub likes: i32,
    /// 难度平均分
    pub avg_difficulty: Option<f64>,
    /// 总体质量平均分
    pub avg_overall_quality: Option<f64>,
    /// 参考答案质量平均分
    pub avg_answer_quality: Option<f64>,
    /// 格式质量平均分
    pub avg_format_quality: Option<f64>,
    /// 知识点详细程度平均分
    pub avg_detail_level: Option<f64>,
    /// 评分人数
    pub rating_count: i32,
}

/// 资源列表响应 DTO
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceListResponse {
    pub resources: Vec<ResourceListItem>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

/// 资源列表项 DTO
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceListItem {
    pub id: Uuid,
    pub title: String,
    pub course_name: Option<String>,
    pub resource_type: String,
    pub category: String,
    pub tags: Option<Vec<String>>,
    pub audit_status: String,
    pub created_at: NaiveDateTime,
    pub stats: ResourceStatsResponse,
    pub uploader_name: Option<String>,
}

/// 资源列表查询参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceListQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub resource_type: Option<String>,
    pub category: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

/// 资源搜索查询参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceSearchQuery {
    pub q: String,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub resource_type: Option<String>,
    pub category: Option<String>,
}

impl ResourceListQuery {
    pub fn get_page(&self) -> i32 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn get_per_page(&self) -> i32 {
        self.per_page.unwrap_or(20).min(100).max(1)
    }
}

impl ResourceSearchQuery {
    pub fn get_page(&self) -> i32 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn get_per_page(&self) -> i32 {
        self.per_page.unwrap_or(20).min(100).max(1)
    }
}

/// AI 审核结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAuditResult {
    pub passed: bool,
    pub reason: Option<String>,
    pub accuracy_score: Option<f64>,
}

/// 更新资源内容请求 DTO（用于Markdown在线编辑）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateResourceContentRequest {
    pub content: String,
}

impl UpdateResourceContentRequest {
    /// 验证请求
    pub fn validate(&self) -> Result<(), String> {
        // 内容不能为空
        if self.content.trim().is_empty() {
            return Err("内容不能为空".to_string());
        }
        // 内容长度限制（10MB）
        if self.content.len() > 10 * 1024 * 1024 {
            return Err("内容大小超过10MB限制".to_string());
        }
        Ok(())
    }
}

/// 更新资源内容响应 DTO
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateResourceContentResponse {
    pub id: Uuid,
    pub updated_at: chrono::NaiveDateTime,
}

/// 热门资源查询参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotResourcesQuery {
    pub limit: Option<i32>,
}

/// 热门资源列表项 DTO
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HotResourceItem {
    pub id: Uuid,
    pub title: String,
    pub course_name: Option<String>,
    pub resource_type: String,
    pub downloads: i32,
    pub views: i32,
    pub likes: i32,
}
