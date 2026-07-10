use sqlx::PgPool;

use crate::models::{
    BatchDeleteTeachersResult, BatchImportTeacherItem, BatchImportTeachersResult,
    CreateTeacherRequest, FailedTeacherDeleteItem, FailedTeacherImportItem, Teacher,
    TeacherListQuery, TeacherListResponse, UpdateTeacherRequest, UpdateTeacherStatusRequest,
};

/// 教师服务错误类型
#[derive(Debug, thiserror::Error)]
pub enum TeacherError {
    #[error("未找到: {0}")]
    NotFound(String),
    #[error("验证错误: {0}")]
    ValidationError(String),
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),
}

impl actix_web::ResponseError for TeacherError {
    fn error_response(&self) -> actix_web::HttpResponse {
        use crate::utils::{bad_request, internal_error, not_found};
        match self {
            TeacherError::NotFound(msg) => not_found(msg),
            TeacherError::ValidationError(msg) => bad_request(msg),
            TeacherError::Database(e) => {
                log::error!("[Teacher] 数据库错误 | error={}", e);
                internal_error("服务器内部错误")
            }
        }
    }
}

/// 教师服务
pub struct TeacherService;

impl TeacherService {
    /// 创建教师
    pub async fn create_teacher(
        pool: &PgPool,
        req: CreateTeacherRequest,
    ) -> Result<Teacher, TeacherError> {
        // 验证请求
        if let Err(e) = req.validate() {
            return Err(TeacherError::ValidationError(e));
        }

        let teacher = sqlx::query_as::<_, Teacher>(
            r#"
            INSERT INTO teachers (name, department, is_active)
            VALUES ($1, $2, true)
            RETURNING id, sn, name, department, is_active, created_at, updated_at
            "#,
        )
        .bind(&req.name)
        .bind(&req.department)
        .fetch_one(pool)
        .await?;

        Ok(teacher)
    }

    /// 获取教师列表（管理员）
    pub async fn get_teacher_list(
        pool: &PgPool,
        query: TeacherListQuery,
    ) -> Result<TeacherListResponse, TeacherError> {
        let page = query.get_page();
        let per_page = query.get_per_page();
        let offset = (page - 1) * per_page;

        // 构建查询条件
        let mut conditions = vec!["1=1".to_string()];
        if let Some(dept) = &query.department {
            conditions.push(format!("department = '{}'", dept));
        }
        if let Some(is_active) = query.is_active {
            conditions.push(format!("is_active = {}", is_active));
        }
        let where_clause = conditions.join(" AND ");

        // 查询总数
        let count_sql = format!("SELECT COUNT(*) FROM teachers WHERE {}", where_clause);
        let total: i64 = sqlx::query_scalar(&count_sql).fetch_one(pool).await?;

        // 查询列表
        let list_sql = format!(
            r#"
            SELECT id, sn, name, department, is_active, created_at, updated_at
            FROM teachers
            WHERE {}
            ORDER BY sn ASC
            LIMIT {} OFFSET {}
            "#,
            where_clause, per_page, offset
        );

        let teachers = sqlx::query_as::<_, Teacher>(&list_sql)
            .fetch_all(pool)
            .await?;

        Ok(TeacherListResponse {
            teachers,
            total,
            page,
            per_page,
        })
    }

    /// 根据编号获取教师
    pub async fn get_teacher_by_sn(pool: &PgPool, sn: i64) -> Result<Teacher, TeacherError> {
        let teacher = sqlx::query_as::<_, Teacher>(
            r#"
            SELECT id, sn, name, department, is_active, created_at, updated_at
            FROM teachers
            WHERE sn = $1
            "#,
        )
        .bind(sn)
        .fetch_optional(pool)
        .await?;

        teacher.ok_or_else(|| TeacherError::NotFound(format!("教师编号 {} 不存在", sn)))
    }

    /// 获取有效教师列表（公开）
    /// 当 with_resources_only 为 true 时，只返回有关联资源的教师
    pub async fn get_active_teachers(
        pool: &PgPool,
        with_resources_only: bool,
    ) -> Result<Vec<Teacher>, TeacherError> {
        let sql = if with_resources_only {
            r#"
            SELECT DISTINCT t.id, t.sn, t.name, t.department, t.is_active, t.created_at, t.updated_at
            FROM teachers t
            INNER JOIN resource_teachers rt ON t.sn = rt.teacher_sn
            WHERE t.is_active = true
            ORDER BY t.sn ASC
            "#
        } else {
            r#"
            SELECT id, sn, name, department, is_active, created_at, updated_at
            FROM teachers
            WHERE is_active = true
            ORDER BY sn ASC
            "#
        };

        let teachers = sqlx::query_as::<_, Teacher>(sql).fetch_all(pool).await?;

        Ok(teachers)
    }

    /// 更新教师信息
    pub async fn update_teacher(
        pool: &PgPool,
        sn: i64,
        req: UpdateTeacherRequest,
    ) -> Result<Teacher, TeacherError> {
        // 验证请求
        if let Err(e) = req.validate() {
            return Err(TeacherError::ValidationError(e));
        }

        // 检查教师是否存在
        let existing = Self::get_teacher_by_sn(pool, sn).await?;

        // 构建更新语句
        let mut updates = vec![];
        if let Some(name) = req.name {
            updates.push(format!("name = '{}'", name));
        }
        if let Some(department) = req.department {
            updates.push(format!("department = '{}'", department));
        }

        if updates.is_empty() {
            return Ok(existing);
        }

        let update_sql = format!(
            "UPDATE teachers SET {}, updated_at = CURRENT_TIMESTAMP WHERE sn = $1 RETURNING id, sn, name, department, is_active, created_at, updated_at",
            updates.join(", ")
        );

        let teacher = sqlx::query_as::<_, Teacher>(&update_sql)
            .bind(sn)
            .fetch_one(pool)
            .await?;

        Ok(teacher)
    }

    /// 更新教师状态
    pub async fn update_teacher_status(
        pool: &PgPool,
        sn: i64,
        req: UpdateTeacherStatusRequest,
    ) -> Result<Teacher, TeacherError> {
        let teacher = sqlx::query_as::<_, Teacher>(
            r#"
            UPDATE teachers
            SET is_active = $1, updated_at = CURRENT_TIMESTAMP
            WHERE sn = $2
            RETURNING id, sn, name, department, is_active, created_at, updated_at
            "#,
        )
        .bind(req.is_active)
        .bind(sn)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("no rows") {
                TeacherError::NotFound(format!("教师编号 {} 不存在", sn))
            } else {
                TeacherError::Database(e)
            }
        })?;

        Ok(teacher)
    }

    /// 删除教师
    pub async fn delete_teacher(pool: &PgPool, sn: i64) -> Result<(), TeacherError> {
        let result = sqlx::query("DELETE FROM teachers WHERE sn = $1")
            .bind(sn)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(TeacherError::NotFound(format!("教师编号 {} 不存在", sn)));
        }

        Ok(())
    }

    /// 批量导入教师
    pub async fn batch_import_teachers(
        pool: &PgPool,
        items: Vec<BatchImportTeacherItem>,
    ) -> Result<BatchImportTeachersResult, TeacherError> {
        let mut success_count = 0i32;
        let mut fail_count = 0i32;
        let mut failed_items: Vec<FailedTeacherImportItem> = Vec::new();

        for item in items {
            // 验证数据
            if item.name.trim().is_empty() {
                fail_count += 1;
                failed_items.push(FailedTeacherImportItem {
                    name: item.name.clone(),
                    reason: "教师姓名不能为空".to_string(),
                });
                continue;
            }

            if item.name.len() > 100 {
                fail_count += 1;
                failed_items.push(FailedTeacherImportItem {
                    name: item.name.clone(),
                    reason: "教师姓名不能超过100个字符".to_string(),
                });
                continue;
            }

            if let Some(ref dept) = item.department {
                if dept.len() > 100 {
                    fail_count += 1;
                    failed_items.push(FailedTeacherImportItem {
                        name: item.name.clone(),
                        reason: "学院名称不能超过100个字符".to_string(),
                    });
                    continue;
                }
            }

            // 检查是否已存在同名教师（同姓名+同学院视为重复）
            let existing: Option<(i64,)> = if let Some(ref dept) = item.department {
                sqlx::query_as(
                    "SELECT sn FROM teachers WHERE name = $1 AND department = $2 LIMIT 1",
                )
                .bind(&item.name)
                .bind(dept)
                .fetch_optional(pool)
                .await?
            } else {
                sqlx::query_as(
                    "SELECT sn FROM teachers WHERE name = $1 AND department IS NULL LIMIT 1",
                )
                .bind(&item.name)
                .fetch_optional(pool)
                .await?
            };

            if existing.is_some() {
                fail_count += 1;
                failed_items.push(FailedTeacherImportItem {
                    name: item.name.clone(),
                    reason: "教师姓名在该学院已存在".to_string(),
                });
                continue;
            }

            // 插入教师
            let result = sqlx::query(
                r#"
                INSERT INTO teachers (name, department, is_active)
                VALUES ($1, $2, true)
                "#,
            )
            .bind(&item.name)
            .bind(&item.department)
            .execute(pool)
            .await;

            match result {
                Ok(_) => {
                    success_count += 1;
                }
                Err(e) => {
                    fail_count += 1;
                    failed_items.push(FailedTeacherImportItem {
                        name: item.name.clone(),
                        reason: format!("数据库错误: {}", e),
                    });
                }
            }
        }

        Ok(BatchImportTeachersResult {
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

    /// 批量删除教师
    pub async fn batch_delete_teachers(
        pool: &PgPool,
        sns_str: &str,
    ) -> Result<BatchDeleteTeachersResult, TeacherError> {
        // 解析编号列表
        let sns = match Self::parse_sn_ranges(sns_str) {
            Ok(sns) => sns,
            Err(e) => return Err(TeacherError::ValidationError(e)),
        };

        if sns.is_empty() {
            return Err(TeacherError::ValidationError(
                "编号列表不能为空".to_string(),
            ));
        }

        let mut success_count = 0i32;
        let mut not_found_count = 0i32;
        let mut failed_items: Vec<FailedTeacherDeleteItem> = Vec::new();

        // 逐个删除（由于需要准确反馈每条记录的结果，使用循环删除而非批量删除）
        for sn in sns {
            match sqlx::query("DELETE FROM teachers WHERE sn = $1")
                .bind(sn)
                .execute(pool)
                .await
            {
                Ok(result) => {
                    if result.rows_affected() > 0 {
                        success_count += 1;
                    } else {
                        not_found_count += 1;
                        failed_items.push(FailedTeacherDeleteItem {
                            sn,
                            reason: "教师编号不存在".to_string(),
                        });
                    }
                }
                Err(e) => {
                    failed_items.push(FailedTeacherDeleteItem {
                        sn,
                        reason: format!("数据库错误: {}", e),
                    });
                }
            }
        }

        Ok(BatchDeleteTeachersResult {
            success_count,
            fail_count: failed_items.len() as i32,
            not_found_count,
            failed_items,
        })
    }
}
