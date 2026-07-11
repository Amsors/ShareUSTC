use sqlx::PgPool;

use crate::models::{
    BatchDeleteCoursesResult, BatchImportCourseItem, BatchImportCoursesResult, Course,
    CourseListQuery, CourseListResponse, CreateCourseRequest, FailedCourseDeleteItem,
    FailedCourseImportItem, UpdateCourseRequest, UpdateCourseStatusRequest,
};

/// 课程服务错误类型
#[derive(Debug, thiserror::Error)]
pub enum CourseError {
    #[error("未找到: {0}")]
    NotFound(String),
    #[error("验证错误: {0}")]
    ValidationError(String),
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),
}

impl actix_web::ResponseError for CourseError {
    fn error_response(&self) -> actix_web::HttpResponse {
        use crate::utils::{bad_request, internal_error, not_found};
        match self {
            CourseError::NotFound(msg) => not_found(msg),
            CourseError::ValidationError(msg) => bad_request(msg),
            CourseError::Database(e) => {
                log::error!("[Course] 数据库错误 | error={}", e);
                internal_error("服务器内部错误")
            }
        }
    }
}

/// 课程服务
pub struct CourseService;

impl CourseService {
    /// 创建课程
    pub async fn create_course(
        pool: &PgPool,
        req: CreateCourseRequest,
    ) -> Result<Course, CourseError> {
        // 验证请求
        if let Err(e) = req.validate() {
            return Err(CourseError::ValidationError(e));
        }

        let course = sqlx::query_as::<_, Course>(
            r#"
            INSERT INTO courses (name, semester, credits, is_active)
            VALUES ($1, $2, $3, true)
            RETURNING id, sn, name, semester, credits, is_active, created_at, updated_at
            "#,
        )
        .bind(&req.name)
        .bind(&req.semester)
        .bind(req.credits)
        .fetch_one(pool)
        .await?;

        Ok(course)
    }

    /// 获取课程列表（管理员）
    pub async fn get_course_list(
        pool: &PgPool,
        query: CourseListQuery,
    ) -> Result<CourseListResponse, CourseError> {
        let page = query.get_page();
        let per_page = query.get_per_page();
        let offset = (page - 1) * per_page;

        // 构建查询条件
        let mut conditions = vec!["1=1".to_string()];
        if let Some(semester) = &query.semester {
            conditions.push(format!("semester = '{}'", semester));
        }
        if let Some(is_active) = query.is_active {
            conditions.push(format!("is_active = {}", is_active));
        }
        let where_clause = conditions.join(" AND ");

        // 查询总数
        let count_sql = format!("SELECT COUNT(*) FROM courses WHERE {}", where_clause);
        let total: i64 = sqlx::query_scalar(&count_sql).fetch_one(pool).await?;

        // 查询列表
        let list_sql = format!(
            r#"
            SELECT id, sn, name, semester, credits, is_active, created_at, updated_at
            FROM courses
            WHERE {}
            ORDER BY sn ASC
            LIMIT {} OFFSET {}
            "#,
            where_clause, per_page, offset
        );

        let courses = sqlx::query_as::<_, Course>(&list_sql)
            .fetch_all(pool)
            .await?;

        Ok(CourseListResponse {
            courses,
            total,
            page,
            per_page,
        })
    }

    /// 根据编号获取课程
    pub async fn get_course_by_sn(pool: &PgPool, sn: i64) -> Result<Course, CourseError> {
        let course = sqlx::query_as::<_, Course>(
            r#"
            SELECT id, sn, name, semester, credits, is_active, created_at, updated_at
            FROM courses
            WHERE sn = $1
            "#,
        )
        .bind(sn)
        .fetch_optional(pool)
        .await?;

        course.ok_or_else(|| CourseError::NotFound(format!("课程编号 {} 不存在", sn)))
    }

    /// 获取有效课程列表（公开）
    /// 当 with_resources_only 为 true 时，只返回有关联资源的课程
    pub async fn get_active_courses(
        pool: &PgPool,
        with_resources_only: bool,
    ) -> Result<Vec<Course>, CourseError> {
        let sql = if with_resources_only {
            r#"
            SELECT DISTINCT c.id, c.sn, c.name, c.semester, c.credits, c.is_active, c.created_at, c.updated_at
            FROM courses c
            INNER JOIN resource_courses rc ON c.sn = rc.course_sn
            WHERE c.is_active = true
            ORDER BY c.sn ASC
            "#
        } else {
            r#"
            SELECT id, sn, name, semester, credits, is_active, created_at, updated_at
            FROM courses
            WHERE is_active = true
            ORDER BY sn ASC
            "#
        };

        let courses = sqlx::query_as::<_, Course>(sql).fetch_all(pool).await?;

        Ok(courses)
    }

    /// 更新课程信息
    pub async fn update_course(
        pool: &PgPool,
        sn: i64,
        req: UpdateCourseRequest,
    ) -> Result<Course, CourseError> {
        // 验证请求
        if let Err(e) = req.validate() {
            return Err(CourseError::ValidationError(e));
        }

        // 检查课程是否存在
        let existing = Self::get_course_by_sn(pool, sn).await?;

        // 构建更新语句
        let mut updates = vec![];
        if let Some(name) = req.name {
            updates.push(format!("name = '{}'", name));
        }
        if let Some(semester) = req.semester {
            updates.push(format!("semester = '{}'", semester));
        }
        if let Some(credits) = req.credits {
            updates.push(format!("credits = {}", credits));
        }

        if updates.is_empty() {
            return Ok(existing);
        }

        let update_sql = format!(
            "UPDATE courses SET {}, updated_at = CURRENT_TIMESTAMP WHERE sn = $1 RETURNING id, sn, name, semester, credits, is_active, created_at, updated_at",
            updates.join(", ")
        );

        let course = sqlx::query_as::<_, Course>(&update_sql)
            .bind(sn)
            .fetch_one(pool)
            .await?;

        Ok(course)
    }

    /// 更新课程状态
    pub async fn update_course_status(
        pool: &PgPool,
        sn: i64,
        req: UpdateCourseStatusRequest,
    ) -> Result<Course, CourseError> {
        let course = sqlx::query_as::<_, Course>(
            r#"
            UPDATE courses
            SET is_active = $1, updated_at = CURRENT_TIMESTAMP
            WHERE sn = $2
            RETURNING id, sn, name, semester, credits, is_active, created_at, updated_at
            "#,
        )
        .bind(req.is_active)
        .bind(sn)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("no rows") {
                CourseError::NotFound(format!("课程编号 {} 不存在", sn))
            } else {
                CourseError::Database(e)
            }
        })?;

        Ok(course)
    }

    /// 删除课程
    pub async fn delete_course(pool: &PgPool, sn: i64) -> Result<(), CourseError> {
        let result = sqlx::query("DELETE FROM courses WHERE sn = $1")
            .bind(sn)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(CourseError::NotFound(format!("课程编号 {} 不存在", sn)));
        }

        Ok(())
    }

    /// 批量导入课程
    pub async fn batch_import_courses(
        pool: &PgPool,
        items: Vec<BatchImportCourseItem>,
    ) -> Result<BatchImportCoursesResult, CourseError> {
        let mut success_count = 0i32;
        let mut fail_count = 0i32;
        let mut failed_items: Vec<FailedCourseImportItem> = Vec::new();

        for item in items {
            // 验证数据
            if item.name.trim().is_empty() {
                fail_count += 1;
                failed_items.push(FailedCourseImportItem {
                    name: item.name.clone(),
                    reason: "课程名称不能为空".to_string(),
                });
                continue;
            }

            if item.name.len() > 255 {
                fail_count += 1;
                failed_items.push(FailedCourseImportItem {
                    name: item.name.clone(),
                    reason: "课程名称不能超过255个字符".to_string(),
                });
                continue;
            }

            if let Some(ref semester) = item.semester {
                if semester.len() > 50 {
                    fail_count += 1;
                    failed_items.push(FailedCourseImportItem {
                        name: item.name.clone(),
                        reason: "开课学期不能超过50个字符".to_string(),
                    });
                    continue;
                }
            }

            if let Some(credits) = item.credits {
                if !(0.0..=100.0).contains(&credits) {
                    fail_count += 1;
                    failed_items.push(FailedCourseImportItem {
                        name: item.name.clone(),
                        reason: "学分必须在0-100之间".to_string(),
                    });
                    continue;
                }
            }

            // 检查是否已存在同名课程
            let existing: Option<(i64,)> =
                sqlx::query_as("SELECT sn FROM courses WHERE name = $1 LIMIT 1")
                    .bind(&item.name)
                    .fetch_optional(pool)
                    .await?;

            if existing.is_some() {
                fail_count += 1;
                failed_items.push(FailedCourseImportItem {
                    name: item.name.clone(),
                    reason: "课程名称已存在".to_string(),
                });
                continue;
            }

            // 插入课程
            let result = sqlx::query(
                r#"
                INSERT INTO courses (name, semester, credits, is_active)
                VALUES ($1, $2, $3, true)
                "#,
            )
            .bind(&item.name)
            .bind(&item.semester)
            .bind(item.credits)
            .execute(pool)
            .await;

            match result {
                Ok(_) => {
                    success_count += 1;
                }
                Err(e) => {
                    fail_count += 1;
                    failed_items.push(FailedCourseImportItem {
                        name: item.name.clone(),
                        reason: format!("数据库错误: {}", e),
                    });
                }
            }
        }

        Ok(BatchImportCoursesResult {
            success_count,
            fail_count,
            failed_items,
        })
    }

    /// 解析编号字符串，如 "1,2-10,100-200,344" 解析为 [1, 2, 3, ..., 10, 100, 101, ..., 200, 344]
    fn parse_sn_ranges(sns_str: &str) -> Result<Vec<i64>, String> {
        let mut sns = Vec::new();

        for part in sns_str.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            if part.contains('-') {
                // 范围格式: start-end
                let range_parts: Vec<&str> = part.split('-').collect();
                if range_parts.len() != 2 {
                    return Err(format!("无效的编号范围: {}", part));
                }

                let start: i64 = range_parts[0]
                    .trim()
                    .parse()
                    .map_err(|_| format!("无效的编号: {}", range_parts[0]))?;
                let end: i64 = range_parts[1]
                    .trim()
                    .parse()
                    .map_err(|_| format!("无效的编号: {}", range_parts[1]))?;

                if start > end {
                    return Err(format!("范围起始值不能大于结束值: {}", part));
                }

                if start <= 0 || end <= 0 {
                    return Err("编号必须为正整数".to_string());
                }

                // 限制单次范围大小为10000，防止内存溢出
                if end - start > 10000 {
                    return Err(format!("范围 {}-{} 过大，单次最多支持10000个", start, end));
                }

                for sn in start..=end {
                    sns.push(sn);
                }
            } else {
                // 单个编号
                let sn: i64 = part.parse().map_err(|_| format!("无效的编号: {}", part))?;

                if sn <= 0 {
                    return Err("编号必须为正整数".to_string());
                }

                sns.push(sn);
            }
        }

        // 去重并排序
        sns.sort_unstable();
        sns.dedup();

        // 限制总数为50000，防止内存溢出
        if sns.len() > 50000 {
            return Err("单次删除数量不能超过50000条".to_string());
        }

        Ok(sns)
    }

    /// 批量删除课程
    pub async fn batch_delete_courses(
        pool: &PgPool,
        sns_str: &str,
    ) -> Result<BatchDeleteCoursesResult, CourseError> {
        // 解析编号列表
        let sns = match Self::parse_sn_ranges(sns_str) {
            Ok(sns) => sns,
            Err(e) => return Err(CourseError::ValidationError(e)),
        };

        if sns.is_empty() {
            return Err(CourseError::ValidationError("编号列表不能为空".to_string()));
        }

        let mut success_count = 0i32;
        let mut not_found_count = 0i32;
        let mut failed_items: Vec<FailedCourseDeleteItem> = Vec::new();

        // 逐个删除（由于需要准确反馈每条记录的结果，使用循环删除而非批量删除）
        for sn in sns {
            match sqlx::query("DELETE FROM courses WHERE sn = $1")
                .bind(sn)
                .execute(pool)
                .await
            {
                Ok(result) => {
                    if result.rows_affected() > 0 {
                        success_count += 1;
                    } else {
                        not_found_count += 1;
                        failed_items.push(FailedCourseDeleteItem {
                            sn,
                            reason: "课程编号不存在".to_string(),
                        });
                    }
                }
                Err(e) => {
                    failed_items.push(FailedCourseDeleteItem {
                        sn,
                        reason: format!("数据库错误: {}", e),
                    });
                }
            }
        }

        Ok(BatchDeleteCoursesResult {
            success_count,
            fail_count: failed_items.len() as i32,
            not_found_count,
            failed_items,
        })
    }
}
