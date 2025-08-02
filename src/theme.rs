//! Theme system for rendering markdown content to HTML.

use crate::error::{Result, WeChatError};
use askama::Template;
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

/// Askama template for rendering articles with themes.
#[derive(Template)]
#[template(path = "article.html")]
pub struct ArticleTemplate {
    pub title: String,
    pub description: String,
    pub author: String,
    pub content: String,
    pub theme_css: String,
}

/// Theme template containing CSS for styling.
#[derive(Debug, Clone)]
pub struct ThemeTemplate {
    /// CSS styles for the theme
    pub css: String,
    /// Theme name
    pub name: String,
}

impl ThemeTemplate {
    /// Creates a new theme template.
    pub fn new(css: String, name: String) -> Self {
        Self { css, name }
    }

    /// Renders content using this theme with inline styles for WeChat.
    pub fn render(&self, content: &str, metadata: &HashMap<String, String>) -> Result<String> {
        // Create Askama template with the provided content and metadata
        let template = ArticleTemplate {
            title: metadata.get("title").cloned().unwrap_or_default(),
            description: metadata.get("description").cloned().unwrap_or_default(),
            author: metadata.get("author").cloned().unwrap_or_default(),
            content: content.to_string(),
            theme_css: self.css.clone(),
        };

        // Render the template to HTML
        let html_with_css = template.render().map_err(|e| {
            WeChatError::Internal(anyhow::anyhow!("Template rendering failed: {}", e))
        })?;

        // Use css-inline to convert CSS to inline styles
        let inlined_html = css_inline::inline(&html_with_css)
            .map_err(|e| WeChatError::Internal(anyhow::anyhow!("CSS inlining failed: {}", e)))?;

        Ok(inlined_html)
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
        let css = std::fs::read_to_string(css_path).map_err(|e| {
            WeChatError::file_error(
                css_path.display().to_string(),
                format!("Failed to read theme CSS: {e}"),
            )
        })?;

        let template = ThemeTemplate::new(css, theme_name.to_string());
        self.templates.insert(theme_name.to_string(), template);
        Ok(())
    }

    /// Creates a built-in theme template.
    fn create_builtin_theme(&self, theme: BuiltinTheme) -> ThemeTemplate {
        let css = self.load_builtin_theme_css(theme);
        ThemeTemplate::new(css, theme.as_str().to_string())
    }

    /// Loads CSS content for built-in themes from the themes directory.
    fn load_builtin_theme_css(&self, theme: BuiltinTheme) -> String {
        let theme_path = format!("themes/{}.css", theme.as_str());

        if Path::new(&theme_path).exists() {
            std::fs::read_to_string(&theme_path).unwrap()
        } else {
            log::warn!("Could not load theme file: {theme_path}, using default CSS");
            std::fs::read_to_string("themes/default.css").unwrap()
        }
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
        assert_eq!("pie".parse::<BuiltinTheme>().unwrap(), BuiltinTheme::Pie);
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
        assert!(html.contains("<h1"));
        assert!(html.contains("Test Title"));
        assert!(html.contains("<strong"));
        assert!(html.contains("bold"));
        assert!(html.contains("id=\"wenyan\""));
        assert!(html.contains("data-provider=\"WenYan\""));
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

        let custom_template =
            ThemeTemplate::new("#wepub { color: red; }".to_string(), "custom".to_string());

        manager.add_theme("custom".to_string(), custom_template);
        assert!(manager.has_theme("custom"));

        let result = manager.render("# Test", "custom", &HashMap::new());
        assert!(result.is_ok());

        let html = result.unwrap();
        assert!(html.contains("style="));
        assert!(html.contains("Test"));
        assert!(html.contains("id=\"wenyan\""));
    }

    #[test]
    fn test_template_css_inlining() {
        let css = "#wepub h1 { color: red; font-size: 2em; }";
        let template = ThemeTemplate::new(css.to_string(), "theme".to_string());

        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "My Title".to_string());
        metadata.insert("author".to_string(), "John Doe".to_string());

        let result = template.render("<h1>Content</h1>", &metadata);
        assert!(result.is_ok());

        let html = result.unwrap();
        assert!(html.contains("id=\"wenyan\""));
        assert!(html.contains("data-provider=\"WenYan\""));
        assert!(html.contains("<h1"));
        assert!(html.contains("Content"));
        // Check that CSS was applied as inline styles
        assert!(html.contains("style="));
    }
}
