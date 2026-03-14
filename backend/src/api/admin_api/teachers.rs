use actix_multipart::Multipart;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use calamine::Reader;
use futures_util::StreamExt;

use crate::db::AppState;
use crate::models::{
    BatchDeleteTeachersRequest, BatchImportTeacherItem, BatchImportTeachersRequest,
    CreateTeacherRequest, CurrentUser, TeacherListQuery, UpdateTeacherRequest,
    UpdateTeacherStatusRequest,
};
use crate::services::TeacherService;
use crate::utils::bad_request;

use super::{
    check_admin,
    utils::{handle_admin_error, handle_teacher_error},
};

/// 获取教师列表（管理员）
#[get("/admin/teachers")]
async fn get_teacher_list(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    query: web::Query<TeacherListQuery>,
) -> impl Responder {
    let user = current_user.into_inner();
    log::info!("[Admin] 获取教师列表 | admin_id={}", user.id);

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    match TeacherService::get_teacher_list(&data.pool, query.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => handle_teacher_error(e),
    }
}

/// 添加教师
#[post("/admin/teachers")]
async fn create_teacher(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    req: web::Json<CreateTeacherRequest>,
) -> impl Responder {
    let user = current_user.into_inner();
    log::info!("[Admin] 添加教师 | admin_id={}", user.id);

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    match TeacherService::create_teacher(&data.pool, req.into_inner()).await {
        Ok(teacher) => HttpResponse::Created().json(teacher),
        Err(e) => handle_teacher_error(e),
    }
}

/// 更新教师信息
#[put("/admin/teachers/{sn}")]
async fn update_teacher(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<i64>,
    req: web::Json<UpdateTeacherRequest>,
) -> impl Responder {
    let user = current_user.into_inner();
    let sn = path.into_inner();
    log::info!(
        "[Admin] 更新教师信息 | admin_id={}, teacher_sn={}",
        user.id,
        sn
    );

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    match TeacherService::update_teacher(&data.pool, sn, req.into_inner()).await {
        Ok(teacher) => HttpResponse::Ok().json(teacher),
        Err(e) => handle_teacher_error(e),
    }
}

/// 更新教师状态
#[put("/admin/teachers/{sn}/status")]
async fn update_teacher_status(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<i64>,
    req: web::Json<UpdateTeacherStatusRequest>,
) -> impl Responder {
    let user = current_user.into_inner();
    let sn = path.into_inner();
    log::info!(
        "[Admin] 更新教师状态 | admin_id={}, teacher_sn={}, is_active={}",
        user.id,
        sn,
        req.is_active
    );

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    match TeacherService::update_teacher_status(&data.pool, sn, req.into_inner()).await {
        Ok(teacher) => HttpResponse::Ok().json(teacher),
        Err(e) => handle_teacher_error(e),
    }
}

/// 删除教师
#[delete("/admin/teachers/{sn}")]
async fn delete_teacher(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<i64>,
) -> impl Responder {
    let user = current_user.into_inner();
    let sn = path.into_inner();
    log::info!("[Admin] 删除教师 | admin_id={}, teacher_sn={}", user.id, sn);

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    match TeacherService::delete_teacher(&data.pool, sn).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => handle_teacher_error(e),
    }
}

/// 批量导入教师
#[post("/admin/teachers/batch-import")]
async fn batch_import_teachers(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    req: web::Json<BatchImportTeachersRequest>,
) -> impl Responder {
    let user = current_user.into_inner();
    log::info!(
        "[Admin] 批量导入教师 | admin_id={}, count={}",
        user.id,
        req.teachers.len()
    );

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    if req.teachers.is_empty() {
        return bad_request("导入数据不能为空");
    }

    match TeacherService::batch_import_teachers(&data.pool, req.teachers.clone()).await {
        Ok(result) => {
            log::info!(
                "[Admin] 批量导入教师完成 | admin_id={}, success={}, fail={}",
                user.id,
                result.success_count,
                result.fail_count
            );
            HttpResponse::Ok().json(result)
        }
        Err(e) => handle_teacher_error(e),
    }
}

/// 从文件批量导入教师
#[post("/admin/teachers/batch-import-file")]
async fn batch_import_teachers_from_file(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    mut payload: Multipart,
) -> impl Responder {
    let user = current_user.into_inner();
    log::info!("[Admin] 开始从文件批量导入教师 | admin_id={}", user.id);

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    let mut file_data: Vec<u8> = Vec::new();
    let mut file_type: String = String::new();

    // 读取上传的文件
    while let Some(Ok(mut field)) = payload.next().await {
        let content_disposition = field.content_disposition();
        let name = content_disposition
            .get_name()
            .unwrap_or_default()
            .to_string();

        if name == "file" {
            // 从文件名推断文件类型
            if let Some(filename) = content_disposition.get_filename() {
                file_type = if filename.ends_with(".json") {
                    "json".to_string()
                } else if filename.ends_with(".csv") {
                    "csv".to_string()
                } else if filename.ends_with(".xlsx") {
                    "xlsx".to_string()
                } else {
                    return bad_request("不支持的文件格式，请上传 .json, .csv 或 .xlsx 文件");
                };
            }

            // 读取文件内容
            while let Some(chunk) = field.next().await {
                match chunk {
                    Ok(bytes) => file_data.extend_from_slice(&bytes),
                    Err(e) => {
                        log::error!("[Admin] 读取文件失败 | error={}", e);
                        return bad_request("文件读取失败");
                    }
                }
            }
        }
    }

    if file_data.is_empty() {
        return bad_request("未上传文件或文件为空");
    }

    if file_type.is_empty() {
        return bad_request("无法识别文件类型");
    }

    // 解析文件内容
    let teachers = match parse_teachers_from_bytes(&file_data, &file_type) {
        Ok(teachers) => teachers,
        Err(e) => {
            return bad_request(&e);
        }
    };

    if teachers.is_empty() {
        return bad_request("文件中没有有效的教师数据");
    }

    log::info!(
        "[Admin] 文件解析成功，开始导入 | admin_id={}, count={}",
        user.id,
        teachers.len()
    );

    // 调用批量导入服务
    match TeacherService::batch_import_teachers(&data.pool, teachers).await {
        Ok(result) => {
            log::info!(
                "[Admin] 批量导入教师完成 | admin_id={}, success={}, fail={}",
                user.id,
                result.success_count,
                result.fail_count
            );
            HttpResponse::Ok().json(result)
        }
        Err(e) => handle_teacher_error(e),
    }
}

/// 批量删除教师
#[post("/admin/teachers/batch-delete")]
async fn batch_delete_teachers(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    req: web::Json<BatchDeleteTeachersRequest>,
) -> impl Responder {
    let user = current_user.into_inner();
    log::info!(
        "[Admin] 批量删除教师 | admin_id={}, sns={}",
        user.id,
        req.sns
    );

    if let Err(e) = check_admin(&user) {
        return handle_admin_error(e);
    }

    if req.sns.trim().is_empty() {
        return bad_request("编号列表不能为空");
    }

    match TeacherService::batch_delete_teachers(&data.pool, &req.sns).await {
        Ok(result) => {
            log::info!(
                "[Admin] 批量删除教师完成 | admin_id={}, success={}, not_found={}, fail={}",
                user.id,
                result.success_count,
                result.not_found_count,
                result.fail_count
            );
            HttpResponse::Ok().json(result)
        }
        Err(e) => handle_teacher_error(e),
    }
}

/// 解析文件内容为教师数据
fn parse_teachers_from_bytes(
    data: &[u8],
    file_type: &str,
) -> Result<Vec<BatchImportTeacherItem>, String> {
    match file_type {
        "json" => {
            let teachers: Vec<BatchImportTeacherItem> =
                serde_json::from_slice(data).map_err(|e| format!("JSON解析错误: {}", e))?;
            Ok(teachers)
        }
        "csv" => {
            let mut rdr = csv::Reader::from_reader(data);
            let mut teachers = Vec::new();
            for (idx, result) in rdr.records().enumerate() {
                let record = result.map_err(|e| format!("CSV第{}行解析错误: {}", idx + 1, e))?;
                let name = record
                    .get(0)
                    .ok_or_else(|| format!("CSV第{}行: 缺少姓名", idx + 1))?
                    .trim()
                    .to_string();
                let department = record.get(1).map(|s| s.trim().to_string());
                let department = if department.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                    None
                } else {
                    department
                };
                teachers.push(BatchImportTeacherItem { name, department });
            }
            Ok(teachers)
        }
        "xlsx" => {
            let mut teachers = Vec::new();
            let cursor = std::io::Cursor::new(data);
            let mut workbook: calamine::Xlsx<std::io::Cursor<&[u8]>> =
                calamine::Xlsx::new(cursor).map_err(|e| format!("Excel文件解析错误: {:?}", e))?;
            let range = workbook
                .worksheet_range_at(0)
                .ok_or("无法读取Excel第一个工作表")?
                .map_err(|e| format!("Excel读取错误: {:?}", e))?;

            for (idx, row) in range.rows().enumerate().skip(1) {
                // 跳过标题行
                let name_cell = row
                    .get(0)
                    .ok_or_else(|| format!("Excel第{}行: 缺少姓名", idx + 1))?;
                let name = name_cell.to_string().trim().to_string();
                let department: Option<String> =
                    row.get(1).map(|c| c.to_string().trim().to_string());
                let department = if department.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                    None
                } else {
                    department
                };
                teachers.push(BatchImportTeacherItem { name, department });
            }
            Ok(teachers)
        }
        _ => Err("不支持的文件格式".to_string()),
    }
}

/// 配置路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_teacher_list)
        .service(create_teacher)
        .service(update_teacher)
        .service(update_teacher_status)
        .service(delete_teacher)
        .service(batch_import_teachers)
        .service(batch_import_teachers_from_file)
        .service(batch_delete_teachers);
}
