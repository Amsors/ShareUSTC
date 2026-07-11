use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 授课教师结构体（对应数据库 teachers 表）
#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Teacher {
    pub id: Uuid,
    pub sn: i64,
    pub name: String,
    pub department: Option<String>,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// 创建教师请求 DTO
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTeacherRequest {
    pub name: String,
    pub department: Option<String>,
}

impl CreateTeacherRequest {
    /// 验证请求
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("教师姓名不能为空".to_string());
        }
        if self.name.len() > 100 {
            return Err("教师姓名不能超过100个字符".to_string());
        }
        if let Some(ref dept) = self.department {
            if dept.len() > 100 {
                return Err("学院名称不能超过100个字符".to_string());
            }
        }
        Ok(())
    }
}

/// 更新教师请求 DTO
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTeacherRequest {
    pub name: Option<String>,
    pub department: Option<String>,
}

impl UpdateTeacherRequest {
    /// 验证请求
    pub fn validate(&self) -> Result<(), String> {
        if let Some(ref name) = self.name {
            if name.trim().is_empty() {
                return Err("教师姓名不能为空".to_string());
            }
            if name.len() > 100 {
                return Err("教师姓名不能超过100个字符".to_string());
            }
        }
        if let Some(ref dept) = self.department {
            if dept.len() > 100 {
                return Err("学院名称不能超过100个字符".to_string());
            }
        }
        Ok(())
    }
}

/// 更新教师状态请求 DTO
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTeacherStatusRequest {
    pub is_active: bool,
}

/// 教师列表查询参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeacherListQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub department: Option<String>,
    pub is_active: Option<bool>,
}

impl TeacherListQuery {
    pub fn get_page(&self) -> i32 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn get_per_page(&self) -> i32 {
        self.per_page.unwrap_or(20).clamp(1, 100)
    }
}

/// 教师列表响应 DTO
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeacherListResponse {
    pub teachers: Vec<Teacher>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

/// 批量导入教师请求项
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BatchImportTeacherItem {
    pub name: String,
    pub department: Option<String>,
}

/// 批量导入教师请求 DTO
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchImportTeachersRequest {
    pub teachers: Vec<BatchImportTeacherItem>,
}

/// 批量导入教师结果
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchImportTeachersResult {
    pub success_count: i32,
    pub fail_count: i32,
    pub failed_items: Vec<FailedTeacherImportItem>,
}

/// 导入失败的教师项
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FailedTeacherImportItem {
    pub name: String,
    pub reason: String,
}

/// 批量删除教师请求 DTO
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchDeleteTeachersRequest {
    /// 编号列表，格式如 "1,2-10,100-200,344"
    pub sns: String,
}

/// 批量删除教师结果
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchDeleteTeachersResult {
    pub success_count: i32,
    pub fail_count: i32,
    pub not_found_count: i32,
    pub failed_items: Vec<FailedTeacherDeleteItem>,
}

/// 删除失败的教师项
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FailedTeacherDeleteItem {
    pub sn: i64,
    pub reason: String,
}
