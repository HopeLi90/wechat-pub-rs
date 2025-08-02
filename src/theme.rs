//! Theme system for rendering markdown content to HTML.

use crate::error::{Result, WeChatError};
use pulldown_cmark::{html, Options, Parser};
use std::collections::HashMap;
use std::path::Path;

/// Built-in theme options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinTheme {
    /// Simple, clean default theme
    Default,
    /// Lapis theme with blue accents
    Lapis,
    /// Maize theme with yellow tones
    Maize,
    /// Orange Heart theme with orange accents
    OrangeHeart,
    /// PhyCat theme
    PhyCat,
    /// Pie theme
    Pie,
    /// Purple theme with purple accents
    Purple,
    /// Rainbow theme with colorful elements
    Rainbow,
}

impl BuiltinTheme {
    /// Gets the theme name as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            BuiltinTheme::Default => "default",
            BuiltinTheme::Lapis => "lapis",
            BuiltinTheme::Maize => "maize",
            BuiltinTheme::OrangeHeart => "orangeheart",
            BuiltinTheme::PhyCat => "phycat",
            BuiltinTheme::Pie => "pie",
            BuiltinTheme::Purple => "purple",
            BuiltinTheme::Rainbow => "rainbow",
        }
    }

    /// Gets all available built-in themes.
    pub fn all() -> Vec<BuiltinTheme> {
        vec![
            BuiltinTheme::Default,
            BuiltinTheme::Lapis,
            BuiltinTheme::Maize,
            BuiltinTheme::OrangeHeart,
            BuiltinTheme::PhyCat,
            BuiltinTheme::Pie,
            BuiltinTheme::Purple,
            BuiltinTheme::Rainbow,
        ]
    }
}

impl std::str::FromStr for BuiltinTheme {
    type Err = WeChatError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "default" => Ok(BuiltinTheme::Default),
            "lapis" => Ok(BuiltinTheme::Lapis),
            "maize" => Ok(BuiltinTheme::Maize),
            "orangeheart" => Ok(BuiltinTheme::OrangeHeart),
            "phycat" => Ok(BuiltinTheme::PhyCat),
            "pie" => Ok(BuiltinTheme::Pie),
            "purple" => Ok(BuiltinTheme::Purple),
            "rainbow" => Ok(BuiltinTheme::Rainbow),
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

    /// Loads theme CSS from file.
    pub fn load_theme_from_file(&mut self, theme_name: &str, css_path: &Path) -> Result<()> {
        let css = std::fs::read_to_string(css_path)
            .map_err(|e| WeChatError::file_error(
                css_path.display().to_string(),
                format!("Failed to read theme CSS: {}", e)
            ))?;
        
        let html_template = self.get_default_html_template();
        let template = ThemeTemplate::new(css, html_template, theme_name.to_string());
        self.templates.insert(theme_name.to_string(), template);
        Ok(())
    }

    /// Gets the default HTML template.
    fn get_default_html_template(&self) -> String {
        r#"<!DOCTYPE html>
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
    <section id="wenyan">
        {{CONTENT}}
    </section>
</body>
</html>"#.to_string()
    }

    /// Creates a built-in theme template.
    fn create_builtin_theme(&self, theme: BuiltinTheme) -> ThemeTemplate {
        // For now, return a basic template. In production, these would load from CSS files.
        let css = self.get_theme_css(theme);
        let html_template = self.get_default_html_template();
        ThemeTemplate::new(css, html_template, theme.as_str().to_string())
    }

    /// Gets theme CSS content.
    fn get_theme_css(&self, theme: BuiltinTheme) -> String {
        // This is a placeholder. In production, load from actual CSS files.
        match theme {
            BuiltinTheme::Default => self.get_default_css(),
            _ => self.get_default_css(), // For now, use default for all themes
        }
    }

    /// Gets default theme CSS.
    fn get_default_css(&self) -> String {
        r#"
#wenyan {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    line-height: 1.75;
    font-size: 16px;
    color: #333;
    max-width: 800px;
    margin: 0 auto;
    padding: 20px;
}

#wenyan h1, #wenyan h2, #wenyan h3, #wenyan h4, #wenyan h5, #wenyan h6 {
    color: #2c3e50;
    margin-top: 1.5em;
    margin-bottom: 0.5em;
}

#wenyan h1 {
    font-size: 1.5em;
    text-align: center;
    border-bottom: 2px solid #3498db;
    padding-bottom: 0.3em;
}

#wenyan h2 {
    font-size: 1.3em;
    border-bottom: 1px solid #bdc3c7;
    padding-bottom: 0.2em;
}

#wenyan p {
    margin-bottom: 1em;
    text-align: justify;
}

#wenyan img {
    max-width: 100%;
    height: auto;
    border-radius: 4px;
    box-shadow: 0 2px 8px rgba(0,0,0,0.1);
    margin: 1em 0;
    display: block;
    margin-left: auto;
    margin-right: auto;
}

#wenyan code {
    background-color: #f8f9fa;
    padding: 0.2em 0.4em;
    border-radius: 3px;
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    font-size: 0.9em;
}

#wenyan pre {
    background-color: #f8f9fa;
    padding: 1em;
    border-radius: 5px;
    overflow-x: auto;
    border-left: 4px solid #3498db;
}

#wenyan pre code {
    background-color: transparent;
    padding: 0;
}

#wenyan blockquote {
    border-left: 4px solid #3498db;
    margin: 1em 0;
    padding-left: 1em;
    color: #7f8c8d;
    font-style: italic;
}

#wenyan table {
    border-collapse: collapse;
    width: 100%;
    margin: 1em 0;
}

#wenyan th, #wenyan td {
    border: 1px solid #ddd;
    padding: 0.5em;
    text-align: left;
}

#wenyan th {
    background-color: #f8f9fa;
    font-weight: bold;
}

#wenyan ul, #wenyan ol {
    padding-left: 2em;
    margin: 1em 0;
}

#wenyan li {
    margin: 0.3em 0;
}
"#.to_string()
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
            "lapis".parse::<BuiltinTheme>().unwrap(),
            BuiltinTheme::Lapis
        );
        assert_eq!(
            "maize".parse::<BuiltinTheme>().unwrap(),
            BuiltinTheme::Maize
        );
        assert_eq!(
            "orangeheart".parse::<BuiltinTheme>().unwrap(),
            BuiltinTheme::OrangeHeart
        );
        assert_eq!(
            "phycat".parse::<BuiltinTheme>().unwrap(),
            BuiltinTheme::PhyCat
        );
        assert_eq!(
            "pie".parse::<BuiltinTheme>().unwrap(),
            BuiltinTheme::Pie
        );
        assert_eq!(
            "purple".parse::<BuiltinTheme>().unwrap(),
            BuiltinTheme::Purple
        );
        assert_eq!(
            "rainbow".parse::<BuiltinTheme>().unwrap(),
            BuiltinTheme::Rainbow
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
