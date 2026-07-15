use sqlx::PgPool;

/// 健康检查错误。
#[derive(Debug, thiserror::Error)]
pub enum HealthCheckError {
    /// 数据库探测失败。
    #[error("数据库不可达: {0}")]
    Database(#[from] sqlx::Error),
}

/// 健康检查服务。
pub struct HealthService;

impl HealthService {
    /// 检查数据库是否可用。
    pub async fn check_readiness(pool: &PgPool) -> Result<(), HealthCheckError> {
        sqlx::query("SELECT 1").execute(pool).await?;
        Ok(())
    }
}
