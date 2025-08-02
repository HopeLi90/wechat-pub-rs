//! Utility functions and helpers.

use std::path::Path;

/// Checks if a file exists and is readable.
pub async fn file_exists(path: &Path) -> bool {
    tokio::fs::metadata(path).await.is_ok()
}

/// Gets the file extension from a path.
pub fn get_file_extension(path: &Path) -> Option<&str> {
    path.extension().and_then(|ext| ext.to_str())
}

/// Validates that a path points to a markdown file.
pub fn is_markdown_file(path: &Path) -> bool {
    match get_file_extension(path) {
        Some(ext) => matches!(ext.to_lowercase().as_str(), "md" | "markdown"),
        None => false,
    }
}

/// Validates that a path points to an image file.
pub fn is_image_file(path: &Path) -> bool {
    match get_file_extension(path) {
        Some(ext) => matches!(
            ext.to_lowercase().as_str(),
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp"
        ),
        None => false,
    }
}

/// Sanitizes a string for use as a filename.
pub fn sanitize_filename(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

/// Truncates text to a maximum length, adding ellipsis if needed.
pub fn truncate_text(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        text.to_string()
    } else {
        let mut truncated = text.chars().take(max_length).collect::<String>();
        if truncated.len() < text.len() {
            truncated.push_str("...");
        }
        truncated
    }
}

/// Formats file size in human-readable format.
pub fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    const THRESHOLD: u64 = 1024;

    if size == 0 {
        return "0 B".to_string();
    }

    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= THRESHOLD as f64 && unit_index < UNITS.len() - 1 {
        size /= THRESHOLD as f64;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Creates a unique identifier for requests.
pub fn generate_request_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Validates WeChat app credentials format.
pub fn validate_app_credentials(app_id: &str, app_secret: &str) -> Result<(), String> {
    if app_id.is_empty() {
        return Err("App ID cannot be empty".to_string());
    }

    if app_secret.is_empty() {
        return Err("App secret cannot be empty".to_string());
    }

    // WeChat app IDs typically start with "wx" and are 18 characters long
    if !app_id.starts_with("wx") || app_id.len() != 18 {
        return Err(
            "Invalid app ID format (should start with 'wx' and be 18 characters)".to_string(),
        );
    }

    // WeChat app secrets are typically 32 characters long
    if app_secret.len() != 32 {
        return Err("Invalid app secret format (should be 32 characters)".to_string());
    }

    Ok(())
}

/// Extracts the base directory from a file path.
pub fn get_base_directory(file_path: &Path) -> Option<&Path> {
    file_path.parent()
}

/// Resolves relative paths against a base directory.
pub fn resolve_path(base_dir: &Path, relative_path: &str) -> std::path::PathBuf {
    if Path::new(relative_path).is_absolute() {
        Path::new(relative_path).to_path_buf()
    } else {
        base_dir.join(relative_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_get_file_extension() {
        assert_eq!(get_file_extension(Path::new("test.md")), Some("md"));
        assert_eq!(
            get_file_extension(Path::new("test.markdown")),
            Some("markdown")
        );
        assert_eq!(get_file_extension(Path::new("test.jpg")), Some("jpg"));
        assert_eq!(get_file_extension(Path::new("test")), None);
        assert_eq!(get_file_extension(Path::new(".gitignore")), None);
    }

    #[test]
    fn test_is_markdown_file() {
        assert!(is_markdown_file(Path::new("test.md")));
        assert!(is_markdown_file(Path::new("test.markdown")));
        assert!(is_markdown_file(Path::new("TEST.MD")));
        assert!(!is_markdown_file(Path::new("test.txt")));
        assert!(!is_markdown_file(Path::new("test")));
    }

    #[test]
    fn test_is_image_file() {
        assert!(is_image_file(Path::new("test.jpg")));
        assert!(is_image_file(Path::new("test.PNG")));
        assert!(is_image_file(Path::new("test.gif")));
        assert!(!is_image_file(Path::new("test.txt")));
        assert!(!is_image_file(Path::new("test")));
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("hello world"), "hello world");
        assert_eq!(sanitize_filename("hello/world"), "hello_world");
        assert_eq!(sanitize_filename("hello:world*test"), "hello_world_test");
        assert_eq!(sanitize_filename("  hello world  "), "hello world");
    }

    #[test]
    fn test_truncate_text() {
        assert_eq!(truncate_text("hello", 10), "hello");
        assert_eq!(truncate_text("hello world", 5), "hello...");
        assert_eq!(truncate_text("hello world", 11), "hello world");
        assert_eq!(truncate_text("", 5), "");
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn test_generate_request_id() {
        let id1 = generate_request_id();
        let id2 = generate_request_id();

        assert_ne!(id1, id2);
        assert_eq!(id1.len(), 36); // UUID v4 format
        assert!(id1.contains('-'));
    }

    #[test]
    fn test_validate_app_credentials() {
        // Valid credentials
        assert!(
            validate_app_credentials("wx1234567890123456", "12345678901234567890123456789012")
                .is_ok()
        );

        // Invalid app ID
        assert!(validate_app_credentials("", "12345678901234567890123456789012").is_err());
        assert!(validate_app_credentials("invalid", "12345678901234567890123456789012").is_err());
        assert!(validate_app_credentials("wx123", "12345678901234567890123456789012").is_err());

        // Invalid app secret
        assert!(validate_app_credentials("wx1234567890123456", "").is_err());
        assert!(validate_app_credentials("wx1234567890123456", "short").is_err());
    }

    #[test]
    fn test_get_base_directory() {
        assert_eq!(
            get_base_directory(Path::new("/path/to/file.md")),
            Some(Path::new("/path/to"))
        );
        assert_eq!(
            get_base_directory(Path::new("file.md")),
            Some(Path::new(""))
        );
        assert_eq!(get_base_directory(Path::new("/")), None);
    }

    #[test]
    fn test_resolve_path() {
        let base = Path::new("/base/dir");

        assert_eq!(
            resolve_path(base, "relative.md"),
            PathBuf::from("/base/dir/relative.md")
        );
        assert_eq!(
            resolve_path(base, "/absolute.md"),
            PathBuf::from("/absolute.md")
        );
        assert_eq!(
            resolve_path(base, "./relative.md"),
            PathBuf::from("/base/dir/./relative.md")
        );
    }
}
