use crate::services::{AdminError, CourseError, ResourceError, TeacherError};
use crate::utils::{bad_request, conflict, forbidden, internal_error, not_found};
use actix_web::HttpResponse;

/// 将AdminError转换为HttpResponse
/// 使用正确的 HTTP 状态码
pub fn handle_admin_error(err: AdminError) -> HttpResponse {
    match err {
        AdminError::NotFound(msg) => not_found(&msg),
        AdminError::ValidationError(msg) => bad_request(&msg),
        AdminError::Forbidden(msg) => forbidden(&msg),
        AdminError::DatabaseError(msg) => {
            log::error!("[Admin] 数据库错误 | error={}", msg);
            internal_error("服务器内部错误")
        }
    }
}

/// 将TeacherError转换为HttpResponse
pub fn handle_teacher_error(err: TeacherError) -> HttpResponse {
    match err {
        TeacherError::NotFound(msg) => not_found(&msg),
        TeacherError::ValidationError(msg) => bad_request(&msg),
        TeacherError::DatabaseError(msg) => {
            log::error!("[Admin] 教师服务数据库错误 | error={}", msg);
            internal_error("服务器内部错误")
        }
    }
}

/// 将CourseError转换为HttpResponse
pub fn handle_course_error(err: CourseError) -> HttpResponse {
    match err {
        CourseError::NotFound(msg) => not_found(&msg),
        CourseError::ValidationError(msg) => bad_request(&msg),
        CourseError::DatabaseError(msg) => {
            log::error!("[Admin] 课程服务数据库错误 | error={}", msg);
            internal_error("服务器内部错误")
        }
    }
}

/// 将ResourceError转换为HttpResponse
pub fn handle_resource_error(err: ResourceError) -> HttpResponse {
    match err {
        ResourceError::NotFound(msg) => not_found(&msg),
        ResourceError::ValidationError(msg) => bad_request(&msg),
        ResourceError::Unauthorized(msg) => forbidden(&msg),
        ResourceError::Conflict(msg) => conflict(&msg),
        _ => {
            log::error!("[Admin] 资源服务错误 | error={}", err);
            internal_error("服务器内部错误")
        }
    }
}
