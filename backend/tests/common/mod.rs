#![allow(dead_code, clippy::expect_used)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use backend::config::{BrandConfig, Config};
use backend::models::{
    CurrentUser, RegisterRequest, ResourceCategory, ResourceType, UploadResourceRequest, UserRole,
};
use backend::services::{
    AuthService, StorageBackend, StorageBackendType, StorageError, StorageFileMetadata,
    StorageFuture, StorageStsCredentials,
};
use sqlx::PgPool;
use uuid::Uuid;

pub const JWT_SECRET: &str = "stage6-test-secret-with-sufficient-length";

/// 构造测试用配置。集成测试不经过 `Config::from_env()`（其对缺失必填项会退出进程），
/// 这里给出一份合法的本地/内存存储配置，供 `AppState::new` 使用。
pub fn test_app_config() -> Config {
    Config {
        database_url: "postgres://test:test@localhost/shareustc_test".to_string(),
        jwt_secret: JWT_SECRET.to_string(),
        server_host: "127.0.0.1".to_string(),
        server_port: 8080,
        cors_allowed_origins: vec!["http://localhost:5173".to_string()],
        admin_usernames: vec![],
        cookie_secure: false,
        image_base_url: "http://localhost:8080".to_string(),
        file_upload_path: "./uploads".to_string(),
        storage_backend: "local".to_string(),
        oss_access_key_id: None,
        oss_access_key_secret: None,
        oss_endpoint: None,
        oss_bucket: None,
        oss_region: None,
        oss_sts_role_arn: None,
        oss_sts_session_duration: 900,
        oss_key_prefix: String::new(),
        oss_signed_url_expiry: 600,
        require_email_on_register: false,
        allow_username_change: true,
        allow_email_change: true,
        brand: BrandConfig::default(),
        pdf_preview_challenge_uuid: None,
        pdf_preview_challenge_code: None,
    }
}

#[derive(Default)]
pub struct MemoryStorage {
    files: Mutex<HashMap<String, Vec<u8>>>,
}

impl MemoryStorage {
    pub fn shared() -> Arc<dyn StorageBackend> {
        Arc::new(Self::default())
    }
}

impl StorageBackend for MemoryStorage {
    fn save_file<'a>(
        &'a self,
        key: &'a str,
        data: Vec<u8>,
        _content_type: Option<&'a str>,
    ) -> StorageFuture<'a, String> {
        Box::pin(async move {
            self.files
                .lock()
                .map_err(|_| StorageError::Backend("测试存储锁已污染".to_string()))?
                .insert(key.to_string(), data);
            Ok(key.to_string())
        })
    }

    fn read_file<'a>(&'a self, key: &'a str) -> StorageFuture<'a, Vec<u8>> {
        Box::pin(async move {
            self.files
                .lock()
                .map_err(|_| StorageError::Backend("测试存储锁已污染".to_string()))?
                .get(key)
                .cloned()
                .ok_or_else(|| StorageError::NotFound(key.to_string()))
        })
    }

    fn write_file<'a>(
        &'a self,
        key: &'a str,
        data: Vec<u8>,
        _content_type: Option<&'a str>,
    ) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            self.files
                .lock()
                .map_err(|_| StorageError::Backend("测试存储锁已污染".to_string()))?
                .insert(key.to_string(), data);
            Ok(())
        })
    }

    fn delete_file<'a>(&'a self, key: &'a str) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            self.files
                .lock()
                .map_err(|_| StorageError::Backend("测试存储锁已污染".to_string()))?
                .remove(key);
            Ok(())
        })
    }

    fn get_file_url<'a>(&'a self, key: &'a str, _expires_secs: u64) -> StorageFuture<'a, String> {
        Box::pin(async move { Ok(format!("memory://{key}")) })
    }

    fn get_download_url<'a>(
        &'a self,
        key: &'a str,
        _filename: &'a str,
        _expires_secs: u64,
    ) -> StorageFuture<'a, String> {
        self.get_file_url(key, 0)
    }

    fn get_upload_url<'a>(
        &'a self,
        key: &'a str,
        _expires_secs: u64,
        _content_type: Option<&'a str>,
    ) -> StorageFuture<'a, String> {
        self.get_file_url(key, 0)
    }

    fn head_file<'a>(&'a self, key: &'a str) -> StorageFuture<'a, StorageFileMetadata> {
        Box::pin(async move {
            let files = self
                .files
                .lock()
                .map_err(|_| StorageError::Backend("测试存储锁已污染".to_string()))?;
            let data = files
                .get(key)
                .ok_or_else(|| StorageError::NotFound(key.to_string()))?;
            Ok(StorageFileMetadata {
                content_length: Some(data.len() as u64),
                content_type: None,
                etag: None,
            })
        })
    }

    fn get_sts_token<'a>(
        &'a self,
        _key: &'a str,
        _duration_secs: u64,
    ) -> StorageFuture<'a, StorageStsCredentials> {
        Box::pin(async { Err(StorageError::Backend("测试存储不支持 STS".to_string())) })
    }

    fn backend_type(&self) -> StorageBackendType {
        StorageBackendType::Local
    }
}

pub async fn create_user(pool: &PgPool, username: &str) -> CurrentUser {
    let response = AuthService::register(
        pool,
        JWT_SECRET,
        RegisterRequest {
            username: username.to_string(),
            password: "correct-password".to_string(),
            email: Some(format!("{username}@example.com")),
        },
        true,
        &[],
    )
    .await
    .expect("测试用户应创建成功");

    CurrentUser {
        id: response.user.id,
        username: response.user.username,
        role: UserRole::User,
        is_verified: false,
    }
}

pub fn upload_request(title: &str) -> UploadResourceRequest {
    UploadResourceRequest {
        title: title.to_string(),
        course_name: Some("测试课程".to_string()),
        resource_type: ResourceType::Pdf,
        category: ResourceCategory::Lecture,
        tags: Some(vec!["集成测试".to_string()]),
        description: Some("阶段 6 集成测试资源".to_string()),
        teacher_sns: None,
        course_sns: None,
        related_resource_ids: None,
    }
}

pub async fn seed_resource(pool: &PgPool, uploader_id: Uuid, title: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO resources
           (id, title, uploader_id, resource_type, category, file_path, file_size, audit_status, storage_type)
           VALUES ($1, $2, $3, 'pdf', 'lecture', $4, 4, 'approved', 'local')"#,
    )
    .bind(id)
    .bind(title)
    .bind(uploader_id)
    .bind(format!("resources/{id}.pdf"))
    .execute(pool)
    .await
    .expect("测试资源应插入成功");
    sqlx::query("INSERT INTO resource_stats (resource_id) VALUES ($1)")
        .bind(id)
        .execute(pool)
        .await
        .expect("测试资源统计应插入成功");
    id
}
