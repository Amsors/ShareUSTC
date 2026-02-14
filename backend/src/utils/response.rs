// 统一响应处理工具

use actix_web::HttpResponse;

/// 构建错误响应
pub fn error_response(status: u16, message: &str) -> HttpResponse {
    let error = match status {
        400 => "BadRequest",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "NotFound",
        409 => "Conflict",
        422 => "UnprocessableEntity",
        500 => "InternalServerError",
        502 => "BadGateway",
        503 => "ServiceUnavailable",
        _ => "UnknownError",
    };

    HttpResponse::build(
        actix_web::http::StatusCode::from_u16(status).unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
    )
    .json(serde_json::json!({
        "error": error,
        "message": message
    }))
}

/// 快速构建 400 Bad Request 错误
pub fn bad_request(message: &str) -> HttpResponse {
    error_response(400, message)
}

/// 快速构建 401 Unauthorized 错误
pub fn unauthorized(message: &str) -> HttpResponse {
    error_response(401, message)
}

/// 快速构建 403 Forbidden 错误
pub fn forbidden(message: &str) -> HttpResponse {
    error_response(403, message)
}

/// 快速构建 404 Not Found 错误
pub fn not_found(message: &str) -> HttpResponse {
    error_response(404, message)
}

/// 快速构建 409 Conflict 错误
pub fn conflict(message: &str) -> HttpResponse {
    error_response(409, message)
}

/// 快速构建 422 Unprocessable Entity 错误
pub fn unprocessable_entity(message: &str) -> HttpResponse {
    error_response(422, message)
}

/// 快速构建 500 Internal Server Error 错误
pub fn internal_error(message: &str) -> HttpResponse {
    error_response(500, message)
}

/// 构建成功响应（200 OK）
pub fn ok<T: serde::Serialize>(data: T) -> HttpResponse {
    HttpResponse::Ok().json(data)
}

/// 构建创建成功响应（201 Created）
pub fn created<T: serde::Serialize>(data: T) -> HttpResponse {
    HttpResponse::Created().json(data)
}

/// 构建无内容响应（204 No Content）
pub fn no_content() -> HttpResponse {
    HttpResponse::NoContent().finish()
}
