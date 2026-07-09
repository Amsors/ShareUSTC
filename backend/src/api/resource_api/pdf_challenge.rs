use actix_web::{get, post, web, HttpResponse, Responder};

use crate::db::AppState;

/// 获取 PDF 预览检测配置
#[get("/resources/pdf-preview-challenge/config")]
pub async fn get_pdf_preview_challenge_config(state: web::Data<AppState>) -> impl Responder {
    match &state.pdf_preview_challenge_uuid {
        Some(uuid) => HttpResponse::Ok().json(serde_json::json!({
            "resourceId": uuid,
            "enabled": true
        })),
        None => HttpResponse::Ok().json(serde_json::json!({
            "resourceId": null,
            "enabled": false
        })),
    }
}

/// 验证 PDF 预览检测答案
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct VerifyPdfPreviewChallengeRequest {
    code: String,
}

#[post("/resources/pdf-preview-challenge/verify")]
pub async fn verify_pdf_preview_challenge(
    state: web::Data<AppState>,
    request: web::Json<VerifyPdfPreviewChallengeRequest>,
) -> impl Responder {
    // 检查是否配置了挑战（uuid 与 code 均须存在），未配置直接返回 503
    // 使用 let-else 解构避免 unwrap 造成的 panic 风险
    let (Some(_uuid), Some(expected_code)) = (
        &state.pdf_preview_challenge_uuid,
        &state.pdf_preview_challenge_code,
    ) else {
        return HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "ServiceUnavailable",
            "message": "PDF预览检测功能未配置"
        }));
    };

    // 验证答案
    if request.code.trim() == expected_code.trim() {
        HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "验证成功"
        }))
    } else {
        HttpResponse::Ok().json(serde_json::json!({
            "success": false,
            "message": "验证码错误"
        }))
    }
}
