use crate::utils::{deserialize_vec_i64, deserialize_vec_string};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;
use uuid::Uuid;

/// 资源类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Default)]
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
    #[default]
    Other,
}

impl std::fmt::Display for ResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ResourceType::WebMarkdown => "web_markdown",
            ResourceType::Ppt => "ppt",
            ResourceType::Pptx => "pptx",
            ResourceType::Doc => "doc",
            ResourceType::Docx => "docx",
            ResourceType::Pdf => "pdf",
            ResourceType::Txt => "txt",
            ResourceType::Jpeg => "jpeg",
            ResourceType::Jpg => "jpg",
            ResourceType::Png => "png",
            ResourceType::Zip => "zip",
            ResourceType::Other => "other",
        };
        f.write_str(s)
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
            "md", "markdown", "ppt", "pptx", "doc", "docx", "pdf", "txt", "jpeg", "jpg", "png",
            "zip",
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
            ResourceType::Pptx => {
                "application/vnd.openxmlformats-officedocument.presentationml.presentation"
            }
            ResourceType::Doc => "application/msword",
            ResourceType::Docx => {
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            }
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
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Default)]
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
    #[default]
    Other,
}

impl std::fmt::Display for ResourceCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ResourceCategory::ExamResult => "exam_result",
            ResourceCategory::LearningNote => "learning_note",
            ResourceCategory::PastPaper => "past_paper",
            ResourceCategory::Note => "note",
            ResourceCategory::ReviewOutline => "review_outline",
            ResourceCategory::Lecture => "lecture",
            ResourceCategory::Other => "other",
        };
        f.write_str(s)
    }
}

/// 审核状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Default)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AuditStatus {
    /// 待审核
    #[default]
    Pending,
    /// 已通过
    Approved,
    /// 已拒绝
    Rejected,
}

impl std::fmt::Display for AuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AuditStatus::Pending => "pending",
            AuditStatus::Approved => "approved",
            AuditStatus::Rejected => "rejected",
        };
        f.write_str(s)
    }
}

/// 资源结构体（对应数据库 resources 表）
#[derive(Debug, Clone, FromRow, Serialize)]
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
    pub storage_type: Option<String>,
    pub description: Option<String>,
}

/// 资源统计信息（对应数据库 resource_stats 表）
#[derive(Debug, Clone, FromRow, Serialize)]
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

/// 关联教师信息
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TeacherInfo {
    pub sn: i64,
    pub name: String,
    pub department: Option<String>,
}

/// 关联课程信息
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CourseInfo {
    pub sn: i64,
    pub name: String,
    pub semester: Option<String>,
    pub credits: Option<f64>,
}

/// 关联资源信息（简要信息，用于资源详情页展示）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RelatedResourceInfo {
    pub id: Uuid,
    pub title: String,
    pub resource_type: String,
    pub category: String,
    pub created_at: chrono::NaiveDateTime,
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
    /// 关联教师编号列表（可选）
    pub teacher_sns: Option<Vec<i64>>,
    /// 关联课程编号列表（可选）
    pub course_sns: Option<Vec<i64>>,
    /// 关联资源ID列表（可选）
    pub related_resource_ids: Option<Vec<Uuid>>,
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
    /// 关联的教师列表
    pub teachers: Vec<TeacherInfo>,
    /// 关联的课程列表
    pub courses: Vec<CourseInfo>,
    /// 关联的资源列表（该资源主动关联的其他资源）
    pub related_resources: Vec<RelatedResourceInfo>,
    /// 存储类型：local 或 oss
    pub storage_type: String,
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
    /// 忽略当前分类选择后，其他筛选条件下的分类资源数
    pub category_counts: HashMap<String, i64>,
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
    /// 存储类型：local 或 oss
    pub storage_type: String,
}

/// 资源列表查询参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceListQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub resource_type: Option<String>,
    pub category: Option<String>,
    /// 资源类型多选筛选
    #[serde(default, deserialize_with = "deserialize_vec_string")]
    pub resource_types: Vec<String>,
    /// 资源分类多选筛选
    #[serde(default, deserialize_with = "deserialize_vec_string")]
    pub categories: Vec<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    /// 关联教师编号列表（筛选）
    #[serde(default, deserialize_with = "deserialize_vec_i64")]
    pub teacher_sns: Vec<i64>,
    /// 关联课程编号列表（筛选）
    #[serde(default, deserialize_with = "deserialize_vec_i64")]
    pub course_sns: Vec<i64>,
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
    /// 资源类型多选筛选
    #[serde(default, deserialize_with = "deserialize_vec_string")]
    pub resource_types: Vec<String>,
    /// 资源分类多选筛选
    #[serde(default, deserialize_with = "deserialize_vec_string")]
    pub categories: Vec<String>,
    /// 关联教师编号列表（筛选）
    #[serde(default, deserialize_with = "deserialize_vec_i64")]
    pub teacher_sns: Vec<i64>,
    /// 关联课程编号列表（筛选）
    #[serde(default, deserialize_with = "deserialize_vec_i64")]
    pub course_sns: Vec<i64>,
}

impl ResourceListQuery {
    pub fn get_page(&self) -> i32 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn get_per_page(&self) -> i32 {
        self.per_page.unwrap_or(20).clamp(1, 1000)
    }

    /// 合并新多选参数与遗留单选参数
    pub fn get_resource_types(&self) -> Vec<String> {
        if self.resource_types.is_empty() {
            self.resource_type.iter().cloned().collect()
        } else {
            self.resource_types.clone()
        }
    }

    /// 合并新多选参数与遗留单选参数
    pub fn get_categories(&self) -> Vec<String> {
        if self.categories.is_empty() {
            self.category.iter().cloned().collect()
        } else {
            self.categories.clone()
        }
    }
}

impl ResourceSearchQuery {
    pub fn get_page(&self) -> i32 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn get_per_page(&self) -> i32 {
        self.per_page.unwrap_or(20).clamp(1, 1000)
    }

    /// 合并新多选参数与遗留单选参数
    pub fn get_resource_types(&self) -> Vec<String> {
        if self.resource_types.is_empty() {
            self.resource_type.iter().cloned().collect()
        } else {
            self.resource_types.clone()
        }
    }

    /// 合并新多选参数与遗留单选参数
    pub fn get_categories(&self) -> Vec<String> {
        if self.categories.is_empty() {
            self.category.iter().cloned().collect()
        } else {
            self.categories.clone()
        }
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

/// 更新资源关联信息请求 DTO
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateResourceRelationsRequest {
    /// 关联教师编号列表
    pub teacher_sns: Vec<i64>,
    /// 关联课程编号列表
    pub course_sns: Vec<i64>,
    /// 关联资源ID列表
    pub related_resource_ids: Vec<Uuid>,
}

/// 更新资源描述请求 DTO
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateResourceDescriptionRequest {
    pub description: Option<String>,
}

impl UpdateResourceDescriptionRequest {
    /// 验证请求
    pub fn validate(&self) -> Result<(), String> {
        // 描述长度限制（10KB）
        if let Some(desc) = &self.description {
            if desc.len() > 10 * 1024 {
                return Err("资源描述不能超过10KB".to_string());
            }
        }
        Ok(())
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    mod resource_type_tests {
        use super::*;

        #[test]
        fn test_from_extension_markdown() {
            assert_eq!(
                ResourceType::from_extension("md"),
                ResourceType::WebMarkdown
            );
            assert_eq!(
                ResourceType::from_extension("markdown"),
                ResourceType::WebMarkdown
            );
            assert_eq!(
                ResourceType::from_extension("MD"),
                ResourceType::WebMarkdown
            );
        }

        #[test]
        fn test_from_extension_pdf() {
            assert_eq!(ResourceType::from_extension("pdf"), ResourceType::Pdf);
            assert_eq!(ResourceType::from_extension("PDF"), ResourceType::Pdf);
        }

        #[test]
        fn test_from_extension_ppt() {
            assert_eq!(ResourceType::from_extension("ppt"), ResourceType::Ppt);
            assert_eq!(ResourceType::from_extension("pptx"), ResourceType::Pptx);
        }

        #[test]
        fn test_from_extension_doc() {
            assert_eq!(ResourceType::from_extension("doc"), ResourceType::Doc);
            assert_eq!(ResourceType::from_extension("docx"), ResourceType::Docx);
        }

        #[test]
        fn test_from_extension_image() {
            assert_eq!(ResourceType::from_extension("jpg"), ResourceType::Jpg);
            assert_eq!(ResourceType::from_extension("jpeg"), ResourceType::Jpeg);
            assert_eq!(ResourceType::from_extension("png"), ResourceType::Png);
        }

        #[test]
        fn test_from_extension_unknown() {
            assert_eq!(ResourceType::from_extension("exe"), ResourceType::Other);
            assert_eq!(ResourceType::from_extension("unknown"), ResourceType::Other);
            assert_eq!(ResourceType::from_extension(""), ResourceType::Other);
        }

        #[test]
        fn test_supported_extensions() {
            let extensions = ResourceType::supported_extensions();
            assert!(extensions.contains(&"md"));
            assert!(extensions.contains(&"pdf"));
            assert!(extensions.contains(&"jpg"));
            assert!(extensions.contains(&"zip"));
            assert!(!extensions.contains(&"exe"));
        }

        #[test]
        fn test_is_previewable() {
            assert!(ResourceType::WebMarkdown.is_previewable());
            assert!(ResourceType::Pdf.is_previewable());
            assert!(ResourceType::Jpg.is_previewable());
            assert!(!ResourceType::Ppt.is_previewable());
            assert!(!ResourceType::Zip.is_previewable());
        }

        #[test]
        fn test_mime_type() {
            assert_eq!(ResourceType::Pdf.mime_type(), "application/pdf");
            assert_eq!(ResourceType::Jpg.mime_type(), "image/jpeg");
            assert_eq!(ResourceType::WebMarkdown.mime_type(), "text/markdown");
        }

        #[test]
        fn test_default_resource_type() {
            assert_eq!(ResourceType::default(), ResourceType::Other);
        }

        #[test]
        fn test_resource_type_to_string() {
            assert_eq!(ResourceType::Pdf.to_string(), "pdf");
            assert_eq!(ResourceType::WebMarkdown.to_string(), "web_markdown");
        }
    }

    mod resource_category_tests {
        use super::*;

        #[test]
        fn test_default_category() {
            assert_eq!(ResourceCategory::default(), ResourceCategory::Other);
        }

        #[test]
        fn test_category_to_string() {
            assert_eq!(ResourceCategory::ExamResult.to_string(), "exam_result");
            assert_eq!(ResourceCategory::Note.to_string(), "note");
            assert_eq!(ResourceCategory::Other.to_string(), "other");
        }
    }

    mod audit_status_tests {
        use super::*;

        #[test]
        fn test_default_audit_status() {
            assert_eq!(AuditStatus::default(), AuditStatus::Pending);
        }

        #[test]
        fn test_audit_status_to_string() {
            assert_eq!(AuditStatus::Pending.to_string(), "pending");
            assert_eq!(AuditStatus::Approved.to_string(), "approved");
            assert_eq!(AuditStatus::Rejected.to_string(), "rejected");
        }
    }

    mod resource_stats_tests {
        use super::*;

        fn create_test_stats(
            difficulty_total: i32,
            difficulty_count: i32,
            overall_quality_total: i32,
            overall_quality_count: i32,
        ) -> ResourceStats {
            ResourceStats {
                resource_id: Uuid::new_v4(),
                views: 0,
                downloads: 0,
                likes: 0,
                difficulty_total,
                difficulty_count,
                overall_quality_total,
                overall_quality_count,
                answer_quality_total: 0,
                answer_quality_count: 0,
                format_quality_total: 0,
                format_quality_count: 0,
                detail_level_total: 0,
                detail_level_count: 0,
            }
        }

        #[test]
        fn test_avg_difficulty_with_ratings() {
            let stats = create_test_stats(25, 5, 0, 0);
            assert_eq!(stats.avg_difficulty(), Some(5.0));
        }

        #[test]
        fn test_avg_difficulty_no_ratings() {
            let stats = create_test_stats(0, 0, 0, 0);
            assert_eq!(stats.avg_difficulty(), None);
        }

        #[test]
        fn test_avg_overall_quality_with_ratings() {
            let stats = create_test_stats(0, 0, 35, 7);
            assert_eq!(stats.avg_overall_quality(), Some(5.0));
        }

        #[test]
        fn test_avg_overall_quality_no_ratings() {
            let stats = create_test_stats(0, 0, 0, 0);
            assert_eq!(stats.avg_overall_quality(), None);
        }

        #[test]
        fn test_rating_count_single_dimension() {
            let stats = ResourceStats {
                resource_id: Uuid::new_v4(),
                views: 0,
                downloads: 0,
                likes: 0,
                difficulty_total: 10,
                difficulty_count: 2,
                overall_quality_total: 0,
                overall_quality_count: 0,
                answer_quality_total: 0,
                answer_quality_count: 0,
                format_quality_total: 0,
                format_quality_count: 0,
                detail_level_total: 0,
                detail_level_count: 0,
            };
            assert_eq!(stats.rating_count(), 2);
        }

        #[test]
        fn test_rating_count_multiple_dimensions() {
            let stats = ResourceStats {
                resource_id: Uuid::new_v4(),
                views: 0,
                downloads: 0,
                likes: 0,
                difficulty_total: 10,
                difficulty_count: 3,
                overall_quality_total: 20,
                overall_quality_count: 5,
                answer_quality_total: 0,
                answer_quality_count: 0,
                format_quality_total: 0,
                format_quality_count: 0,
                detail_level_total: 0,
                detail_level_count: 0,
            };
            assert_eq!(stats.rating_count(), 5);
        }

        #[test]
        fn test_rating_count_no_ratings() {
            let stats = create_test_stats(0, 0, 0, 0);
            assert_eq!(stats.rating_count(), 0);
        }
    }

    mod upload_resource_request_tests {
        use super::*;

        fn create_upload_request(title: &str, tags: Option<Vec<String>>) -> UploadResourceRequest {
            UploadResourceRequest {
                title: title.to_string(),
                course_name: None,
                resource_type: ResourceType::Pdf,
                category: ResourceCategory::Note,
                tags,
                description: None,
                teacher_sns: None,
                course_sns: None,
                related_resource_ids: None,
            }
        }

        #[test]
        fn test_valid_upload_request() {
            let req = create_upload_request("Valid Title", None);
            assert!(req.validate().is_ok());
        }

        #[test]
        fn test_empty_title() {
            let req = create_upload_request("", None);
            assert!(req.validate().is_err());
            assert!(req.validate().unwrap_err().contains("标题不能为空"));
        }

        #[test]
        fn test_whitespace_only_title() {
            let req = create_upload_request("   ", None);
            assert!(req.validate().is_err());
        }

        #[test]
        fn test_title_too_long() {
            let req = create_upload_request(&"a".repeat(256), None);
            assert!(req.validate().is_err());
            assert!(req.validate().unwrap_err().contains("不能超过255个字符"));
        }

        #[test]
        fn test_title_max_length() {
            let req = create_upload_request(&"a".repeat(255), None);
            assert!(req.validate().is_ok());
        }

        #[test]
        fn test_valid_tags() {
            let tags = vec!["tag1".to_string(), "tag2".to_string()];
            let req = create_upload_request("Title", Some(tags));
            assert!(req.validate().is_ok());
        }

        #[test]
        fn test_too_many_tags() {
            let tags: Vec<String> = (0..11).map(|i| format!("tag{}", i)).collect();
            let req = create_upload_request("Title", Some(tags));
            assert!(req.validate().is_err());
            assert!(req.validate().unwrap_err().contains("不能超过10个"));
        }

        #[test]
        fn test_tag_too_long() {
            let tags = vec!["a".repeat(51)];
            let req = create_upload_request("Title", Some(tags));
            assert!(req.validate().is_err());
            assert!(req.validate().unwrap_err().contains("不能超过50个字符"));
        }
    }

    mod resource_list_query_tests {
        use super::*;

        #[test]
        fn test_default_page() {
            let query = ResourceListQuery {
                page: None,
                per_page: None,
                resource_type: None,
                category: None,
                resource_types: vec![],
                categories: vec![],
                sort_by: None,
                sort_order: None,
                teacher_sns: vec![],
                course_sns: vec![],
            };
            assert_eq!(query.get_page(), 1);
        }

        #[test]
        fn test_custom_page() {
            let query = ResourceListQuery {
                page: Some(5),
                per_page: None,
                resource_type: None,
                category: None,
                resource_types: vec![],
                categories: vec![],
                sort_by: None,
                sort_order: None,
                teacher_sns: vec![],
                course_sns: vec![],
            };
            assert_eq!(query.get_page(), 5);
        }

        #[test]
        fn test_page_less_than_one() {
            let query = ResourceListQuery {
                page: Some(0),
                per_page: None,
                resource_type: None,
                category: None,
                resource_types: vec![],
                categories: vec![],
                sort_by: None,
                sort_order: None,
                teacher_sns: vec![],
                course_sns: vec![],
            };
            assert_eq!(query.get_page(), 1);
        }

        #[test]
        fn test_default_per_page() {
            let query = ResourceListQuery {
                page: None,
                per_page: None,
                resource_type: None,
                category: None,
                resource_types: vec![],
                categories: vec![],
                sort_by: None,
                sort_order: None,
                teacher_sns: vec![],
                course_sns: vec![],
            };
            assert_eq!(query.get_per_page(), 20);
        }

        #[test]
        fn test_custom_per_page() {
            let query = ResourceListQuery {
                page: None,
                per_page: Some(50),
                resource_type: None,
                category: None,
                resource_types: vec![],
                categories: vec![],
                sort_by: None,
                sort_order: None,
                teacher_sns: vec![],
                course_sns: vec![],
            };
            assert_eq!(query.get_per_page(), 50);
        }

        #[test]
        fn test_per_page_too_high() {
            let query = ResourceListQuery {
                page: None,
                per_page: Some(2000),
                resource_type: None,
                category: None,
                resource_types: vec![],
                categories: vec![],
                sort_by: None,
                sort_order: None,
                teacher_sns: vec![],
                course_sns: vec![],
            };
            assert_eq!(query.get_per_page(), 1000);
        }

        #[test]
        fn test_per_page_less_than_one() {
            let query = ResourceListQuery {
                page: None,
                per_page: Some(0),
                resource_type: None,
                category: None,
                resource_types: vec![],
                categories: vec![],
                sort_by: None,
                sort_order: None,
                teacher_sns: vec![],
                course_sns: vec![],
            };
            assert_eq!(query.get_per_page(), 1);
        }

        #[test]
        fn test_multi_select_query_deserialization() {
            let query: ResourceListQuery = serde_json::from_value(serde_json::json!({
                "resourceTypes": "pdf,ppt,image",
                "categories": ["lecture", "past_paper"]
            }))
            .expect("多选查询参数应可反序列化");

            assert_eq!(query.resource_types, vec!["pdf", "ppt", "image"]);
            assert_eq!(query.categories, vec!["lecture", "past_paper"]);
        }

        #[test]
        fn test_legacy_single_select_query_is_preserved() {
            let query: ResourceListQuery = serde_json::from_value(serde_json::json!({
                "resourceType": "pdf",
                "category": "lecture"
            }))
            .expect("遗留单选查询参数应可反序列化");

            assert_eq!(query.get_resource_types(), vec!["pdf"]);
            assert_eq!(query.get_categories(), vec!["lecture"]);
        }
    }

    mod update_resource_content_request_tests {
        use super::*;

        fn create_update_request(content: &str) -> UpdateResourceContentRequest {
            UpdateResourceContentRequest {
                content: content.to_string(),
            }
        }

        #[test]
        fn test_valid_content() {
            let req = create_update_request("This is valid content");
            assert!(req.validate().is_ok());
        }

        #[test]
        fn test_empty_content() {
            let req = create_update_request("");
            assert!(req.validate().is_err());
            assert!(req.validate().unwrap_err().contains("内容不能为空"));
        }

        #[test]
        fn test_whitespace_only_content() {
            let req = create_update_request("   \n\t  ");
            assert!(req.validate().is_err());
        }

        #[test]
        fn test_content_too_large() {
            let req = create_update_request(&"a".repeat(10 * 1024 * 1024 + 1));
            assert!(req.validate().is_err());
            assert!(req.validate().unwrap_err().contains("超过10MB"));
        }

        #[test]
        fn test_content_max_size() {
            let req = create_update_request(&"a".repeat(10 * 1024 * 1024));
            assert!(req.validate().is_ok());
        }
    }
}
