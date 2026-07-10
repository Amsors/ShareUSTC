use actix_multipart::Multipart;
use actix_web::{delete, get, post, put, web, HttpResponse};
use calamine::Reader;
use futures_util::StreamExt;

use crate::db::AppState;
use crate::models::{
    BatchDeleteCoursesRequest, BatchImportCourseItem, BatchImportCoursesRequest, CourseListQuery,
    CreateCourseRequest, CurrentUser, UpdateCourseRequest, UpdateCourseStatusRequest,
};
use crate::services::CourseService;
use crate::utils::bad_request;

use super::check_admin;

/// 获取课程列表（管理员）
#[get("/admin/courses")]
async fn get_course_list(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    query: web::Query<CourseListQuery>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let user = current_user.into_inner();
    log::info!("[Admin] 获取课程列表 | admin_id={}", user.id);

    check_admin(&user)?;

    let response = CourseService::get_course_list(&data.pool, query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(response))
}

/// 添加课程
#[post("/admin/courses")]
async fn create_course(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    req: web::Json<CreateCourseRequest>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let user = current_user.into_inner();
    log::info!("[Admin] 添加课程 | admin_id={}", user.id);

    check_admin(&user)?;

    let course = CourseService::create_course(&data.pool, req.into_inner()).await?;
    Ok(HttpResponse::Created().json(course))
}

/// 更新课程信息
#[put("/admin/courses/{sn}")]
async fn update_course(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<i64>,
    req: web::Json<UpdateCourseRequest>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let user = current_user.into_inner();
    let sn = path.into_inner();
    log::info!(
        "[Admin] 更新课程信息 | admin_id={}, course_sn={}",
        user.id,
        sn
    );

    check_admin(&user)?;

    let course = CourseService::update_course(&data.pool, sn, req.into_inner()).await?;
    Ok(HttpResponse::Ok().json(course))
}

/// 更新课程状态
#[put("/admin/courses/{sn}/status")]
async fn update_course_status(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<i64>,
    req: web::Json<UpdateCourseStatusRequest>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let user = current_user.into_inner();
    let sn = path.into_inner();
    log::info!(
        "[Admin] 更新课程状态 | admin_id={}, course_sn={}, is_active={}",
        user.id,
        sn,
        req.is_active
    );

    check_admin(&user)?;

    let course = CourseService::update_course_status(&data.pool, sn, req.into_inner()).await?;
    Ok(HttpResponse::Ok().json(course))
}

/// 删除课程
#[delete("/admin/courses/{sn}")]
async fn delete_course(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    path: web::Path<i64>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let user = current_user.into_inner();
    let sn = path.into_inner();
    log::info!("[Admin] 删除课程 | admin_id={}, course_sn={}", user.id, sn);

    check_admin(&user)?;

    CourseService::delete_course(&data.pool, sn).await?;
    Ok(HttpResponse::NoContent().finish())
}

/// 批量导入课程
#[post("/admin/courses/batch-import")]
async fn batch_import_courses(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    req: web::Json<BatchImportCoursesRequest>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let user = current_user.into_inner();
    log::info!(
        "[Admin] 批量导入课程 | admin_id={}, count={}",
        user.id,
        req.courses.len()
    );

    check_admin(&user)?;

    if req.courses.is_empty() {
        return Ok(bad_request("导入数据不能为空"));
    }

    let result = CourseService::batch_import_courses(&data.pool, req.courses.clone()).await?;
    log::info!(
        "[Admin] 批量导入课程完成 | admin_id={}, success={}, fail={}",
        user.id,
        result.success_count,
        result.fail_count
    );
    Ok(HttpResponse::Ok().json(result))
}

/// 从文件批量导入课程
#[post("/admin/courses/batch-import-file")]
async fn batch_import_courses_from_file(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    mut payload: Multipart,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let user = current_user.into_inner();
    log::info!("[Admin] 开始从文件批量导入课程 | admin_id={}", user.id);

    check_admin(&user)?;

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
                    return Ok(bad_request(
                        "不支持的文件格式，请上传 .json, .csv 或 .xlsx 文件",
                    ));
                };
            }

            // 读取文件内容
            while let Some(chunk) = field.next().await {
                match chunk {
                    Ok(bytes) => file_data.extend_from_slice(&bytes),
                    Err(e) => {
                        log::error!("[Admin] 读取文件失败 | error={}", e);
                        return Ok(bad_request("文件读取失败"));
                    }
                }
            }
        }
    }

    if file_data.is_empty() {
        return Ok(bad_request("未上传文件或文件为空"));
    }

    if file_type.is_empty() {
        return Ok(bad_request("无法识别文件类型"));
    }

    // 解析文件内容
    let courses = match parse_courses_from_bytes(&file_data, &file_type) {
        Ok(courses) => courses,
        Err(e) => {
            return Ok(bad_request(&e));
        }
    };

    if courses.is_empty() {
        return Ok(bad_request("文件中没有有效的课程数据"));
    }

    log::info!(
        "[Admin] 文件解析成功，开始导入 | admin_id={}, count={}",
        user.id,
        courses.len()
    );

    // 调用批量导入服务
    let result = CourseService::batch_import_courses(&data.pool, courses).await?;
    log::info!(
        "[Admin] 批量导入课程完成 | admin_id={}, success={}, fail={}",
        user.id,
        result.success_count,
        result.fail_count
    );
    Ok(HttpResponse::Ok().json(result))
}

/// 批量删除课程
#[post("/admin/courses/batch-delete")]
async fn batch_delete_courses(
    data: web::Data<AppState>,
    current_user: actix_web::web::ReqData<CurrentUser>,
    req: web::Json<BatchDeleteCoursesRequest>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let user = current_user.into_inner();
    log::info!(
        "[Admin] 批量删除课程 | admin_id={}, sns={}",
        user.id,
        req.sns
    );

    check_admin(&user)?;

    if req.sns.trim().is_empty() {
        return Ok(bad_request("编号列表不能为空"));
    }

    let result = CourseService::batch_delete_courses(&data.pool, &req.sns).await?;
    log::info!(
        "[Admin] 批量删除课程完成 | admin_id={}, success={}, not_found={}, fail={}",
        user.id,
        result.success_count,
        result.not_found_count,
        result.fail_count
    );
    Ok(HttpResponse::Ok().json(result))
}

/// 解析文件内容为课程数据
fn parse_courses_from_bytes(
    data: &[u8],
    file_type: &str,
) -> Result<Vec<BatchImportCourseItem>, String> {
    match file_type {
        "json" => {
            let courses: Vec<BatchImportCourseItem> =
                serde_json::from_slice(data).map_err(|e| format!("JSON解析错误: {}", e))?;
            Ok(courses)
        }
        "csv" => {
            let mut rdr = csv::Reader::from_reader(data);
            let mut courses = Vec::new();
            for (idx, result) in rdr.records().enumerate() {
                let record = result.map_err(|e| format!("CSV第{}行解析错误: {}", idx + 1, e))?;
                let name = record
                    .get(0)
                    .ok_or_else(|| format!("CSV第{}行: 缺少课程名称", idx + 1))?
                    .trim()
                    .to_string();
                let semester = record.get(1).map(|s| s.trim().to_string());
                let semester = if semester.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                    None
                } else {
                    semester
                };
                let credits = record
                    .get(2)
                    .and_then(|s| s.trim().parse::<f64>().ok())
                    .filter(|&c| c > 0.0);
                courses.push(BatchImportCourseItem {
                    name,
                    semester,
                    credits,
                });
            }
            Ok(courses)
        }
        "xlsx" => {
            let mut courses = Vec::new();
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
                    .ok_or_else(|| format!("Excel第{}行: 缺少课程名称", idx + 1))?;
                let name = name_cell.to_string().trim().to_string();
                let semester: Option<String> = row.get(1).map(|c| c.to_string().trim().to_string());
                let semester = if semester.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                    None
                } else {
                    semester
                };
                let credits = row
                    .get(2)
                    .and_then(|c| c.to_string().trim().parse::<f64>().ok())
                    .filter(|&c| c > 0.0);
                courses.push(BatchImportCourseItem {
                    name,
                    semester,
                    credits,
                });
            }
            Ok(courses)
        }
        _ => Err("不支持的文件格式".to_string()),
    }
}

/// 配置路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_course_list)
        .service(create_course)
        .service(update_course)
        .service(update_course_status)
        .service(delete_course)
        .service(batch_import_courses)
        .service(batch_import_courses_from_file)
        .service(batch_delete_courses);
}
