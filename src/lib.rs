//! # WeChat Official Account Rust SDK
//!
//! A simple, high-performance WeChat Official Account SDK for uploading articles and managing drafts.
//!
//! ## Features
//!
//! - **Simple API**: One function to upload entire articles: `wx.upload("./article.md", "theme1")`
//! - **Robust**: Comprehensive error handling and retry mechanisms
//! - **Fast**: Async/await with concurrent image uploads
//! - **Type Safe**: Compile-time guarantees and runtime reliability
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use wechat_pub_rs::{WeChatClient, Result};
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let client = WeChatClient::new("your_app_id", "your_app_secret").await?;
//!     let draft_id = client.upload("./article.md", "default").await?;
//!     println!("Draft created with ID: {}", draft_id);
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod client;
pub mod http;
pub mod auth;
pub mod upload;
pub mod markdown;
pub mod theme;
pub mod utils;

// Re-export main types for convenience
pub use client::{WeChatClient, UploadOptions};
pub use error::{WeChatError, Result};
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
