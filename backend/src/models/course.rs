use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 课程结构体（对应数据库 courses 表）
#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Course {
    pub id: Uuid,
    pub sn: i64,
    pub name: String,
    pub semester: Option<String>,
    pub credits: Option<f64>,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// 创建课程请求 DTO
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCourseRequest {
    pub name: String,
    pub semester: Option<String>,
    pub credits: Option<f64>,
}

impl CreateCourseRequest {
    /// 验证请求
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("课程名称不能为空".to_string());
        }
        if self.name.len() > 255 {
            return Err("课程名称不能超过255个字符".to_string());
        }
        if let Some(ref semester) = self.semester {
            if semester.len() > 50 {
                return Err("开课学期不能超过50个字符".to_string());
            }
        }
        if let Some(credits) = self.credits {
            if !(0.0..=100.0).contains(&credits) {
                return Err("学分必须在0-100之间".to_string());
            }
        }
        Ok(())
    }
}

/// 更新课程请求 DTO
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCourseRequest {
    pub name: Option<String>,
    pub semester: Option<String>,
    pub credits: Option<f64>,
}

impl UpdateCourseRequest {
    /// 验证请求
    pub fn validate(&self) -> Result<(), String> {
        if let Some(ref name) = self.name {
            if name.trim().is_empty() {
                return Err("课程名称不能为空".to_string());
            }
            if name.len() > 255 {
                return Err("课程名称不能超过255个字符".to_string());
            }
        }
        if let Some(ref semester) = self.semester {
            if semester.len() > 50 {
                return Err("开课学期不能超过50个字符".to_string());
            }
        }
        if let Some(credits) = self.credits {
            if !(0.0..=100.0).contains(&credits) {
                return Err("学分必须在0-100之间".to_string());
            }
        }
        Ok(())
    }
}

/// 更新课程状态请求 DTO
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCourseStatusRequest {
    pub is_active: bool,
}

/// 课程列表查询参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseListQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub semester: Option<String>,
    pub is_active: Option<bool>,
}

impl CourseListQuery {
    pub fn get_page(&self) -> i32 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn get_per_page(&self) -> i32 {
        self.per_page.unwrap_or(20).clamp(1, 100)
    }
}

/// 课程列表响应 DTO
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseListResponse {
    pub courses: Vec<Course>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

/// 批量导入课程请求项
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchImportCourseItem {
    pub name: String,
    pub semester: Option<String>,
    pub credits: Option<f64>,
}

/// 批量导入课程请求 DTO
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchImportCoursesRequest {
    pub courses: Vec<BatchImportCourseItem>,
}

/// 批量导入课程结果
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchImportCoursesResult {
    pub success_count: i32,
    pub fail_count: i32,
    pub failed_items: Vec<FailedCourseImportItem>,
}

/// 导入失败的课程项
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FailedCourseImportItem {
    pub name: String,
    pub reason: String,
}

/// 批量删除课程请求 DTO
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchDeleteCoursesRequest {
    /// 编号列表，格式如 "1,2-10,100-200,344"
    pub sns: String,
}

/// 批量删除课程结果
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchDeleteCoursesResult {
    pub success_count: i32,
    pub fail_count: i32,
    pub not_found_count: i32,
    pub failed_items: Vec<FailedCourseDeleteItem>,
}

/// 删除失败的课程项
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FailedCourseDeleteItem {
    pub sn: i64,
    pub reason: String,
}
