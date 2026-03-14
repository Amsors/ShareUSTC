use crate::config::Config;
use crate::utils::build_content_disposition;
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::StatusCode;
use serde::Deserialize;
use sha1::Sha1;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use uuid::Uuid;

use super::storage_service::{
    StorageBackend, StorageBackendType, StorageError, StorageFileMetadata, StorageFuture,
    StorageStsCredentials,
};

#[derive(Debug, Clone)]
pub struct OssConfig {
    pub access_key_id: String,
    pub access_key_secret: String,
    pub endpoint: String,
    pub bucket: String,
    pub region: Option<String>,
    pub sts_role_arn: Option<String>,
    pub sts_session_duration: u64,
    pub key_prefix: String,
    pub signed_url_expiry: u64,
}

#[derive(Debug, Clone)]
pub struct OssStorage {
    config: OssConfig,
    client: reqwest::Client,
}

impl OssStorage {
    pub fn from_config(config: &Config) -> Result<Self, StorageError> {
        let access_key_id = required(config.oss_access_key_id.as_deref(), "OSS_ACCESS_KEY_ID")?;
        let access_key_secret = required(
            config.oss_access_key_secret.as_deref(),
            "OSS_ACCESS_KEY_SECRET",
        )?;
        let endpoint = required(config.oss_endpoint.as_deref(), "OSS_ENDPOINT")?;
        let bucket = required(config.oss_bucket.as_deref(), "OSS_BUCKET")?;

        let key_prefix = config.oss_key_prefix.trim().trim_matches('/').to_string();

        let signed_url_expiry = if config.oss_signed_url_expiry == 0 {
            600
        } else {
            config.oss_signed_url_expiry
        };
        let sts_session_duration = clamp_sts_duration(config.oss_sts_session_duration);

        let client = reqwest::Client::builder()
            .build()
            .map_err(|e| StorageError::Config(format!("初始化 OSS HTTP 客户端失败: {}", e)))?;

        Ok(Self {
            config: OssConfig {
                access_key_id,
                access_key_secret,
                endpoint,
                bucket,
                region: config.oss_region.clone(),
                sts_role_arn: config.oss_sts_role_arn.clone(),
                sts_session_duration,
                key_prefix,
                signed_url_expiry,
            },
            client,
        })
    }

    fn normalize_key(&self, key: &str) -> Result<String, StorageError> {
        let key = key.trim().trim_start_matches('/').to_string();
        if key.is_empty() {
            return Err(StorageError::Validation("文件 key 不能为空".to_string()));
        }

        if self.config.key_prefix.is_empty() {
            return Ok(key);
        }

        let prefix = format!("{}/", self.config.key_prefix);
        if key == self.config.key_prefix || key.starts_with(&prefix) {
            return Ok(key);
        }

        Ok(format!("{}/{}", self.config.key_prefix, key))
    }

    fn endpoint_scheme(&self) -> &'static str {
        if self.config.endpoint.starts_with("http://") {
            "http"
        } else {
            "https"
        }
    }

    fn endpoint_host(&self) -> String {
        self.config
            .endpoint
            .trim()
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_end_matches('/')
            .to_string()
    }

    fn object_host(&self) -> String {
        format!("{}.{}", self.config.bucket, self.endpoint_host())
    }

    fn resolve_region(&self) -> Result<String, StorageError> {
        if let Some(region) = self
            .config
            .region
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
        {
            return Ok(normalize_region(&region));
        }

        let endpoint = self.endpoint_host();
        if let Some(rest) = endpoint.strip_prefix("oss-") {
            if let Some(region) = rest.split('.').next() {
                if !region.is_empty() {
                    return Ok(region.to_string());
                }
            }
        }

        Err(StorageError::Config(
            "缺少 OSS_REGION，且无法从 OSS_ENDPOINT 推断".to_string(),
        ))
    }

    async fn issue_sts_credentials(
        &self,
        key: &str,
        duration_secs: u64,
    ) -> Result<StorageStsCredentials, StorageError> {
        let role_arn = self
            .config
            .sts_role_arn
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| StorageError::Config("缺少配置: OSS_STS_ROLE_ARN".to_string()))?;

        let normalized_key = self.normalize_key(key)?;
        let region = self.resolve_region()?;
        let expires_in = clamp_sts_duration(if duration_secs == 0 {
            self.config.sts_session_duration
        } else {
            duration_secs
        });

        let session_name = format!("shareustc-{}", Uuid::new_v4().simple());
        let policy = build_assume_role_policy(&self.config.bucket, &normalized_key);

        let mut params = BTreeMap::new();
        params.insert("Action".to_string(), "AssumeRole".to_string());
        params.insert("Format".to_string(), "JSON".to_string());
        params.insert("Version".to_string(), "2015-04-01".to_string());
        params.insert("AccessKeyId".to_string(), self.config.access_key_id.clone());
        params.insert("SignatureMethod".to_string(), "HMAC-SHA1".to_string());
        params.insert(
            "Timestamp".to_string(),
            Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        );
        params.insert("SignatureVersion".to_string(), "1.0".to_string());
        params.insert("SignatureNonce".to_string(), Uuid::new_v4().to_string());
        params.insert("RoleArn".to_string(), role_arn);
        params.insert("RoleSessionName".to_string(), session_name);
        params.insert("DurationSeconds".to_string(), expires_in.to_string());
        params.insert("Policy".to_string(), policy);

        let signature = sign_sts_rpc_query(&params, &self.config.access_key_secret)?;
        params.insert("Signature".to_string(), signature);

        let response = self
            .client
            .get("https://sts.aliyuncs.com/")
            .query(&params)
            .send()
            .await
            .map_err(|e| StorageError::Backend(format!("请求 STS AssumeRole 失败: {}", e)))?;

        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        if !status.is_success() {
            if let Ok(error_resp) = serde_json::from_str::<AssumeRoleErrorResponse>(&body) {
                return Err(StorageError::Backend(format!(
                    "STS AssumeRole 失败 [{}]: {}",
                    error_resp.code.unwrap_or_else(|| "UnknownCode".to_string()),
                    error_resp
                        .message
                        .unwrap_or_else(|| "未返回详细错误信息".to_string())
                )));
            }
            return Err(StorageError::Backend(format!(
                "STS AssumeRole 失败，HTTP 状态码: {}，响应: {}",
                status, body
            )));
        }

        let assume_role_response: AssumeRoleResponse =
            serde_json::from_str(&body).map_err(|e| {
                StorageError::Backend(format!(
                    "解析 STS AssumeRole 响应失败: {}，响应: {}",
                    e, body
                ))
            })?;

        Ok(StorageStsCredentials {
            access_key_id: assume_role_response.credentials.access_key_id,
            access_key_secret: assume_role_response.credentials.access_key_secret,
            security_token: assume_role_response.credentials.security_token,
            expiration: assume_role_response.credentials.expiration,
            bucket: self.config.bucket.clone(),
            region,
            endpoint: self.endpoint_host(),
            upload_key: normalized_key,
            expires_in,
        })
    }

    fn build_presigned_url(
        &self,
        method: &str,
        key: &str,
        expires_secs: u64,
        response_content_disposition: Option<String>,
        content_type: Option<&str>,
    ) -> Result<String, StorageError> {
        let scheme = self.endpoint_scheme();
        let host = self.object_host();
        let region = self.resolve_region()?;
        let normalized_key = self.normalize_key(key)?;
        let request_uri = format!("/{}", percent_encode(&normalized_key, false));
        let canonical_uri = format!(
            "/{}/{}",
            percent_encode(&self.config.bucket, false),
            percent_encode(&normalized_key, false)
        );
        let expires = if expires_secs == 0 {
            self.config.signed_url_expiry
        } else {
            expires_secs
        };

        let now = Utc::now();
        let short_date = now.format("%Y%m%d").to_string();
        let full_date = now.format("%Y%m%dT%H%M%SZ").to_string();
        let credential_scope = format!("{}/{}/oss/aliyun_v4_request", short_date, region);
        let credential = format!("{}/{}", self.config.access_key_id, credential_scope);

        let mut query_params = BTreeMap::new();
        query_params.insert(
            "x-oss-signature-version".to_string(),
            "OSS4-HMAC-SHA256".to_string(),
        );
        query_params.insert("x-oss-credential".to_string(), credential);
        query_params.insert("x-oss-date".to_string(), full_date.clone());
        query_params.insert("x-oss-expires".to_string(), expires.to_string());

        // 构建 canonical headers 和 additional headers
        // 注意：additional headers 必须按字母顺序排列
        let mut canonical_headers = String::new();
        let mut additional_headers = Vec::new();

        // content-type 按字母顺序在 host 之前
        if let Some(ct) = content_type {
            canonical_headers.push_str(&format!("content-type:{}\n", ct));
            additional_headers.push("content-type");
        }

        canonical_headers.push_str(&format!("host:{}\n", host));
        additional_headers.push("host");

        query_params.insert(
            "x-oss-additional-headers".to_string(),
            additional_headers.join(";"),
        );

        if let Some(content_disposition) = response_content_disposition {
            query_params.insert(
                "response-content-disposition".to_string(),
                content_disposition,
            );
        }

        let canonical_query = canonical_query_string(&query_params);
        let canonical_request = format!(
            "{}\n{}\n{}\n{}\n{}\nUNSIGNED-PAYLOAD",
            method,
            canonical_uri,
            canonical_query,
            canonical_headers,
            additional_headers.join(";")
        );

        let hashed_request = hex_lower(&Sha256::digest(canonical_request.as_bytes()));
        let string_to_sign = format!(
            "OSS4-HMAC-SHA256\n{}\n{}\n{}",
            full_date, credential_scope, hashed_request
        );

        let signing_key = derive_signing_key(&self.config.access_key_secret, &short_date, &region);
        let signature = hex_lower(&hmac_sha256(&signing_key, string_to_sign.as_bytes()));

        let mut signed_params = query_params;
        signed_params.insert("x-oss-signature".to_string(), signature);
        let signed_query = canonical_query_string(&signed_params);

        Ok(format!(
            "{}://{}{}?{}",
            scheme, host, request_uri, signed_query
        ))
    }

    async fn put_object(
        &self,
        key: &str,
        data: Vec<u8>,
        content_type: Option<&str>,
    ) -> Result<String, StorageError> {
        let normalized_key = self.normalize_key(key)?;
        let presigned_url = self.build_presigned_url(
            "PUT",
            &normalized_key,
            self.config.signed_url_expiry,
            None,
            content_type,
        )?;

        let mut request = self.client.put(&presigned_url);

        if let Some(content_type) = content_type {
            request = request.header("Content-Type", content_type);
        }

        let response = request
            .body(data)
            .send()
            .await
            .map_err(|e| StorageError::Backend(format!("OSS 写入请求失败: {}", e)))?;

        if response.status().is_success() {
            Ok(normalized_key)
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(StorageError::Backend(format!(
                "OSS 写入失败，HTTP 状态码: {}，响应: {}",
                status, body
            )))
        }
    }

    async fn delete_object(&self, key: &str) -> Result<(), StorageError> {
        let normalized_key = self.normalize_key(key)?;
        let presigned_url = self.build_presigned_url(
            "DELETE",
            &normalized_key,
            self.config.signed_url_expiry,
            None,
            None,
        )?;
        let response = self
            .client
            .delete(&presigned_url)
            .send()
            .await
            .map_err(|e| StorageError::Backend(format!("OSS 删除请求失败: {}", e)))?;

        match response.status() {
            StatusCode::NOT_FOUND => Ok(()),
            status if status.is_success() => Ok(()),
            status => {
                let body = response.text().await.unwrap_or_default();
                Err(StorageError::Backend(format!(
                    "OSS 删除失败，HTTP 状态码: {}，响应: {}",
                    status, body
                )))
            }
        }
    }
}

impl StorageBackend for OssStorage {
    fn save_file<'a>(
        &'a self,
        key: &'a str,
        data: Vec<u8>,
        content_type: Option<&'a str>,
    ) -> StorageFuture<'a, String> {
        Box::pin(async move { self.put_object(key, data, content_type).await })
    }

    fn read_file<'a>(&'a self, key: &'a str) -> StorageFuture<'a, Vec<u8>> {
        Box::pin(async move {
            let signed_url =
                self.build_presigned_url("GET", key, self.config.signed_url_expiry, None, None)?;

            let response = self
                .client
                .get(&signed_url)
                .send()
                .await
                .map_err(|e| StorageError::Backend(format!("OSS 读取请求失败: {}", e)))?;

            match response.status() {
                StatusCode::OK | StatusCode::PARTIAL_CONTENT => {
                    let bytes = response
                        .bytes()
                        .await
                        .map_err(|e| StorageError::Backend(format!("OSS 响应读取失败: {}", e)))?;
                    Ok(bytes.to_vec())
                }
                StatusCode::NOT_FOUND => {
                    Err(StorageError::NotFound(format!("OSS 文件不存在: {}", key)))
                }
                status => Err(StorageError::Backend(format!(
                    "OSS 读取失败，HTTP 状态码: {}",
                    status
                ))),
            }
        })
    }

    fn write_file<'a>(
        &'a self,
        key: &'a str,
        data: Vec<u8>,
        content_type: Option<&'a str>,
    ) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            self.put_object(key, data, content_type).await?;
            Ok(())
        })
    }

    fn delete_file<'a>(&'a self, key: &'a str) -> StorageFuture<'a, ()> {
        Box::pin(async move { self.delete_object(key).await })
    }

    fn get_file_url<'a>(&'a self, key: &'a str, expires_secs: u64) -> StorageFuture<'a, String> {
        Box::pin(async move { self.build_presigned_url("GET", key, expires_secs, None, None) })
    }

    fn get_download_url<'a>(
        &'a self,
        key: &'a str,
        filename: &'a str,
        expires_secs: u64,
    ) -> StorageFuture<'a, String> {
        let content_disposition = build_content_disposition(filename);
        Box::pin(async move {
            self.build_presigned_url("GET", key, expires_secs, Some(content_disposition), None)
        })
    }

    fn get_upload_url<'a>(
        &'a self,
        key: &'a str,
        expires_secs: u64,
        _content_type: Option<&'a str>,
    ) -> StorageFuture<'a, String> {
        Box::pin(async move { self.build_presigned_url("PUT", key, expires_secs, None, None) })
    }

    fn head_file<'a>(&'a self, key: &'a str) -> StorageFuture<'a, StorageFileMetadata> {
        Box::pin(async move {
            let normalized_key = self.normalize_key(key)?;
            let presigned_url =
                self.build_presigned_url("HEAD", &normalized_key, 60, None, None)?;

            let head_response = self
                .client
                .head(&presigned_url)
                .send()
                .await
                .map_err(|e| StorageError::Backend(format!("OSS 读取文件元信息失败: {}", e)))?;

            match head_response.status() {
                StatusCode::OK => Ok(parse_storage_metadata_from_headers(
                    head_response.headers(),
                    None,
                )),
                StatusCode::NOT_FOUND => {
                    Err(StorageError::NotFound(format!("OSS 文件不存在: {}", key)))
                }
                StatusCode::FORBIDDEN | StatusCode::METHOD_NOT_ALLOWED => {
                    // 某些策略下可能未授予 HeadObject；回退到 GET Range 获取元信息。
                    let get_url =
                        self.build_presigned_url("GET", &normalized_key, 60, None, None)?;
                    let get_response = self
                        .client
                        .get(&get_url)
                        .header("Range", "bytes=0-0")
                        .send()
                        .await
                        .map_err(|e| {
                            StorageError::Backend(format!("OSS 回退读取文件元信息失败: {}", e))
                        })?;

                    match get_response.status() {
                        StatusCode::OK | StatusCode::PARTIAL_CONTENT => {
                            Ok(parse_storage_metadata_from_headers(
                                get_response.headers(),
                                parse_total_length_from_content_range(
                                    get_response
                                        .headers()
                                        .get(reqwest::header::CONTENT_RANGE)
                                        .and_then(|v| v.to_str().ok()),
                                ),
                            ))
                        }
                        StatusCode::NOT_FOUND => {
                            Err(StorageError::NotFound(format!("OSS 文件不存在: {}", key)))
                        }
                        status => {
                            let body = get_response.text().await.unwrap_or_default();
                            Err(StorageError::Backend(format!(
                                "OSS 回退读取文件元信息失败，HTTP 状态码: {}，响应: {}",
                                status, body
                            )))
                        }
                    }
                }
                status => {
                    let body = head_response.text().await.unwrap_or_default();
                    Err(StorageError::Backend(format!(
                        "OSS 读取文件元信息失败，HTTP 状态码: {}，响应: {}",
                        status, body
                    )))
                }
            }
        })
    }

    fn get_sts_token<'a>(
        &'a self,
        key: &'a str,
        duration_secs: u64,
    ) -> StorageFuture<'a, StorageStsCredentials> {
        Box::pin(async move { self.issue_sts_credentials(key, duration_secs).await })
    }

    fn backend_type(&self) -> StorageBackendType {
        StorageBackendType::Oss
    }

    fn supports_sts(&self) -> bool {
        self.config.sts_role_arn.is_some()
    }

    fn default_signed_url_expiry(&self) -> u64 {
        self.config.signed_url_expiry
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AssumeRoleResponse {
    credentials: AssumeRoleCredentials,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AssumeRoleCredentials {
    access_key_id: String,
    access_key_secret: String,
    security_token: String,
    expiration: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AssumeRoleErrorResponse {
    code: Option<String>,
    message: Option<String>,
}

fn clamp_sts_duration(duration_secs: u64) -> u64 {
    duration_secs.clamp(900, 3600)
}

fn normalize_region(region: &str) -> String {
    let trimmed = region.trim();
    if let Some(stripped) = trimmed.strip_prefix("oss-") {
        return stripped.to_string();
    }
    trimmed.to_string()
}

fn build_assume_role_policy(bucket: &str, normalized_key: &str) -> String {
    serde_json::json!({
        "Version": "1",
        "Statement": [
            {
                "Effect": "Allow",
                "Action": ["oss:PutObject"],
                "Resource": [format!("acs:oss:*:*:{}/{}", bucket, normalized_key)]
            }
        ]
    })
    .to_string()
}

fn sign_sts_rpc_query(
    params: &BTreeMap<String, String>,
    access_key_secret: &str,
) -> Result<String, StorageError> {
    let canonicalized = params
        .iter()
        .map(|(k, v)| format!("{}={}", percent_encode(k, true), percent_encode(v, true)))
        .collect::<Vec<_>>()
        .join("&");
    let string_to_sign = format!("GET&%2F&{}", percent_encode(&canonicalized, true));
    let signing_key = format!("{}&", access_key_secret);

    let mut mac = Hmac::<Sha1>::new_from_slice(signing_key.as_bytes())
        .map_err(|e| StorageError::Backend(format!("STS 签名初始化失败: {}", e)))?;
    mac.update(string_to_sign.as_bytes());
    let signature = BASE64_STANDARD.encode(mac.finalize().into_bytes());
    Ok(signature)
}

fn required(value: Option<&str>, env_name: &str) -> Result<String, StorageError> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| StorageError::Config(format!("缺少配置: {}", env_name)))
}

fn canonical_query_string(params: &BTreeMap<String, String>) -> String {
    params
        .iter()
        .map(|(k, v)| format!("{}={}", percent_encode(k, true), percent_encode(v, true)))
        .collect::<Vec<_>>()
        .join("&")
}

fn parse_storage_metadata_from_headers(
    headers: &reqwest::header::HeaderMap,
    content_length_override: Option<u64>,
) -> StorageFileMetadata {
    let content_length = content_length_override.or_else(|| {
        headers
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
    });
    let content_type = headers
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string());
    let etag = headers
        .get(reqwest::header::ETAG)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.trim_matches('"').to_string());

    StorageFileMetadata {
        content_length,
        content_type,
        etag,
    }
}

fn parse_total_length_from_content_range(content_range: Option<&str>) -> Option<u64> {
    let value = content_range?;
    let total = value.rsplit('/').next()?;
    if total == "*" {
        return None;
    }
    total.parse::<u64>().ok()
}

fn percent_encode(input: &str, encode_slash: bool) -> String {
    let mut encoded = String::new();
    for &byte in input.as_bytes() {
        let is_unreserved =
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~');
        let is_slash = byte == b'/';
        if is_unreserved || (!encode_slash && is_slash) {
            encoded.push(byte as char);
        } else {
            encoded.push('%');
            encoded.push_str(&format!("{:02X}", byte));
        }
    }
    encoded
}

fn hex_lower(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}

fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    const BLOCK_SIZE: usize = 64;
    let mut key_block = [0u8; BLOCK_SIZE];

    if key.len() > BLOCK_SIZE {
        let digest = Sha256::digest(key);
        key_block[..32].copy_from_slice(&digest);
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }

    let mut ipad = [0u8; BLOCK_SIZE];
    let mut opad = [0u8; BLOCK_SIZE];
    for i in 0..BLOCK_SIZE {
        ipad[i] = key_block[i] ^ 0x36;
        opad[i] = key_block[i] ^ 0x5c;
    }

    let mut inner = Sha256::new();
    inner.update(ipad);
    inner.update(message);
    let inner_hash = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(opad);
    outer.update(inner_hash);
    let output = outer.finalize();

    let mut result = [0u8; 32];
    result.copy_from_slice(&output);
    result
}

fn derive_signing_key(secret: &str, short_date: &str, region: &str) -> [u8; 32] {
    let secret_key = format!("aliyun_v4{}", secret);
    let date_key = hmac_sha256(secret_key.as_bytes(), short_date.as_bytes());
    let region_key = hmac_sha256(&date_key, region.as_bytes());
    let service_key = hmac_sha256(&region_key, b"oss");
    hmac_sha256(&service_key, b"aliyun_v4_request")
}
