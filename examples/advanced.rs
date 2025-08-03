//! Advanced example demonstrating full WeChat SDK capabilities including
//! error handling, draft management, and theme customization.

use wechat_pub_rs::{Result, UploadOptions, WeChatClient, WeChatError};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger for debugging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Get credentials from environment
    let app_id = std::env::var("WECHAT_APP_ID")
        .map_err(|_| WeChatError::config_error("WECHAT_APP_ID environment variable not set"))?;
    let app_secret = std::env::var("WECHAT_APP_SECRET")
        .map_err(|_| WeChatError::config_error("WECHAT_APP_SECRET environment variable not set"))?;

    println!("🚀 Initializing WeChat client...");
    let client = WeChatClient::new(app_id, app_secret).await?;

    // Display available themes
    println!("\n📋 Available themes:");
    for theme in client.available_themes() {
        println!("  - {theme}");
    }

    // Check token information for debugging
    if let Some(token_info) = client.get_token_info().await {
        println!("\n🔑 Token information:");
        println!("  - Expires at: {}", token_info.expires_at);
        println!(
            "  - Time until expiry: {} seconds",
            token_info.time_until_expiry.num_seconds()
        );
        println!("  - Is expired: {}", token_info.is_expired);
    }

    // Example 1: Basic upload with default settings
    println!("\n📝 Example 1: Basic upload");
    if tokio::fs::metadata("fixtures/example.md").await.is_ok() {
        match client.upload("fixtures/example.md").await {
            Ok(draft_id) => {
                println!("✅ Successfully uploaded article with draft ID: {draft_id}");

                // Get draft information
                match client.get_draft(&draft_id).await {
                    Ok(draft_info) => {
                        println!(
                            "📄 Draft info: {} articles",
                            draft_info.content.news_item.len()
                        );
                        if let Some(article) = draft_info.content.news_item.first() {
                            println!("   Title: {}", article.title);
                            println!("   Author: {}", article.author);
                        }
                    }
                    Err(e) => println!("⚠️  Could not retrieve draft info: {e}"),
                }
            }
            Err(WeChatError::FileNotFound { path }) => {
                println!("❌ File not found: {path}");
            }
            Err(WeChatError::ThemeNotFound { theme }) => {
                println!("❌ Theme not found: {theme}");
            }
            Err(WeChatError::Network { message }) => {
                println!("❌ Network error: {message}");
                println!("💡 This might be due to invalid credentials or network issues");
            }
            Err(e) => {
                println!("❌ Upload failed: {e}");
            }
        }
    } else {
        println!("⚠️  Example file not found, skipping basic upload");
    }

    // Example 2: Advanced upload with custom options
    println!("\n📝 Example 2: Advanced upload with custom options");
    let options = UploadOptions::with_theme("lapis")
        .title("Advanced Example Article")
        .author("WeChat SDK Demo")
        .show_cover(true)
        .comments(true, false)
        .source_url("https://github.com/tyrchen/wechat-pub-rs");

    if tokio::fs::metadata("fixtures/example.md").await.is_ok() {
        match client
            .upload_with_options("fixtures/example.md", options)
            .await
        {
            Ok(draft_id) => {
                println!("✅ Advanced upload successful with draft ID: {draft_id}");

                // Example 3: Update the draft
                println!("\n📝 Example 3: Updating existing draft");
                match client.update_draft(&draft_id, "fixtures/example.md").await {
                    Ok(_) => println!("✅ Draft updated successfully"),
                    Err(e) => println!("❌ Failed to update draft: {e}"),
                }
            }
            Err(e) => println!("❌ Advanced upload failed: {e}"),
        }
    }

    // Example 4: Upload single image
    println!("\n📝 Example 4: Single image upload");
    if tokio::fs::metadata("fixtures/images/02-cover.png")
        .await
        .is_ok()
    {
        match client.upload_image("fixtures/images/02-cover.png").await {
            Ok(url) => println!("✅ Image uploaded successfully: {url}"),
            Err(e) => println!("❌ Image upload failed: {e}"),
        }
    } else {
        println!("⚠️  Cover image not found, skipping image upload");
    }

    // Example 5: List drafts
    println!("\n📝 Example 5: Listing drafts");
    match client.list_drafts(0, 5).await {
        Ok(drafts) => {
            println!("✅ Found {} drafts:", drafts.len());
            for (i, draft) in drafts.iter().enumerate() {
                println!(
                    "  {}. {} articles, updated: {}",
                    i + 1,
                    draft.content.news_item.len(),
                    draft.update_time
                );
                if let Some(article) = draft.content.news_item.first() {
                    println!("     Title: {}", article.title);
                }
            }
        }
        Err(e) => println!("❌ Failed to list drafts: {e}"),
    }

    // Example 6: Error handling demonstration
    println!("\n📝 Example 6: Error handling demonstration");

    // Try to upload a non-existent file
    match client.upload("non_existent_file.md").await {
        Ok(_) => println!("This shouldn't happen"),
        Err(WeChatError::FileNotFound { path }) => {
            println!("✅ Correctly caught file not found error: {path}");
        }
        Err(e) => println!("❌ Unexpected error: {e}"),
    }

    // Try to use a non-existent theme
    let bad_options = UploadOptions::with_theme("non_existent_theme");
    if tokio::fs::metadata("fixtures/example.md").await.is_ok() {
        match client
            .upload_with_options("fixtures/example.md", bad_options)
            .await
        {
            Ok(_) => println!("This shouldn't happen"),
            Err(WeChatError::ThemeNotFound { theme }) => {
                println!("✅ Correctly caught theme not found error: {theme}");
            }
            Err(e) => println!("❌ Unexpected error: {e}"),
        }
    }

    println!("\n🎉 All examples completed!");
    println!("\n💡 Tips:");
    println!("  - Use RUST_LOG=debug for detailed logging");
    println!("  - Check the fixtures/ directory for example files");
    println!("  - Ensure your WeChat credentials have proper permissions");
    println!("  - The SDK automatically handles token refresh and image deduplication");

    Ok(())
}
