//! Theme system for rendering markdown content to HTML.

use crate::error::{Result, WeChatError};
use askama::Template;
use comrak::{
    markdown_to_html_with_plugins, plugins::syntect::SyntectAdapter, ComrakOptions, ComrakPlugins,
};
use std::collections::HashMap;

// Embed all theme CSS files at compile time
const DEFAULT_CSS: &str = include_str!("../themes/default.css");
const LAPIS_CSS: &str = include_str!("../themes/lapis.css");
const MAIZE_CSS: &str = include_str!("../themes/maize.css");
const ORANGEHEART_CSS: &str = include_str!("../themes/orangeheart.css");
const PHYCAT_CSS: &str = include_str!("../themes/phycat.css");
const PIE_CSS: &str = include_str!("../themes/pie.css");
const PURPLE_CSS: &str = include_str!("../themes/purple.css");
const RAINBOW_CSS: &str = include_str!("../themes/rainbow.css");

// Embed all highlight CSS files at compile time
const ATOM_ONE_DARK_CSS: &str = include_str!("../themes/highlight/atom-one-dark.min.css");
const ATOM_ONE_LIGHT_CSS: &str = include_str!("../themes/highlight/atom-one-light.min.css");
const DRACULA_CSS: &str = include_str!("../themes/highlight/dracula.min.css");
const GITHUB_DARK_CSS: &str = include_str!("../themes/highlight/github-dark.min.css");
const GITHUB_CSS: &str = include_str!("../themes/highlight/github.min.css");
const MONOKAI_CSS: &str = include_str!("../themes/highlight/monokai.min.css");
const SOLARIZED_DARK_CSS: &str = include_str!("../themes/highlight/solarized-dark.min.css");
const SOLARIZED_LIGHT_CSS: &str = include_str!("../themes/highlight/solarized-light.min.css");
const XCODE_CSS: &str = include_str!("../themes/highlight/xcode.min.css");

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
    pub highlight_css: String,
}

/// Theme template containing CSS for styling.
#[derive(Debug, Clone)]
pub struct ThemeTemplate {
    /// CSS styles for the theme
    pub theme_css: String,
    /// CSS styles for the highlight theme
    pub code_css: String,
    /// Theme name
    pub name: String,
}

impl ThemeTemplate {
    /// Creates a new theme template.
    pub fn new(theme_css: String, code_css: String, name: String) -> Self {
        Self {
            theme_css,
            code_css,
            name,
        }
    }

    /// Creates a new theme template with static CSS references.
    pub fn from_static(theme_css: &'static str, code_css: &'static str, name: String) -> Self {
        Self {
            theme_css: theme_css.to_string(),
            code_css: code_css.to_string(),
            name,
        }
    }

    /// Renders content using this theme with inline styles for WeChat.
    pub fn render(&self, content: &str, metadata: &HashMap<String, String>) -> Result<String> {
        // Create Askama template with the provided content and metadata
        let template = ArticleTemplate {
            title: metadata.get("title").cloned().unwrap_or_default(),
            description: metadata.get("description").cloned().unwrap_or_default(),
            author: metadata.get("author").cloned().unwrap_or_default(),
            content: content.to_string(),
            theme_css: self.theme_css.clone(),
            highlight_css: self.code_css.clone(),
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
    highlight_css: HashMap<String, String>,
    markdown_options: ComrakOptions<'static>,
}

impl ThemeManager {
    /// Creates a new theme manager with built-in themes.
    pub fn new() -> Self {
        let mut manager = Self {
            templates: HashMap::new(),
            highlight_css: HashMap::new(),
            markdown_options: Self::create_markdown_options(),
        };

        manager.load_builtin_themes();
        manager.load_highlight_themes();
        manager
    }

    /// Creates markdown parsing options.
    fn create_markdown_options() -> ComrakOptions<'static> {
        let mut options = ComrakOptions::default();
        options.extension.strikethrough = true;
        options.extension.table = true;
        options.extension.footnotes = true;
        options.extension.tasklist = true;
        options.parse.smart = true;
        options
    }

    /// Loads all built-in themes from embedded CSS.
    fn load_builtin_themes(&mut self) {
        for theme in BuiltinTheme::all() {
            let template = self.create_builtin_theme(theme);
            self.templates.insert(theme.as_str().to_string(), template);
        }
    }

    /// Loads all highlight themes from embedded CSS.
    fn load_highlight_themes(&mut self) {
        // Load all embedded highlight themes
        self.highlight_css
            .insert("atom-one-dark".to_string(), ATOM_ONE_DARK_CSS.to_string());
        self.highlight_css
            .insert("atom-one-light".to_string(), ATOM_ONE_LIGHT_CSS.to_string());
        self.highlight_css
            .insert("dracula".to_string(), DRACULA_CSS.to_string());
        self.highlight_css
            .insert("github-dark".to_string(), GITHUB_DARK_CSS.to_string());
        self.highlight_css
            .insert("github".to_string(), GITHUB_CSS.to_string());
        self.highlight_css
            .insert("monokai".to_string(), MONOKAI_CSS.to_string());
        self.highlight_css
            .insert("solarized-dark".to_string(), SOLARIZED_DARK_CSS.to_string());
        self.highlight_css.insert(
            "solarized-light".to_string(),
            SOLARIZED_LIGHT_CSS.to_string(),
        );
        self.highlight_css
            .insert("xcode".to_string(), XCODE_CSS.to_string());

        // Add vscode as an alias for github
        self.highlight_css
            .insert("vscode".to_string(), GITHUB_CSS.to_string());
    }

    /// Creates a built-in theme template from embedded CSS.
    fn create_builtin_theme(&self, theme: BuiltinTheme) -> ThemeTemplate {
        let css = self.get_embedded_theme_css(theme);
        ThemeTemplate::from_static(css, "", theme.as_str().to_string())
    }

    /// Gets embedded CSS content for built-in themes.
    fn get_embedded_theme_css(&self, theme: BuiltinTheme) -> &'static str {
        match theme {
            BuiltinTheme::Default => DEFAULT_CSS,
            BuiltinTheme::Lapis => LAPIS_CSS,
            BuiltinTheme::Maize => MAIZE_CSS,
            BuiltinTheme::OrangeHeart => ORANGEHEART_CSS,
            BuiltinTheme::PhyCat => PHYCAT_CSS,
            BuiltinTheme::Pie => PIE_CSS,
            BuiltinTheme::Purple => PURPLE_CSS,
            BuiltinTheme::Rainbow => RAINBOW_CSS,
        }
    }

    /// Renders markdown content with the specified theme and code highlight theme.
    pub fn render(
        &self,
        markdown_content: &str,
        theme_name: &str,
        code_theme: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<String> {
        let template =
            self.templates
                .get(theme_name)
                .ok_or_else(|| WeChatError::ThemeNotFound {
                    theme: theme_name.to_string(),
                })?;

        // Get highlight CSS, defaulting to "vscode" if not specified or not found
        let highlight_css = self.get_highlight_css(code_theme);

        // Create syntect adapter for syntax highlighting
        // Map our CSS theme names to syntect theme names
        let syntect_theme_name = match code_theme {
            "solarized-light" => Some("Solarized (light)"),
            "solarized-dark" => Some("Solarized (dark)"),
            "monokai" => Some("Monokai"),
            "github" | "vscode" => Some("InspiredGitHub"),
            "github-dark" => Some("base16-ocean.dark"),
            "atom-one-dark" => Some("base16-ocean.dark"),
            "atom-one-light" => Some("InspiredGitHub"),
            "dracula" => Some("base16-ocean.dark"),
            "xcode" => Some("InspiredGitHub"),
            _ => None, // Use default theme
        };

        let adapter = SyntectAdapter::new(syntect_theme_name);

        // Set up comrak plugins with syntect adapter
        let mut plugins = ComrakPlugins::default();
        plugins.render.codefence_syntax_highlighter = Some(&adapter);

        // Convert markdown to HTML using comrak with syntect
        let html_content =
            markdown_to_html_with_plugins(markdown_content, &self.markdown_options, &plugins);

        // Create a new template with the highlight CSS
        let template_with_highlight = ThemeTemplate {
            theme_css: template.theme_css.clone(),
            code_css: highlight_css,
            name: template.name.clone(),
        };

        // Apply theme template
        template_with_highlight.render(&html_content, metadata)
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

    /// Gets highlight CSS for a given theme, falling back to default if not found.
    fn get_highlight_css(&self, theme: &str) -> String {
        self.highlight_css.get(theme).cloned().unwrap_or_else(|| {
            log::warn!("Highlight theme '{theme}' not found, falling back to 'github'");
            self.highlight_css
                .get("github")
                .cloned()
                .unwrap_or_default()
        })
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

        let result = manager.render(markdown, "default", "vscode", &metadata);
        assert!(result.is_ok());

        let html = result.unwrap();
        assert!(html.contains("<h1"));
        assert!(html.contains("Test Title"));
        assert!(html.contains("<strong"));
        assert!(html.contains("bold"));
        assert!(html.contains("id=\"wepub\""));
    }

    #[test]
    fn test_nonexistent_theme() {
        let manager = ThemeManager::new();
        let result = manager.render("# Test", "nonexistent", "vscode", &HashMap::new());

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
            "#wepub { color: red; }".to_string(),
            String::new(),
            "custom".to_string(),
        );

        manager.add_theme("custom".to_string(), custom_template);
        assert!(manager.has_theme("custom"));

        let result = manager.render("# Test", "custom", "vscode", &HashMap::new());
        assert!(result.is_ok());

        let html = result.unwrap();
        assert!(html.contains("style="));
        assert!(html.contains("Test"));
        assert!(html.contains("id=\"wepub\""));
    }

    #[test]
    fn test_highlight_theme_rendering() {
        let manager = ThemeManager::new();
        let markdown = "# Test\n\n```rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n```";

        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "Test Article".to_string());
        metadata.insert("author".to_string(), "Test Author".to_string());

        // Test with specific highlight theme
        let result = manager.render(markdown, "default", "solarized-light", &metadata);
        assert!(result.is_ok());

        let html = result.unwrap();
        assert!(html.contains("<h1"));
        assert!(html.contains("Test"));
        assert!(html.contains("<code"));

        // Test with default highlight theme (None)
        let result = manager.render(markdown, "default", "vscode", &metadata);
        assert!(result.is_ok());

        // Test with nonexistent highlight theme (should fallback to github)
        let result = manager.render(markdown, "default", "nonexistent", &metadata);
        assert!(result.is_ok());
    }

    #[test]
    fn test_template_css_inlining() {
        let css = "#wepub h1 { color: red; font-size: 2em; }";
        let template = ThemeTemplate::new(css.to_string(), String::new(), "theme".to_string());

        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "My Title".to_string());
        metadata.insert("author".to_string(), "John Doe".to_string());

        let result = template.render("<h1>Content</h1>", &metadata);
        assert!(result.is_ok());

        let html = result.unwrap();
        assert!(html.contains("id=\"wepub\""));
        assert!(html.contains("<h1"));
        assert!(html.contains("Content"));
        // Check that CSS was applied as inline styles
        assert!(html.contains("style="));
    }
}
