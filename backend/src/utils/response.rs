// 统一响应处理工具

use actix_web::HttpResponse;

/// 对文件名进行 RFC 5987 编码，用于支持中文等非 ASCII 字符
/// 参考: https://datatracker.ietf.org/doc/html/rfc5987
pub fn encode_rfc5987(filename: &str) -> String {
    let mut result = String::new();
    for c in filename.chars() {
        if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
            // ASCII 字母数字和常用符号直接保留
            result.push(c);
        } else {
            // 非 ASCII 字符进行 percent-encoding
            for byte in c.encode_utf8(&mut [0; 4]).bytes() {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

/// 检查文件名是否只包含 ASCII 字符
pub fn is_ascii_filename(filename: &str) -> bool {
    filename.is_ascii()
}

/// 构建 Content-Disposition 头部值，支持中文文件名
///
/// 策略：
/// 1. 对于纯 ASCII 文件名：直接使用 filename="xxx"
/// 2. 对于含中文的文件名：同时提供 filename 和 filename*
///    - filename*：RFC 5987 编码，现代浏览器优先使用
pub fn build_content_disposition(filename: &str) -> String {
    if is_ascii_filename(filename) {
        // 纯 ASCII 文件名，直接使用
        format!("attachment; filename=\"{}\"", filename)
    } else {
        // 包含中文等非 ASCII 字符
        // RFC 5987 编码用于 filename*
        let encoded = encode_rfc5987(filename);

        // 同时提供 filename* 和 filename
        // filename* 优先被现代浏览器使用，能正确显示中文
        format!(
            "attachment; filename*=UTF-8''{}; filename=\"{}\"",
            encoded, filename
        )
    }
}

/// 构建带显式错误码的错误响应
///
/// `error` 为机器可读错误码（PascalCase），用于前端按码分支（如 `InvalidCredentials`、`TokenExpired`）。
/// 详见 dev_docs/specs/api_design.md 第 3.2 节。
pub fn error_response_with_code(status: u16, error: &str, message: &str) -> HttpResponse {
    HttpResponse::build(
        actix_web::http::StatusCode::from_u16(status)
            .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR),
    )
    .json(serde_json::json!({
        "error": error,
        "message": message
    }))
}

/// 构建错误响应（错误码按状态码通用映射）
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

    error_response_with_code(status, error, message)
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

/// 快速构建 500 Internal Server Error 错误
pub fn internal_error(message: &str) -> HttpResponse {
    error_response(500, message)
}

/// 构建创建成功响应（201 Created）
pub fn created<T: serde::Serialize>(data: T) -> HttpResponse {
    HttpResponse::Created().json(data)
}

/// 构建无内容响应（204 No Content）
pub fn no_content() -> HttpResponse {
    HttpResponse::NoContent().finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    mod error_response_tests {
        use super::*;

        #[test]
        fn test_error_response_400() {
            let response = error_response(400, "Bad request message");
            assert_eq!(response.status(), 400);
        }

        #[test]
        fn test_error_response_401() {
            let response = error_response(401, "Unauthorized");
            assert_eq!(response.status(), 401);
        }

        #[test]
        fn test_error_response_403() {
            let response = error_response(403, "Forbidden");
            assert_eq!(response.status(), 403);
        }

        #[test]
        fn test_error_response_404() {
            let response = error_response(404, "Not found");
            assert_eq!(response.status(), 404);
        }

        #[test]
        fn test_error_response_409() {
            let response = error_response(409, "Conflict");
            assert_eq!(response.status(), 409);
        }

        #[test]
        fn test_error_response_500() {
            let response = error_response(500, "Internal server error");
            assert_eq!(response.status(), 500);
        }

        #[test]
        fn test_error_response_unknown_status() {
            let response = error_response(418, "I'm a teapot");
            assert_eq!(response.status(), 418);
        }
    }

    mod helper_functions_tests {
        use super::*;

        #[test]
        fn test_bad_request() {
            let response = bad_request("Invalid input");
            assert_eq!(response.status(), 400);
        }

        #[test]
        fn test_unauthorized() {
            let response = unauthorized("Please login");
            assert_eq!(response.status(), 401);
        }

        #[test]
        fn test_forbidden() {
            let response = forbidden("Access denied");
            assert_eq!(response.status(), 403);
        }

        #[test]
        fn test_not_found() {
            let response = not_found("Resource not found");
            assert_eq!(response.status(), 404);
        }

        #[test]
        fn test_conflict() {
            let response = conflict("Resource already exists");
            assert_eq!(response.status(), 409);
        }

        #[test]
        fn test_internal_error() {
            let response = internal_error("Something went wrong");
            assert_eq!(response.status(), 500);
        }

        #[test]
        fn test_created() {
            #[derive(serde::Serialize)]
            struct TestData {
                id: i32,
                name: String,
            }

            let data = TestData {
                id: 1,
                name: "Test".to_string(),
            };
            let response = created(data);
            assert_eq!(response.status(), 201);
        }

        #[test]
        fn test_no_content() {
            let response = no_content();
            assert_eq!(response.status(), 204);
        }
    }
}
