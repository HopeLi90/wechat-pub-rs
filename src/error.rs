//! Error types and handling for the WeChat Official Account SDK.
//!
//! This module provides comprehensive error handling with specific error types for different
//! failure scenarios. Errors are designed to be actionable and include retry logic.
//!
//! ## Error Categories
//!
//! - **Network Errors**: Connection issues, timeouts (retryable)
//! - **Authentication Errors**: Invalid tokens, credentials (retryable once)
//! - **File System Errors**: Missing files, read failures (not retryable)
//! - **Parsing Errors**: Markdown, JSON parsing failures (not retryable)
//! - **WeChat API Errors**: Server responses with error codes (situational)
//! - **Configuration Errors**: Invalid settings (not retryable)
//!
//! ## Usage
//!
//! ```rust
//! use wechat_pub_rs::{WeChatError, Result};
//! use wechat_pub_rs::error::ErrorSeverity;
//!
//! fn handle_error(error: WeChatError) {
//!     match error.severity() {
//!         ErrorSeverity::Warning => {
//!             log::warn!("Recoverable error: {}", error);
//!             if error.is_retryable() {
//!                 // Implement retry logic
//!             }
//!         }
//!         ErrorSeverity::Error => {
//!             log::error!("Error occurred: {}", error);
//!         }
//!         ErrorSeverity::Critical => {
//!             log::error!("Critical error: {}", error);
//!             // May require immediate attention
//!         }
//!     }
//! }
//! ```

use std::fmt;

/// Result type alias for WeChat SDK operations.
pub type Result<T> = std::result::Result<T, WeChatError>;

/// Comprehensive error type for WeChat SDK operations.
#[derive(Debug, thiserror::Error)]
pub enum WeChatError {
    /// Network-related errors (retryable)
    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),

    /// Request timeout (retryable)
    #[error("Request timeout")]
    Timeout,

    /// Authentication errors (may be retryable once)
    #[error("Invalid access token")]
    InvalidToken,

    #[error("Invalid application credentials")]
    InvalidCredentials,

    /// File system errors (not retryable)
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Failed to read file: {path}, reason: {reason}")]
    FileRead { path: String, reason: String },

    /// Markdown processing errors (not retryable)
    #[error("Markdown parsing failed: {reason}")]
    MarkdownParse { reason: String },

    /// Image processing errors (may be retryable)
    #[error("Image upload failed: {path}, reason: {reason}")]
    ImageUpload { path: String, reason: String },

    /// Theme system errors (not retryable)
    #[error("Theme not found: {theme}")]
    ThemeNotFound { theme: String },

    #[error("Theme rendering failed: {theme}, reason: {reason}")]
    ThemeRender { theme: String, reason: String },

    /// WeChat API errors (retryability depends on error code)
    #[error("WeChat API error [{code}]: {message}")]
    WeChatApi { code: i32, message: String },

    /// Configuration errors (not retryable)
    #[error("Configuration error: {message}")]
    Config { message: String },

    /// JSON serialization/deserialization errors
    #[error("JSON processing failed: {0}")]
    Json(#[from] serde_json::Error),

    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// URL parsing errors
    #[error("Invalid URL: {0}")]
    UrlParse(#[from] url::ParseError),

    /// Generic errors for wrapping other error types
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl WeChatError {
    /// Determines if an error is retryable.
    ///
    /// Network errors, timeouts, and certain WeChat API errors are retryable.
    /// Authentication errors are retryable once (token might be expired).
    /// File system, parsing, and configuration errors are not retryable.
    pub fn is_retryable(&self) -> bool {
        match self {
            // Network and timeout errors are always retryable
            WeChatError::Network(_) | WeChatError::Timeout => true,

            // Authentication errors are retryable once
            WeChatError::InvalidToken => true,

            // Some image upload errors might be retryable (network issues)
            WeChatError::ImageUpload { .. } => true,

            // WeChat API errors - check specific error codes
            WeChatError::WeChatApi { code, .. } => match code {
                // Access token related errors (retryable)
                40001 | 40014 | 42001 | 42007 => true,
                // Rate limiting (retryable with delay)
                45009 | 45011 => true,
                // Server errors (retryable)
                -1 | 50001 | 50002 => true,
                // All other API errors are not retryable
                _ => false,
            },

            // All other errors are not retryable
            _ => false,
        }
    }

    /// Gets the severity level of the error for logging purposes.
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            WeChatError::Network(_) | WeChatError::Timeout | WeChatError::ImageUpload { .. } => {
                ErrorSeverity::Warning
            }

            WeChatError::InvalidToken | WeChatError::InvalidCredentials => ErrorSeverity::Error,

            WeChatError::FileNotFound { .. }
            | WeChatError::FileRead { .. }
            | WeChatError::MarkdownParse { .. }
            | WeChatError::ThemeNotFound { .. }
            | WeChatError::Config { .. } => ErrorSeverity::Error,

            WeChatError::WeChatApi { code, .. } => match code {
                // Critical API errors
                40013 | 48001 => ErrorSeverity::Critical,
                // Regular API errors
                _ => ErrorSeverity::Error,
            },

            WeChatError::ThemeRender { .. }
            | WeChatError::Json(_)
            | WeChatError::Io(_)
            | WeChatError::UrlParse(_)
            | WeChatError::Internal(_) => ErrorSeverity::Error,
        }
    }

    /// Creates a WeChat API error from response data.
    pub fn from_api_response(code: i32, message: impl Into<String>) -> Self {
        WeChatError::WeChatApi {
            code,
            message: message.into(),
        }
    }

    /// Creates a file-related error.
    pub fn file_error(path: impl Into<String>, reason: impl Into<String>) -> Self {
        WeChatError::FileRead {
            path: path.into(),
            reason: reason.into(),
        }
    }

    /// Creates a configuration error.
    pub fn config_error(message: impl Into<String>) -> Self {
        WeChatError::Config {
            message: message.into(),
        }
    }
}

/// Error severity levels for logging and monitoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Low impact errors that don't affect core functionality
    Warning,
    /// Standard errors that affect specific operations
    Error,
    /// High impact errors that affect core functionality
    Critical,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Warning => write!(f, "WARNING"),
            ErrorSeverity::Error => write!(f, "ERROR"),
            ErrorSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_retryability() {
        // Timeout errors should be retryable
        let timeout_err = WeChatError::Timeout;
        assert!(timeout_err.is_retryable());

        // File not found should not be retryable
        let file_err = WeChatError::FileNotFound {
            path: "test.md".to_string(),
        };
        assert!(!file_err.is_retryable());

        // Token errors should be retryable
        let token_err = WeChatError::from_api_response(40001, "invalid credential");
        assert!(token_err.is_retryable());

        // Invalid parameter errors should not be retryable
        let param_err = WeChatError::from_api_response(40003, "invalid openid");
        assert!(!param_err.is_retryable());
    }

    #[test]
    fn test_error_severity() {
        let network_err = WeChatError::Timeout;
        assert_eq!(network_err.severity(), ErrorSeverity::Warning);

        let config_err = WeChatError::config_error("missing app_id");
        assert_eq!(config_err.severity(), ErrorSeverity::Error);

        let critical_api_err = WeChatError::from_api_response(40013, "invalid appid");
        assert_eq!(critical_api_err.severity(), ErrorSeverity::Critical);
    }

    #[test]
    fn test_error_creation_helpers() {
        let file_err = WeChatError::file_error("/path/to/file.md", "permission denied");
        match file_err {
            WeChatError::FileRead { path, reason } => {
                assert_eq!(path, "/path/to/file.md");
                assert_eq!(reason, "permission denied");
            }
            _ => panic!("Expected FileRead error"),
        }

        let config_err = WeChatError::config_error("invalid configuration");
        match config_err {
            WeChatError::Config { message } => {
                assert_eq!(message, "invalid configuration");
            }
            _ => panic!("Expected Config error"),
        }
    }
}
