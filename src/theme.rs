//! Theme system for rendering markdown content to HTML.

use crate::error::{Result, WeChatError};
use pulldown_cmark::{html, Options, Parser};
use std::collections::HashMap;

/// Built-in theme options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinTheme {
    /// Simple, clean default theme
    Default,
    /// GitHub-style theme
    Github,
    /// WeChat native style theme
    Wechat,
    /// Minimalist theme
    Minimal,
}

impl BuiltinTheme {
    /// Gets the theme name as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            BuiltinTheme::Default => "default",
            BuiltinTheme::Github => "github",
            BuiltinTheme::Wechat => "wechat",
            BuiltinTheme::Minimal => "minimal",
        }
    }

    /// Gets all available built-in themes.
    pub fn all() -> Vec<BuiltinTheme> {
        vec![
            BuiltinTheme::Default,
            BuiltinTheme::Github,
            BuiltinTheme::Wechat,
            BuiltinTheme::Minimal,
        ]
    }
}

impl std::str::FromStr for BuiltinTheme {
    type Err = WeChatError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "default" => Ok(BuiltinTheme::Default),
            "github" => Ok(BuiltinTheme::Github),
            "wechat" => Ok(BuiltinTheme::Wechat),
            "minimal" => Ok(BuiltinTheme::Minimal),
            _ => Err(WeChatError::ThemeNotFound {
                theme: s.to_string(),
            }),
        }
    }
}

/// Theme template containing CSS and HTML structure.
#[derive(Debug, Clone)]
pub struct ThemeTemplate {
    /// CSS styles for the theme
    pub css: String,
    /// HTML template (uses basic string replacement)
    pub html_template: String,
    /// Code highlighting theme
    pub code_theme: String,
}

impl ThemeTemplate {
    /// Creates a new theme template.
    pub fn new(css: String, html_template: String, code_theme: String) -> Self {
        Self {
            css,
            html_template,
            code_theme,
        }
    }

    /// Renders content using this theme.
    pub fn render(&self, content: &str, metadata: &HashMap<String, String>) -> Result<String> {
        // Replace placeholders in template
        let mut html = self.html_template.clone();

        // Replace basic placeholders
        html = html.replace("{{CSS}}", &self.css);
        html = html.replace("{{CONTENT}}", content);

        // Replace metadata placeholders
        for (key, value) in metadata {
            let placeholder = format!("{{{{{}}}}}", key.to_uppercase());
            html = html.replace(&placeholder, value);
        }

        // Set default values for common placeholders
        html = html.replace(
            "{{TITLE}}",
            metadata.get("title").unwrap_or(&"Untitled".to_string()),
        );
        html = html.replace(
            "{{AUTHOR}}",
            metadata.get("author").unwrap_or(&"Anonymous".to_string()),
        );

        Ok(html)
    }
}

/// Theme manager for rendering markdown with different styles.
#[derive(Debug)]
pub struct ThemeManager {
    templates: HashMap<String, ThemeTemplate>,
    markdown_options: Options,
}

impl ThemeManager {
    /// Creates a new theme manager with built-in themes.
    pub fn new() -> Self {
        let mut manager = Self {
            templates: HashMap::new(),
            markdown_options: Self::create_markdown_options(),
        };

        manager.load_builtin_themes();
        manager
    }

    /// Creates markdown parsing options.
    fn create_markdown_options() -> Options {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);
        options
    }

    /// Loads all built-in themes.
    fn load_builtin_themes(&mut self) {
        for theme in BuiltinTheme::all() {
            let template = self.create_builtin_theme(theme);
            self.templates.insert(theme.as_str().to_string(), template);
        }
    }

    /// Creates a built-in theme template.
    fn create_builtin_theme(&self, theme: BuiltinTheme) -> ThemeTemplate {
        match theme {
            BuiltinTheme::Default => self.create_default_theme(),
            BuiltinTheme::Github => self.create_github_theme(),
            BuiltinTheme::Wechat => self.create_wechat_theme(),
            BuiltinTheme::Minimal => self.create_minimal_theme(),
        }
    }

    /// Creates the default theme.
    fn create_default_theme(&self) -> ThemeTemplate {
        let css = r#"
body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    line-height: 1.6;
    color: #333;
    max-width: 800px;
    margin: 0 auto;
    padding: 20px;
    background-color: #fff;
}

h1, h2, h3, h4, h5, h6 {
    color: #2c3e50;
    margin-top: 1.5em;
    margin-bottom: 0.5em;
}

h1 {
    font-size: 2.2em;
    border-bottom: 2px solid #3498db;
    padding-bottom: 0.3em;
}

h2 {
    font-size: 1.8em;
    border-bottom: 1px solid #bdc3c7;
    padding-bottom: 0.2em;
}

h3 {
    font-size: 1.4em;
    color: #34495e;
}

p {
    margin-bottom: 1em;
    text-align: justify;
}

img {
    max-width: 100%;
    height: auto;
    border-radius: 4px;
    box-shadow: 0 2px 8px rgba(0,0,0,0.1);
    margin: 1em 0;
}

code {
    background-color: #f8f9fa;
    padding: 0.2em 0.4em;
    border-radius: 3px;
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    font-size: 0.9em;
}

pre {
    background-color: #f8f9fa;
    padding: 1em;
    border-radius: 5px;
    overflow-x: auto;
    border-left: 4px solid #3498db;
}

pre code {
    background-color: transparent;
    padding: 0;
}

blockquote {
    border-left: 4px solid #3498db;
    margin: 1em 0;
    padding-left: 1em;
    color: #7f8c8d;
    font-style: italic;
}

table {
    border-collapse: collapse;
    width: 100%;
    margin: 1em 0;
}

th, td {
    border: 1px solid #ddd;
    padding: 0.5em;
    text-align: left;
}

th {
    background-color: #f8f9fa;
    font-weight: bold;
}

ul, ol {
    padding-left: 2em;
    margin: 1em 0;
}

li {
    margin: 0.3em 0;
}

.author {
    text-align: right;
    color: #7f8c8d;
    font-style: italic;
    margin-top: 2em;
    border-top: 1px solid #ecf0f1;
    padding-top: 1em;
}
"#
        .to_string();

        let html_template = r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{TITLE}}</title>
    <style>
        {{CSS}}
    </style>
</head>
<body>
    {{CONTENT}}
    <div class="author">{{AUTHOR}}</div>
</body>
</html>"#
            .to_string();

        ThemeTemplate::new(css, html_template, "github".to_string())
    }

    /// Creates the GitHub theme.
    fn create_github_theme(&self) -> ThemeTemplate {
        let css = r#"
body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Helvetica Neue', Helvetica, sans-serif;
    line-height: 1.6;
    color: #24292f;
    max-width: 980px;
    margin: 0 auto;
    padding: 45px;
    background-color: #ffffff;
}

h1, h2, h3, h4, h5, h6 {
    margin-top: 24px;
    margin-bottom: 16px;
    font-weight: 600;
    line-height: 1.25;
}

h1 {
    font-size: 2em;
    border-bottom: 1px solid #d0d7de;
    padding-bottom: 0.3em;
}

h2 {
    font-size: 1.5em;
    border-bottom: 1px solid #d0d7de;
    padding-bottom: 0.3em;
}

p {
    margin-top: 0;
    margin-bottom: 16px;
}

img {
    max-width: 100%;
    box-sizing: content-box;
    background-color: #ffffff;
}

code {
    padding: 0.2em 0.4em;
    margin: 0;
    font-size: 85%;
    background-color: rgba(175,184,193,0.2);
    border-radius: 6px;
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Consolas, "Liberation Mono", Menlo, monospace;
}

pre {
    padding: 16px;
    overflow: auto;
    font-size: 85%;
    line-height: 1.45;
    background-color: #f6f8fa;
    border-radius: 6px;
}

pre code {
    display: inline;
    max-width: auto;
    padding: 0;
    margin: 0;
    overflow: visible;
    line-height: inherit;
    word-wrap: normal;
    background-color: transparent;
    border: 0;
}

blockquote {
    padding: 0 1em;
    color: #656d76;
    border-left: 0.25em solid #d0d7de;
    margin: 0 0 16px 0;
}

table {
    border-spacing: 0;
    border-collapse: collapse;
    display: block;
    width: max-content;
    max-width: 100%;
    overflow: auto;
}

th, td {
    padding: 6px 13px;
    border: 1px solid #d0d7de;
}

th {
    font-weight: 600;
    background-color: #f6f8fa;
}

tr:nth-child(2n) {
    background-color: #f6f8fa;
}
"#.to_string();

        let html_template = r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{TITLE}}</title>
    <style>
        {{CSS}}
    </style>
</head>
<body>
    {{CONTENT}}
</body>
</html>"#
            .to_string();

        ThemeTemplate::new(css, html_template, "github".to_string())
    }

    /// Creates the WeChat theme.
    fn create_wechat_theme(&self) -> ThemeTemplate {
        let css = r#"
body {
    font-family: -apple-system-font, BlinkMacSystemFont, "Helvetica Neue", "PingFang SC", "Hiragino Sans GB", "Microsoft YaHei UI", "Microsoft YaHei", Arial, sans-serif;
    line-height: 1.75;
    color: #3f3f3f;
    background-color: #fff;
    padding: 20px;
    max-width: 677px;
    margin: 0 auto;
    word-wrap: break-word;
}

h1, h2, h3, h4, h5, h6 {
    color: #2f2f2f;
    margin-top: 1.2em;
    margin-bottom: 0.6em;
    line-height: 1.35;
    font-weight: bold;
}

h1 {
    font-size: 1.4em;
    text-align: center;
    color: #2f2f2f;
    margin: 1.5em 0 1em 0;
}

h2 {
    font-size: 1.2em;
    border-bottom: 2px solid #00c4b6;
    padding-bottom: 0.2em;
    margin: 1.3em 0 0.8em 0;
}

h3 {
    font-size: 1.1em;
    color: #00c4b6;
}

p {
    margin: 1em 8px;
    text-align: justify;
    text-justify: inter-ideograph;
}

img {
    max-width: 100%;
    border-radius: 4px;
    margin: 1em 0;
    display: block;
    margin-left: auto;
    margin-right: auto;
}

code {
    font-size: 90%;
    color: #d14;
    background: rgba(27,31,35,.05);
    padding: 3px 5px;
    border-radius: 4px;
    margin: 0 2px;
    font-family: Consolas, "Liberation Mono", Menlo, Courier, monospace;
}

pre {
    background: #f6f6f6;
    border-radius: 8px;
    padding: 1em;
    overflow-x: auto;
    margin: 1.2em 8px;
    font-size: 14px;
    color: #383a42;
}

pre code {
    color: inherit;
    background: transparent;
    padding: 0;
    margin: 0;
}

blockquote {
    color: #666;
    padding: 1px 23px;
    margin: 22px 0;
    border-left: 4px solid #00c4b6;
    background-color: #f8f8f8;
}

ul, ol {
    margin: 1em 0;
    padding-left: 2em;
}

li {
    margin: 0.5em 0;
}

table {
    width: 100%;
    border-collapse: collapse;
    margin: 1em 0;
    font-size: 14px;
}

th, td {
    border: 1px solid #dfe2e5;
    padding: 8px 12px;
}

th {
    background-color: #f6f8fa;
    font-weight: bold;
}

.author {
    text-align: right;
    color: #888;
    font-size: 0.9em;
    margin-top: 2em;
    padding-top: 1em;
    border-top: 1px solid #eee;
}
"#.to_string();

        let html_template = r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{TITLE}}</title>
    <style>
        {{CSS}}
    </style>
</head>
<body>
    {{CONTENT}}
    <div class="author">{{AUTHOR}}</div>
</body>
</html>"#
            .to_string();

        ThemeTemplate::new(css, html_template, "wechat".to_string())
    }

    /// Creates the minimal theme.
    fn create_minimal_theme(&self) -> ThemeTemplate {
        let css = r#"
body {
    font-family: Georgia, serif;
    line-height: 1.8;
    color: #333;
    max-width: 600px;
    margin: 0 auto;
    padding: 40px 20px;
    background-color: #fff;
}

h1, h2, h3, h4, h5, h6 {
    color: #222;
    margin-top: 2em;
    margin-bottom: 1em;
    font-weight: normal;
}

h1 {
    font-size: 1.8em;
    text-align: center;
    margin-bottom: 2em;
}

h2 {
    font-size: 1.3em;
}

h3 {
    font-size: 1.1em;
}

p {
    margin-bottom: 1.2em;
    text-align: justify;
}

img {
    max-width: 100%;
    height: auto;
    margin: 2em 0;
    display: block;
    margin-left: auto;
    margin-right: auto;
}

code {
    font-family: "Courier New", monospace;
    background-color: #f5f5f5;
    padding: 2px 4px;
    border-radius: 2px;
}

pre {
    background-color: #f5f5f5;
    padding: 1em;
    overflow-x: auto;
    border-radius: 2px;
    font-family: "Courier New", monospace;
}

blockquote {
    border-left: 3px solid #ccc;
    margin: 1.5em 0;
    padding-left: 1.5em;
    color: #666;
    font-style: italic;
}

ul, ol {
    margin: 1.2em 0;
    padding-left: 2em;
}

li {
    margin: 0.5em 0;
}

.author {
    text-align: center;
    color: #888;
    font-style: italic;
    margin-top: 3em;
    font-size: 0.9em;
}
"#
        .to_string();

        let html_template = r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{TITLE}}</title>
    <style>
        {{CSS}}
    </style>
</head>
<body>
    {{CONTENT}}
    <div class="author">{{AUTHOR}}</div>
</body>
</html>"#
            .to_string();

        ThemeTemplate::new(css, html_template, "minimal".to_string())
    }

    /// Renders markdown content with the specified theme.
    pub fn render(
        &self,
        markdown_content: &str,
        theme_name: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<String> {
        let template =
            self.templates
                .get(theme_name)
                .ok_or_else(|| WeChatError::ThemeNotFound {
                    theme: theme_name.to_string(),
                })?;

        // Convert markdown to HTML
        let parser = Parser::new_ext(markdown_content, self.markdown_options);
        let mut html_content = String::new();
        html::push_html(&mut html_content, parser);

        // Apply theme template
        template.render(&html_content, metadata)
    }

    /// Adds a custom theme.
    pub fn add_theme(&mut self, name: String, template: ThemeTemplate) {
        self.templates.insert(name, template);
    }

    /// Gets the list of available theme names.
    pub fn available_themes(&self) -> Vec<&String> {
        self.templates.keys().collect()
    }

    /// Checks if a theme exists.
    pub fn has_theme(&self, name: &str) -> bool {
        self.templates.contains_key(name)
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_theme_parsing() {
        assert_eq!(
            "default".parse::<BuiltinTheme>().unwrap(),
            BuiltinTheme::Default
        );
        assert_eq!(
            "github".parse::<BuiltinTheme>().unwrap(),
            BuiltinTheme::Github
        );
        assert_eq!(
            "wechat".parse::<BuiltinTheme>().unwrap(),
            BuiltinTheme::Wechat
        );
        assert_eq!(
            "minimal".parse::<BuiltinTheme>().unwrap(),
            BuiltinTheme::Minimal
        );

        assert!("nonexistent".parse::<BuiltinTheme>().is_err());
    }

    #[test]
    fn test_theme_manager_creation() {
        let manager = ThemeManager::new();

        // Should have all built-in themes
        for theme in BuiltinTheme::all() {
            assert!(manager.has_theme(theme.as_str()));
        }

        let themes = manager.available_themes();
        assert!(themes.len() >= 4);
    }

    #[test]
    fn test_theme_rendering() {
        let manager = ThemeManager::new();
        let markdown = "# Test Title\n\nThis is a test paragraph with **bold** text.";

        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "Test Article".to_string());
        metadata.insert("author".to_string(), "Test Author".to_string());

        let result = manager.render(markdown, "default", &metadata);
        assert!(result.is_ok());

        let html = result.unwrap();
        assert!(html.contains("<h1>Test Title</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("Test Article"));
        assert!(html.contains("Test Author"));
    }

    #[test]
    fn test_nonexistent_theme() {
        let manager = ThemeManager::new();
        let result = manager.render("# Test", "nonexistent", &HashMap::new());

        assert!(result.is_err());
        if let Err(WeChatError::ThemeNotFound { theme }) = result {
            assert_eq!(theme, "nonexistent");
        } else {
            panic!("Expected ThemeNotFound error");
        }
    }

    #[test]
    fn test_custom_theme() {
        let mut manager = ThemeManager::new();

        let custom_template = ThemeTemplate::new(
            "body { color: red; }".to_string(),
            "<html><head><style>{{CSS}}</style></head><body>{{CONTENT}}</body></html>".to_string(),
            "custom".to_string(),
        );

        manager.add_theme("custom".to_string(), custom_template);
        assert!(manager.has_theme("custom"));

        let result = manager.render("# Test", "custom", &HashMap::new());
        assert!(result.is_ok());

        let html = result.unwrap();
        assert!(html.contains("color: red"));
        assert!(html.contains("<h1>Test</h1>"));
    }

    #[test]
    fn test_template_placeholder_replacement() {
        let template = ThemeTemplate::new(
            "/* CSS */".to_string(),
            "<!DOCTYPE html><html><head><title>{{TITLE}}</title></head><body>{{CONTENT}}<div>{{AUTHOR}}</div></body></html>".to_string(),
            "theme".to_string(),
        );

        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "My Title".to_string());
        metadata.insert("author".to_string(), "John Doe".to_string());

        let result = template.render("<h1>Content</h1>", &metadata);
        assert!(result.is_ok());

        let html = result.unwrap();
        assert!(html.contains("<title>My Title</title>"));
        assert!(html.contains("<h1>Content</h1>"));
        assert!(html.contains("<div>John Doe</div>"));
    }
}
