use sqlx::postgres::{PgPool, PgPoolOptions};
use std::sync::Arc;
use std::time::Duration;

use crate::config::BrandConfig;
use crate::services::StorageBackend;

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
        .max_connections(20) // 最大连接数
        .min_connections(5) // 最小连接数
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
    pub storage: Arc<dyn StorageBackend>,
    /// 注册时是否强制要求邮箱
    pub require_email_on_register: bool,
    /// 是否允许用户修改用户名
    pub allow_username_change: bool,
    /// 是否允许用户修改邮箱
    pub allow_email_change: bool,
    /// 品牌配置
    pub brand: BrandConfig,
    /// PDF 预览检测资源 UUID
    pub pdf_preview_challenge_uuid: Option<String>,
    /// PDF 预览检测验证码
    pub pdf_preview_challenge_code: Option<String>,
}

impl AppState {
    pub fn new(
        pool: PgPool,
        jwt_secret: String,
        cookie_secure: bool,
        storage: Arc<dyn StorageBackend>,
        require_email_on_register: bool,
        allow_username_change: bool,
        allow_email_change: bool,
        brand: BrandConfig,
        pdf_preview_challenge_uuid: Option<String>,
        pdf_preview_challenge_code: Option<String>,
    ) -> Self {
        Self {
            pool,
            jwt_secret,
            cookie_secure,
            storage,
            require_email_on_register,
            allow_username_change,
            allow_email_change,
            brand,
            pdf_preview_challenge_uuid,
            pdf_preview_challenge_code,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试数据库连接池配置选项
    ///
    /// 注：这是一个单元测试，测试配置选项的构建。
    /// 实际的数据库连接测试需要外部 PostgreSQL 服务，应在集成测试中完成。
    #[test]
    fn test_pool_options_configuration() {
        // 测试连接池选项配置
        let pool_options = PgPoolOptions::new()
            .max_connections(20)
            .min_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .idle_timeout(Duration::from_secs(600))
            .max_lifetime(Duration::from_secs(1800));

        // 如果能构建成功，说明配置选项设置正确
        // PgPoolOptions 没有直接的 getter 方法，但我们可以通过 clone 验证它是有效的
        let _cloned = pool_options.clone();
    }

    /// 测试 AppState 的签名
    ///
    /// 验证 AppState::new 函数的签名和类型正确性
    #[test]
    fn test_app_state_signature() {
        // 这个测试主要验证代码结构和类型正确性
        // 由于无法在没有真实数据库的情况下创建 PgPool，
        // 我们只进行类型检查

        use crate::config::BrandConfig;

        // 验证 AppState::new 的参数类型
        fn _check_app_state_new_signature(
            pool: PgPool,
            jwt_secret: String,
            cookie_secure: bool,
            storage: Arc<dyn StorageBackend>,
            require_email_on_register: bool,
            allow_username_change: bool,
            allow_email_change: bool,
            brand: BrandConfig,
            pdf_preview_challenge_uuid: Option<String>,
            pdf_preview_challenge_code: Option<String>,
        ) -> AppState {
            AppState::new(
                pool,
                jwt_secret,
                cookie_secure,
                storage,
                require_email_on_register,
                allow_username_change,
                allow_email_change,
                brand,
                pdf_preview_challenge_uuid,
                pdf_preview_challenge_code,
            )
        }

        // 验证函数指针类型
        let _: fn(
            PgPool,
            String,
            bool,
            Arc<dyn StorageBackend>,
            bool,
            bool,
            bool,
            BrandConfig,
            Option<String>,
            Option<String>,
        ) -> AppState = _check_app_state_new_signature;

        // 测试通过，类型检查完成
        assert!(true);
    }

    /// 测试无效的数据库 URL 返回错误
    ///
    /// 验证当提供无效的数据库连接字符串时，create_pool 会返回错误
    #[tokio::test]
    async fn test_create_pool_with_invalid_url() {
        let invalid_url = "invalid_url_format";
        let result = create_pool(invalid_url).await;
        assert!(result.is_err(), "无效的数据库 URL 应该返回错误");
    }

    /// 测试数据库连接超时
    ///
    /// 验证当连接到不存在的数据库服务器时，会在超时时间内返回错误
    #[tokio::test]
    async fn test_create_pool_connection_timeout() {
        // 使用一个确定无法连接的地址
        // 注意：这里使用一个私有 IP 地址，通常不会有数据库服务
        let unreachable_url = "postgres://test:test@192.0.2.1:5432/test?connect_timeout=1";

        let start = std::time::Instant::now();
        let result = create_pool(unreachable_url).await;
        let elapsed = start.elapsed();

        // 应该返回错误
        assert!(result.is_err(), "无法连接的数据库应该返回错误");

        // 应该在合理的超时时间内返回（create_pool 中设置了 3 秒超时）
        assert!(
            elapsed < Duration::from_secs(10),
            "连接应该在超时时间内返回，但实际用了 {:?}",
            elapsed
        );
    }
}
