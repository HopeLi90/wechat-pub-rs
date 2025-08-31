use std::path::Path;
use wechat_pub_rs::mermaid::MermaidProcessor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test with fixtures/example.md
    let markdown_path = Path::new("fixtures/example.md");
    let content = tokio::fs::read_to_string(markdown_path).await?;

    println!("Reading markdown file: {}", markdown_path.display());
    println!("Content length: {} characters", content.len());

    // Check for Mermaid blocks
    let charts = MermaidProcessor::detect_mermaid_blocks(&content);
    println!("Found {} Mermaid charts", charts.len());

    if !charts.is_empty() {
        // Process the content
        let base_dir = Path::new("fixtures");
        let document_slug = "example";
        let processor = MermaidProcessor::new(base_dir.to_path_buf(), document_slug.to_string());

        match processor
            .process_mermaid_content_with_source_path(&content, base_dir, Some(markdown_path))
            .await
        {
            Ok((modified_content, images)) => {
                println!("Successfully processed Mermaid charts!");
                println!("Generated {} images", images.len());
                for (i, img) in images.iter().enumerate() {
                    println!("  Image {}: {}", i + 1, img.original_url);
                }

                // Save the modified content for inspection
                let output_path = Path::new("fixtures/example_processed.md");
                tokio::fs::write(output_path, &modified_content).await?;
                println!("Saved processed content to: {}", output_path.display());

                // Show a snippet of the modified content where Mermaid was replaced
                if let Some(pos) = modified_content.find("![Mermaid Chart") {
                    let end = (pos + 100).min(modified_content.len());
                    println!("\nSnippet of modified content:");
                    println!("{}", &modified_content[pos..end]);
                }
            }
            Err(e) => {
                eprintln!("Error processing Mermaid charts: {}", e);
            }
        }
    } else {
        println!("No Mermaid charts found in the file");
    }

    Ok(())
}
