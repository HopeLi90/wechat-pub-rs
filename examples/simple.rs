//! Simple example demonstrating how to upload a markdown article to WeChat Official Account.

use wechat_pub_rs::{Result, WeChatClient};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger (optional)
    env_logger::init();

    // Create WeChat client with your app credentials
    // NOTE: Replace with your actual WeChat Official Account credentials
    let app_id = std::env::var("WECHAT_APP_ID").expect("WECHAT_APP_ID is not set"); // Your App ID
    let app_secret = std::env::var("WECHAT_APP_SECRET").expect("WECHAT_APP_SECRET is not set"); // Your App Secret

    let client = WeChatClient::new(app_id, app_secret).await?;
    let example_md_path = "fixtures/example.md";

    // Check if the file exists
    if tokio::fs::metadata(example_md_path).await.is_ok() {
        match client.upload(example_md_path).await {
            Ok(draft_id) => {
                println!("✅ Successfully uploaded {example_md_path} with draft ID: {draft_id}");
            }
            Err(e) => {
                eprintln!("❌ Real file upload failed: {e}");
            }
        }
    } else {
        println!("⚠️  fixtures/example.md not found, skipping real file upload");
    }

    println!("\n=== Available themes ===");
    for theme in client.available_themes() {
        println!("  - {theme}");
    }

    println!("\n=== Token status (for debugging) ===");
    if let Some(token_info) = client.get_token_info().await {
        println!("\nToken info:");
        println!("  - Expires at: {}", token_info.expires_at);
        println!(
            "  - Time until expiry: {} seconds",
            token_info.time_until_expiry.num_seconds()
        );
        println!("  - Is expired: {}", token_info.is_expired);
    } else {
        println!("\nNo token cached yet");
    }

    Ok(())
}
