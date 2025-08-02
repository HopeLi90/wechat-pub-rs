//! # WeChat Official Account Rust SDK
//!
//! A simple, high-performance WeChat Official Account SDK for uploading articles and managing drafts.
//!
//! ## Features
//!
//! - **Simple API**: One function to upload entire articles: `wx.upload("./article.md")`
//! - **Smart Deduplication**:
//!   - Images deduplicated by BLAKE3 content hash
//!   - Drafts deduplicated by title (updates existing drafts)
//! - **Robust**: Comprehensive error handling and retry mechanisms
//! - **Fast**: Async/await with concurrent image uploads
//! - **Type Safe**: Compile-time guarantees and runtime reliability
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use wechat_pub_rs::{WeChatClient, UploadOptions, Result};
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let client = WeChatClient::new("your_app_id", "your_app_secret").await?;
//!
//!     // Upload using theme from frontmatter, or default theme
//!     let draft_id = client.upload("./article.md").await?;
//!
//!     // Or specify theme explicitly via options
//!     let options = UploadOptions::with_theme("lapis");
//!     let draft_id = client.upload_with_options("./article.md", options).await?;
//!
//!     println!("Draft created with ID: {}", draft_id);
//!     Ok(())
//! }
//! ```

pub mod auth;
pub mod client;
pub mod error;
pub mod http;
pub mod markdown;
pub mod theme;
pub mod upload;
pub mod utils;

// Re-export main types for convenience
pub use client::{UploadOptions, WeChatClient};
pub use error::{Result, WeChatError};
pub use theme::BuiltinTheme;

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_structure() {
        // Basic smoke test to ensure modules compile
        // This test verifies that all modules can be compiled successfully
        assert_eq!(1, 1);
    }
}
