use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

/// 创建数据库连接池
///
/// # Arguments
/// * `database_url` - 数据库连接字符串
///
/// # Returns
/// * `Ok(PgPool)` - 数据库连接池
/// * `Err(sqlx::Error)` - 连接错误
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(20)                    // 最大连接数
        .min_connections(5)                     // 最小连接数
        .acquire_timeout(Duration::from_secs(3)) // 获取连接超时
        .idle_timeout(Duration::from_secs(600)) // 空闲连接超时
        .max_lifetime(Duration::from_secs(1800)) // 连接最大生命周期
        .connect(database_url)
        .await?;

    // 测试连接
    sqlx::query("SELECT 1").fetch_one(&pool).await?;

    log::info!("数据库连接池创建成功");
    Ok(pool)
}

/// 应用状态，包含数据库连接池
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
    pub cookie_secure: bool,
}

impl AppState {
    pub fn new(pool: PgPool, jwt_secret: String, cookie_secure: bool) -> Self {
        Self {
            pool,
            jwt_secret,
            cookie_secure,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_pool() {
        // 从环境变量获取数据库URL
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://shareustc_app:.gitignore@localhost:5432/shareustc".to_string());

        let result = create_pool(&database_url).await;
        assert!(result.is_ok(), "数据库连接失败: {:?}", result.err());
    }
}
