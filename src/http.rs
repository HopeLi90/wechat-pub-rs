//! HTTP client module with retry mechanisms and WeChat API integration.

use crate::error::{Result, WeChatError};
use reqwest::{Client, Response, multipart};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn};

/// Configuration for retry behavior.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Base delay between retries
    pub base_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Exponential backoff factor
    pub backoff_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            backoff_factor: 2.0,
        }
    }
}

/// HTTP client wrapper for WeChat API calls with automatic retry and token management.
#[derive(Debug, Clone)]
pub struct WeChatHttpClient {
    client: Client,
    base_url: String,
    retry_config: RetryConfig,
}

impl WeChatHttpClient {
    /// Creates a new WeChat HTTP client.
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()?;

        Ok(Self {
            client,
            base_url: "https://api.weixin.qq.com".to_string(),
            retry_config: RetryConfig::default(),
        })
    }

    /// Creates a new client with custom retry configuration.
    pub fn with_retry_config(retry_config: RetryConfig) -> Result<Self> {
        let mut client = Self::new()?;
        client.retry_config = retry_config;
        Ok(client)
    }

    /// Makes a GET request with access token.
    pub async fn get_with_token(&self, endpoint: &str, access_token: &str) -> Result<Response> {
        let url = format!(
            "{}{}?access_token={}",
            self.base_url, endpoint, access_token
        );
        self.execute_with_retry(|| self.client.get(&url).send())
            .await
    }

    /// Makes a POST request with JSON body and access token.
    pub async fn post_json_with_token<T: Serialize>(
        &self,
        endpoint: &str,
        access_token: &str,
        body: &T,
    ) -> Result<Response> {
        let url = format!(
            "{}{}?access_token={}",
            self.base_url, endpoint, access_token
        );
        self.execute_with_retry(|| self.client.post(&url).json(body).send())
            .await
    }

    /// Uploads a file using multipart form data.
    pub async fn upload_file(
        &self,
        endpoint: &str,
        access_token: &str,
        field_name: &str,
        file_data: Vec<u8>,
        filename: &str,
    ) -> Result<Response> {
        let url = format!(
            "{}{}?access_token={}",
            self.base_url, endpoint, access_token
        );

        // Guess MIME type from filename
        let mime_type = mime_guess::from_path(filename)
            .first_or_octet_stream()
            .to_string();

        // Clone data for each retry attempt
        let field_name = field_name.to_string();
        let filename = filename.to_string();
        let url = url.clone();
        let client = self.client.clone();

        self.execute_with_retry(move || {
            let part = multipart::Part::bytes(file_data.clone())
                .file_name(filename.clone())
                .mime_str(&mime_type)
                .unwrap();
            let form = multipart::Form::new().part(field_name.clone(), part);
            client.post(&url).multipart(form).send()
        })
        .await
    }

    /// Uploads a permanent material (for cover images).
    pub async fn upload_material(
        &self,
        access_token: &str,
        material_type: &str,
        file_data: Vec<u8>,
        filename: &str,
    ) -> Result<Response> {
        let url = format!(
            "{}{}?access_token={}&type={}",
            self.base_url, "/cgi-bin/material/add_material", access_token, material_type
        );

        // Guess MIME type from filename
        let mime_type = mime_guess::from_path(filename)
            .first_or_octet_stream()
            .to_string();

        // Clone data for each retry attempt
        let filename = filename.to_string();
        let url = url.clone();
        let client = self.client.clone();

        self.execute_with_retry(move || {
            let part = multipart::Part::bytes(file_data.clone())
                .file_name(filename.clone())
                .mime_str(&mime_type)
                .unwrap();

            let form = multipart::Form::new().part("media", part);

            client.post(&url).multipart(form).send()
        })
        .await
    }

    /// Executes a request with retry logic.
    async fn execute_with_retry<F, Fut>(&self, mut operation: F) -> Result<Response>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = std::result::Result<Response, reqwest::Error>>,
    {
        let mut delay = self.retry_config.base_delay;
        let mut last_error = None;

        for attempt in 1..=self.retry_config.max_attempts {
            match operation().await {
                Ok(response) => {
                    // Check for WeChat API errors in successful HTTP responses
                    if response.status().is_success() {
                        return Ok(response);
                    } else {
                        // Convert HTTP error to WeChatError
                        let status = response.status();
                        let error_text = response
                            .text()
                            .await
                            .unwrap_or_else(|_| "Unknown error".to_string());

                        let error = WeChatError::Internal(anyhow::anyhow!(
                            "HTTP {}: {}",
                            status,
                            error_text
                        ));

                        if attempt == self.retry_config.max_attempts || !error.is_retryable() {
                            return Err(error);
                        }

                        last_error = Some(error);
                    }
                }
                Err(e) => {
                    let error = WeChatError::Network(e);

                    if attempt == self.retry_config.max_attempts || !error.is_retryable() {
                        return Err(error);
                    }

                    last_error = Some(error);
                }
            }

            // Wait before retry
            if attempt < self.retry_config.max_attempts {
                warn!(
                    "Request failed (attempt {}/{}), retrying in {:?}",
                    attempt, self.retry_config.max_attempts, delay
                );

                sleep(delay).await;

                // Exponential backoff with jitter
                delay = std::cmp::min(
                    Duration::from_millis(
                        (delay.as_millis() as f64 * self.retry_config.backoff_factor) as u64,
                    ),
                    self.retry_config.max_delay,
                );
            }
        }

        Err(last_error.unwrap_or_else(|| {
            WeChatError::Internal(anyhow::anyhow!("Retry loop completed without error"))
        }))
    }

    /// Downloads content from a URL.
    pub async fn download(&self, url: &str) -> Result<Vec<u8>> {
        let response = self
            .execute_with_retry(|| self.client.get(url).send())
            .await?;

        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }

    /// Downloads content from a URL with size limits and streaming.
    pub async fn download_with_limit(&self, url: &str, max_size: u64) -> Result<Vec<u8>> {
        use futures::StreamExt;

        let response = self
            .execute_with_retry(|| self.client.get(url).send())
            .await?;

        // Check content length if available
        if let Some(content_length) = response.content_length() {
            if content_length > max_size {
                return Err(WeChatError::ImageUpload {
                    path: url.to_string(),
                    reason: format!(
                        "Content too large: {content_length} bytes (max: {max_size} bytes)"
                    ),
                });
            }
        }

        let mut downloaded_size = 0u64;
        let mut data = Vec::new();
        let mut stream = response.bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            downloaded_size += chunk.len() as u64;

            if downloaded_size > max_size {
                return Err(WeChatError::ImageUpload {
                    path: url.to_string(),
                    reason: format!(
                        "Content too large during download: {downloaded_size} bytes (max: {max_size} bytes)"
                    ),
                });
            }

            data.extend_from_slice(&chunk);
        }

        debug!("Downloaded {downloaded_size} bytes from {url}");
        Ok(data)
    }
}

/// Standard WeChat API response structure.
#[derive(Debug, Deserialize, Serialize)]
pub struct WeChatResponse<T> {
    /// Error code (0 for success)
    #[serde(default)]
    pub errcode: i32,
    /// Error message
    #[serde(default)]
    pub errmsg: String,
    /// Response data (flattened)
    #[serde(flatten)]
    pub data: Option<T>,
}

impl<T: std::fmt::Debug> WeChatResponse<T> {
    /// Converts the response to a Result, checking for API errors.
    pub fn into_result(self) -> Result<T> {
        if self.errcode == 0 {
            self.data.ok_or_else(|| {
                WeChatError::Internal(anyhow::anyhow!(
                    "Missing response data. errcode: {}, errmsg: {}",
                    self.errcode,
                    self.errmsg
                ))
            })
        } else {
            Err(WeChatError::from_api_response(self.errcode, self.errmsg))
        }
    }
}

/// Access token response from WeChat API.
#[derive(Debug, Deserialize, Serialize)]
pub struct AccessTokenResponse {
    pub access_token: String,
    pub expires_in: u64,
}

/// Image upload response from WeChat API (uploadimg endpoint).
#[derive(Debug, Deserialize, Serialize)]
pub struct ImageUploadResponse {
    pub url: String,
}

/// Material upload response from WeChat API (for permanent materials like cover images).
#[derive(Debug, Deserialize, Serialize)]
pub struct MaterialUploadResponse {
    pub media_id: String,
    pub url: String,
}

/// Draft creation response from WeChat API.
#[derive(Debug, Deserialize, Serialize)]
pub struct DraftResponse {
    pub media_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_http_client_creation() {
        let client = WeChatHttpClient::new();
        assert!(client.is_ok());
    }

    #[test]
    fn test_retry_config() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.base_delay, Duration::from_millis(500));
        assert_eq!(config.backoff_factor, 2.0);
    }

    #[test]
    fn test_wechat_response_success() {
        let response: WeChatResponse<AccessTokenResponse> = WeChatResponse {
            errcode: 0,
            errmsg: "ok".to_string(),
            data: Some(AccessTokenResponse {
                access_token: "test_token".to_string(),
                expires_in: 7200,
            }),
        };

        let result = response.into_result();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().access_token, "test_token");
    }

    #[test]
    fn test_wechat_response_error() {
        let response: WeChatResponse<AccessTokenResponse> = WeChatResponse {
            errcode: 40001,
            errmsg: "invalid credential".to_string(),
            data: None,
        };

        let result = response.into_result();
        assert!(result.is_err());

        if let Err(WeChatError::WeChatApi { code, message }) = result {
            assert_eq!(code, 40001);
            assert_eq!(message, "invalid credential");
        } else {
            panic!("Expected WeChatApi error");
        }
    }
}
