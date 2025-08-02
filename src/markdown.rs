//! Markdown parsing and image extraction module.

use crate::error::{Result, WeChatError};
use comrak::{nodes::NodeValue, Arena, ComrakOptions};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Represents an image reference found in markdown content.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageRef {
    /// Alt text for the image
    pub alt_text: String,
    /// Original URL or file path
    pub original_url: String,
    /// Position in the markdown text (start, end)
    pub position: (usize, usize),
    /// Whether this is a local file or remote URL
    pub is_local: bool,
}

impl ImageRef {
    /// Creates a new image reference.
    pub fn new(alt_text: String, url: String, position: (usize, usize)) -> Self {
        let is_local = !url.starts_with("http://") && !url.starts_with("https://");
        Self {
            alt_text,
            original_url: url,
            position,
            is_local,
        }
    }

    /// Resolves the image path relative to a base directory.
    pub fn resolve_path(&self, base_path: &Path) -> PathBuf {
        if self.is_local {
            base_path.join(&self.original_url)
        } else {
            PathBuf::from(&self.original_url)
        }
    }
}

/// Parsed markdown content with metadata and image references.
#[derive(Debug, Clone)]
pub struct MarkdownContent {
    /// Article title (extracted from # heading or front matter)
    pub title: Option<String>,
    /// Author name (from front matter)
    pub author: Option<String>,
    /// Cover image path (from front matter)
    pub cover: Option<String>,
    /// Theme name (from front matter)
    pub theme: Option<String>,
    /// Code syntax highlighting theme (from front matter)
    pub code: Option<String>,
    /// Main content (markdown text)
    pub content: String,
    /// List of image references
    pub images: Vec<ImageRef>,
    /// Additional metadata from front matter
    pub metadata: HashMap<String, String>,
    /// The original markdown text
    pub original_text: String,
}

impl MarkdownContent {
    /// Replaces image URLs in the content with new URLs.
    pub fn replace_image_urls(&mut self, url_mapping: &HashMap<String, String>) -> Result<()> {
        let mut content = self.content.clone();

        // Sort images by position in reverse order to avoid position shifting
        let mut sorted_images = self.images.clone();
        sorted_images.sort_by(|a, b| b.position.0.cmp(&a.position.0));

        for image in &sorted_images {
            if let Some(new_url) = url_mapping.get(&image.original_url) {
                // Find and replace the image URL in markdown
                let old_markdown = format!("![{}]({})", image.alt_text, image.original_url);
                let new_markdown = format!("![{}]({})", image.alt_text, new_url);

                content = content.replace(&old_markdown, &new_markdown);
            }
        }

        self.content = content;
        Ok(())
    }

    /// Gets a summary of the content (first paragraph or up to 200 characters).
    pub fn get_summary(&self, max_length: usize) -> String {
        let arena = Arena::new();
        let options = ComrakOptions::default();
        let root = comrak::parse_document(&arena, &self.content, &options);
        let mut summary = String::new();
        let mut text_length = 0;
        let mut found_paragraph = false;

        fn collect_text<'a>(
            node: &'a comrak::nodes::AstNode<'a>,
            summary: &mut String,
            max_length: usize,
            text_length: &mut usize,
            found_paragraph: &mut bool,
        ) -> bool {
            match &node.data.borrow().value {
                NodeValue::Paragraph => {
                    if *found_paragraph {
                        // We already found a paragraph, stop here
                        return true;
                    }
                    *found_paragraph = true;
                    for child in node.children() {
                        if collect_text(child, summary, max_length, text_length, found_paragraph) {
                            break;
                        }
                        if *text_length >= max_length {
                            break;
                        }
                    }
                    if !summary.is_empty() {
                        return true; // Found first paragraph content, stop processing
                    }
                    *found_paragraph = false;
                }
                NodeValue::Text(text) => {
                    if *found_paragraph && *text_length < max_length {
                        let remaining = max_length - *text_length;
                        if text.len() <= remaining {
                            summary.push_str(text);
                            *text_length += text.len();
                        } else {
                            summary.push_str(&text[..remaining]);
                            summary.push_str("...");
                            *text_length = max_length;
                        }
                    }
                }
                _ => {
                    for child in node.children() {
                        if collect_text(child, summary, max_length, text_length, found_paragraph) {
                            return true;
                        }
                        if *text_length >= max_length {
                            break;
                        }
                    }
                }
            }
            false
        }

        collect_text(
            root,
            &mut summary,
            max_length,
            &mut text_length,
            &mut found_paragraph,
        );

        if summary.is_empty() {
            // Fallback: take first characters of content
            let content_text = self.extract_plain_text();
            if content_text.len() > max_length {
                format!("{}...", &content_text[..max_length])
            } else {
                content_text
            }
        } else {
            summary
        }
    }

    /// Extracts plain text from markdown content.
    pub fn extract_plain_text(&self) -> String {
        let arena = Arena::new();
        let options = ComrakOptions::default();
        let root = comrak::parse_document(&arena, &self.content, &options);
        let mut text = String::new();

        fn collect_text<'a>(node: &'a comrak::nodes::AstNode<'a>, text: &mut String) {
            match &node.data.borrow().value {
                NodeValue::Text(content) => {
                    text.push_str(content);
                    text.push(' ');
                }
                _ => {
                    for child in node.children() {
                        collect_text(child, text);
                    }
                }
            }
        }

        collect_text(root, &mut text);
        text.trim().to_string()
    }
}

/// Markdown parser with image extraction capabilities.
#[derive(Debug)]
pub struct MarkdownParser {
    options: ComrakOptions<'static>,
}

impl MarkdownParser {
    /// Creates a new markdown parser with default options.
    pub fn new() -> Self {
        let mut options = ComrakOptions::<'static>::default();
        options.extension.strikethrough = true;
        options.extension.table = true;
        options.extension.footnotes = true;
        options.extension.tasklist = true;
        options.parse.smart = true;

        Self { options }
    }

    /// Parses markdown content from a string.
    pub fn parse(&self, markdown: &str) -> Result<MarkdownContent> {
        let (metadata, content_without_frontmatter) = self.extract_frontmatter(markdown)?;
        let title = self.extract_title(&content_without_frontmatter, &metadata);
        let author = metadata.get("author").cloned();
        let cover = metadata.get("cover").cloned();
        let theme = metadata.get("theme").cloned();
        let code = metadata.get("code").cloned();
        let images = self.extract_images(&content_without_frontmatter)?;

        Ok(MarkdownContent {
            title,
            author,
            cover,
            theme,
            code,
            content: content_without_frontmatter,
            images,
            metadata,
            original_text: markdown.to_string(),
        })
    }

    /// Parses markdown content from a file.
    pub async fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<MarkdownContent> {
        let content = tokio::fs::read_to_string(&path).await.map_err(|e| {
            WeChatError::file_error(
                path.as_ref().display().to_string(),
                format!("Failed to read file: {e}"),
            )
        })?;

        self.parse(&content)
    }

    /// Extracts front matter (YAML) from markdown content.
    fn extract_frontmatter(&self, markdown: &str) -> Result<(HashMap<String, String>, String)> {
        let mut metadata = HashMap::new();
        let content = if let Some(stripped) = markdown.strip_prefix("---\n") {
            // Find the end of front matter
            if let Some(end_pos) = stripped.find("\n---\n") {
                let frontmatter = &stripped[..end_pos];
                let content = &stripped[end_pos + 5..]; // skip "\n---\n"

                // Parse YAML-like front matter (simple key: value pairs)
                for line in frontmatter.lines() {
                    if let Some((key, value)) = line.split_once(':') {
                        let key = key.trim().to_string();
                        let value = value.trim().trim_matches('"').to_string();
                        metadata.insert(key, value);
                    }
                }

                content.to_string()
            } else {
                markdown.to_string()
            }
        } else {
            markdown.to_string()
        };

        Ok((metadata, content))
    }

    /// Extracts the title from content or metadata.
    fn extract_title(&self, content: &str, metadata: &HashMap<String, String>) -> Option<String> {
        // Check metadata first
        if let Some(title) = metadata.get("title") {
            return Some(title.clone());
        }

        // Look for first # heading
        let arena = Arena::new();
        let root = comrak::parse_document(&arena, content, &self.options);

        fn find_h1_title<'a>(node: &'a comrak::nodes::AstNode<'a>) -> Option<String> {
            match &node.data.borrow().value {
                NodeValue::Heading(heading) if heading.level == 1 => {
                    let mut title = String::new();
                    fn collect_heading_text<'a>(
                        node: &'a comrak::nodes::AstNode<'a>,
                        title: &mut String,
                    ) {
                        match &node.data.borrow().value {
                            NodeValue::Text(text) => title.push_str(text),
                            _ => {
                                for child in node.children() {
                                    collect_heading_text(child, title);
                                }
                            }
                        }
                    }

                    for child in node.children() {
                        collect_heading_text(child, &mut title);
                    }

                    if !title.trim().is_empty() {
                        Some(title.trim().to_string())
                    } else {
                        None
                    }
                }
                _ => {
                    for child in node.children() {
                        if let Some(title) = find_h1_title(child) {
                            return Some(title);
                        }
                    }
                    None
                }
            }
        }

        find_h1_title(root)
    }

    /// Extracts image references from markdown content.
    fn extract_images(&self, content: &str) -> Result<Vec<ImageRef>> {
        let arena = Arena::new();
        let root = comrak::parse_document(&arena, content, &self.options);
        let mut images = Vec::new();

        fn extract_images_recursive<'a>(
            node: &'a comrak::nodes::AstNode<'a>,
            images: &mut Vec<ImageRef>,
            source: &str,
        ) {
            match &node.data.borrow().value {
                NodeValue::Image(link) => {
                    let mut alt_text = String::new();

                    // Collect alt text from children
                    fn collect_alt_text<'a>(
                        node: &'a comrak::nodes::AstNode<'a>,
                        alt: &mut String,
                    ) {
                        match &node.data.borrow().value {
                            NodeValue::Text(text) => alt.push_str(text),
                            NodeValue::Code(code) => alt.push_str(&code.literal),
                            _ => {
                                for child in node.children() {
                                    collect_alt_text(child, alt);
                                }
                            }
                        }
                    }

                    for child in node.children() {
                        collect_alt_text(child, &mut alt_text);
                    }

                    let url = link.url.clone();

                    // Calculate approximate position based on content search
                    let position =
                        if let Some(start) = source.find(&format!("![{alt_text}]({url})")) {
                            let end = start + format!("![{alt_text}]({url})").len();
                            (start, end)
                        } else {
                            (0, 0) // Fallback if exact match not found
                        };

                    let image_ref = ImageRef::new(alt_text, url, position);
                    images.push(image_ref);
                }
                _ => {
                    for child in node.children() {
                        extract_images_recursive(child, images, source);
                    }
                }
            }
        }

        extract_images_recursive(root, &mut images, content);
        Ok(images)
    }
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_image_ref_creation() {
        let img = ImageRef::new("Alt text".to_string(), "image.jpg".to_string(), (10, 20));

        assert_eq!(img.alt_text, "Alt text");
        assert_eq!(img.original_url, "image.jpg");
        assert_eq!(img.position, (10, 20));
        assert!(img.is_local);

        let remote_img = ImageRef::new(
            "Remote".to_string(),
            "https://example.com/image.jpg".to_string(),
            (0, 10),
        );
        assert!(!remote_img.is_local);
    }

    #[test]
    fn test_frontmatter_extraction() {
        let parser = MarkdownParser::new();
        let markdown = r#"---
title: Test Article
author: John Doe
date: 2024-01-01
---

# Content

This is the content."#;

        let (metadata, content) = parser.extract_frontmatter(markdown).unwrap();

        assert_eq!(metadata.get("title"), Some(&"Test Article".to_string()));
        assert_eq!(metadata.get("author"), Some(&"John Doe".to_string()));
        assert_eq!(metadata.get("date"), Some(&"2024-01-01".to_string()));
        assert!(content.contains("# Content"));
    }

    #[test]
    fn test_title_extraction() {
        let parser = MarkdownParser::new();

        // From metadata
        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "Metadata Title".to_string());
        let title = parser.extract_title("# Heading Title", &metadata);
        assert_eq!(title, Some("Metadata Title".to_string()));

        // From heading
        let empty_metadata = HashMap::new();
        let title = parser.extract_title("# Heading Title\n\nContent", &empty_metadata);
        assert_eq!(title, Some("Heading Title".to_string()));

        // No title
        let title = parser.extract_title("Just content", &empty_metadata);
        assert_eq!(title, None);
    }

    #[test]
    fn test_image_extraction() {
        let parser = MarkdownParser::new();
        let markdown = r#"# Test

Here's a local image: ![Alt text](./images/local.jpg)

And a remote image: ![Remote](https://example.com/remote.png)

More content here."#;

        let images = parser.extract_images(markdown).unwrap();
        assert_eq!(images.len(), 2);

        let local_img = &images[0];
        assert_eq!(local_img.alt_text, "Alt text");
        assert_eq!(local_img.original_url, "./images/local.jpg");
        assert!(local_img.is_local);

        let remote_img = &images[1];
        assert_eq!(remote_img.alt_text, "Remote");
        assert_eq!(remote_img.original_url, "https://example.com/remote.png");
        assert!(!remote_img.is_local);
    }

    #[tokio::test]
    async fn test_markdown_parsing() {
        let parser = MarkdownParser::new();
        let markdown = r#"---
title: Test Article
author: Jane Doe
cover: images/cover.jpg
---

# Main Heading

This is a test article with an image: ![Test](./test.jpg)

More content here."#;

        let content = parser.parse(markdown).unwrap();

        assert_eq!(content.title, Some("Test Article".to_string()));
        assert_eq!(content.author, Some("Jane Doe".to_string()));
        assert_eq!(content.cover, Some("images/cover.jpg".to_string()));
        assert_eq!(content.theme, None);
        assert_eq!(content.code, None);
        assert_eq!(content.images.len(), 1);
        assert_eq!(content.images[0].alt_text, "Test");
        assert_eq!(content.images[0].original_url, "./test.jpg");
    }

    #[test]
    fn test_url_replacement() {
        let parser = MarkdownParser::new();
        let markdown = "![Alt](./local.jpg) and ![Remote](https://example.com/remote.png)";

        let mut content = parser.parse(markdown).unwrap();
        let mut url_mapping = HashMap::new();
        url_mapping.insert(
            "./local.jpg".to_string(),
            "https://wechat.com/123".to_string(),
        );

        content.replace_image_urls(&url_mapping).unwrap();

        assert!(content.content.contains("https://wechat.com/123"));
        assert!(content.content.contains("https://example.com/remote.png"));
    }

    #[test]
    fn test_summary_extraction() {
        let parser = MarkdownParser::new();
        let markdown = r#"# Title

This is the first paragraph with some content.

This is the second paragraph.
"#;

        let content = parser.parse(markdown).unwrap();
        let summary = content.get_summary(100);

        assert!(summary.contains("This is the first paragraph"));
        assert!(!summary.contains("This is the second paragraph"));
    }

    #[tokio::test]
    async fn test_file_parsing() {
        let parser = MarkdownParser::new();

        // Create a temporary file
        let temp_file = NamedTempFile::new().unwrap();
        let markdown_content = r#"# Test File

Content from file with ![image](./test.jpg)
"#;

        tokio::fs::write(temp_file.path(), markdown_content)
            .await
            .unwrap();

        let content = parser.parse_file(temp_file.path()).await.unwrap();

        assert_eq!(content.title, Some("Test File".to_string()));
        assert_eq!(content.images.len(), 1);
        assert_eq!(content.images[0].original_url, "./test.jpg");
    }

    #[test]
    fn test_cover_extraction_from_frontmatter() {
        let parser = MarkdownParser::new();
        let markdown_with_cover = r#"---
title: Test Article
cover: images/cover.jpg
---

# Content"#;

        let content = parser.parse(markdown_with_cover).unwrap();
        assert_eq!(content.cover, Some("images/cover.jpg".to_string()));

        let markdown_without_cover = r#"---
title: Test Article
---

# Content"#;

        let content = parser.parse(markdown_without_cover).unwrap();
        assert_eq!(content.cover, None);
    }

    #[test]
    fn test_theme_extraction_from_frontmatter() {
        let parser = MarkdownParser::new();
        let markdown_with_theme = r#"---
title: Test Article
theme: lapis
---

# Content"#;

        let content = parser.parse(markdown_with_theme).unwrap();
        assert_eq!(content.theme, Some("lapis".to_string()));

        let markdown_without_theme = r#"---
title: Test Article
---

# Content"#;

        let content = parser.parse(markdown_without_theme).unwrap();
        assert_eq!(content.theme, None);
    }

    #[test]
    fn test_code_theme_extraction_from_frontmatter() {
        let parser = MarkdownParser::new();
        let markdown_with_code = r#"---
title: Test Article
code: solarized-light
---

# Content"#;

        let content = parser.parse(markdown_with_code).unwrap();
        assert_eq!(content.code, Some("solarized-light".to_string()));

        let markdown_without_code = r#"---
title: Test Article
---

# Content"#;

        let content = parser.parse(markdown_without_code).unwrap();
        assert_eq!(content.code, None);
    }

    #[test]
    fn test_markdown_parsing_with_all_frontmatter() {
        let parser = MarkdownParser::new();
        let markdown = r#"---
title: Full Example
author: John Doe
cover: assets/cover-image.png
date: 2024-01-01
---

# Main Content

Article content with an image: ![Example](./example.jpg)
"#;

        let content = parser.parse(markdown).unwrap();

        assert_eq!(content.title, Some("Full Example".to_string()));
        assert_eq!(content.author, Some("John Doe".to_string()));
        assert_eq!(content.cover, Some("assets/cover-image.png".to_string()));
        assert_eq!(content.theme, None);
        assert_eq!(content.code, None);
        assert_eq!(
            content.metadata.get("date"),
            Some(&"2024-01-01".to_string())
        );
        assert_eq!(content.images.len(), 1);
        assert_eq!(content.images[0].original_url, "./example.jpg");
    }
}
