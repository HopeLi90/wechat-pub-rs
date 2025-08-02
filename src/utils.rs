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
