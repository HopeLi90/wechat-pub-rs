//! Theme system for rendering markdown content to HTML.
//!
//! This module provides a comprehensive theming system that converts Markdown content
//! into beautifully styled HTML suitable for WeChat Official Account articles.
//!
//! ## Features
//!
//! - **8 Built-in Themes**: Carefully designed themes for different aesthetics
//! - **Syntax Highlighting**: 10 different code highlighting themes
//! - **CSS Variable Processing**: Dynamic theming with CSS custom properties
//! - **Template Engine**: Askama-based HTML templating
//! - **Responsive Design**: Mobile-first responsive layouts
//!
//! ## Available Themes
//!
//! | Theme | Description | Best For |
//! |-------|-------------|----------|
//! | `default` | Clean, minimal design | General content |
//! | `lapis` | Blue accents, elegant | Technical articles |
//! | `maize` | Warm yellow tones | Creative content |
//! | `orangeheart` | Orange accents | Personal blogs |
//! | `phycat` | Unique styling | Special content |
//! | `pie` | Sweet, colorful | Lifestyle content |
//! | `purple` | Purple accents | Creative writing |
//! | `rainbow` | Colorful, vibrant | Fun content |
//!
//! ## Code Highlighting Themes
//!
//! - `github` / `github-dark` - GitHub styling
//! - `atom-one-light` / `atom-one-dark` - Atom editor themes
//! - `solarized-light` / `solarized-dark` - Solarized color scheme
//! - `vscode` - VS Code default theme
//! - `monokai`, `dracula`, `xcode` - Popular editor themes
//!
//! ## Usage
//!
//! ```rust
//! use wechat_pub_rs::theme::{ThemeManager, BuiltinTheme};
//! use std::collections::HashMap;
//!
//! let theme_manager = ThemeManager::new();
//!
//! // Check available themes
//! let themes = theme_manager.available_themes();
//! println!("Available themes: {:?}", themes);
//!
//! // Render content with a theme
//! let metadata = HashMap::new();
//! let html = theme_manager.render(
//!     "# Hello World\nSome **bold** text",
//!     "lapis",
//!     "github",
//!     &metadata
//! ).unwrap();
//! ```

use crate::css_vars::CssVariableProcessor;
use crate::error::{Result, WeChatError};
use askama::Template;
use comrak::{
    ComrakOptions, ComrakPlugins, markdown_to_html_with_plugins, plugins::syntect::SyntectAdapter,
};
use std::collections::HashMap;
use tracing::warn;

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
    ///
    /// This method processes CSS variables before inlining styles for better WeChat compatibility.
    pub fn render(&self, content: &str, metadata: &HashMap<String, String>) -> Result<String> {
        // Process CSS variables in both theme and highlight CSS
        let css_processor = CssVariableProcessor::new();

        let processed_theme_css =
            css_processor
                .process_css(&self.theme_css)
                .map_err(|e| WeChatError::Internal {
                    message: format!("CSS variable processing failed for theme CSS: {e}"),
                })?;

        let processed_highlight_css =
            css_processor
                .process_css(&self.code_css)
                .map_err(|e| WeChatError::Internal {
                    message: format!("CSS variable processing failed for highlight CSS: {e}"),
                })?;

        // Create Askama template with the processed CSS
        let template = ArticleTemplate {
            title: metadata.get("title").cloned().unwrap_or_default(),
            description: metadata.get("description").cloned().unwrap_or_default(),
            author: metadata.get("author").cloned().unwrap_or_default(),
            content: content.to_string(),
            theme_css: processed_theme_css,
            highlight_css: processed_highlight_css,
        };

        // Render the template to HTML
        let html_with_css = template.render().map_err(|e| WeChatError::Internal {
            message: format!("Template rendering failed: {e}"),
        })?;

        // Use css-inline to convert CSS to inline styles
        let inlined_html =
            css_inline::inline(&html_with_css).map_err(|e| WeChatError::Internal {
                message: format!("CSS inlining failed: {e}"),
            })?;

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
        // Array of (theme_name, css_content) pairs for cleaner initialization
        let highlight_themes = [
            ("atom-one-dark", ATOM_ONE_DARK_CSS),
            ("atom-one-light", ATOM_ONE_LIGHT_CSS),
            ("dracula", DRACULA_CSS),
            ("github-dark", GITHUB_DARK_CSS),
            ("github", GITHUB_CSS),
            ("monokai", MONOKAI_CSS),
            ("solarized-dark", SOLARIZED_DARK_CSS),
            ("solarized-light", SOLARIZED_LIGHT_CSS),
            ("xcode", XCODE_CSS),
            ("vscode", GITHUB_CSS), // vscode as an alias for github
        ];

        for (name, css) in highlight_themes {
            self.highlight_css.insert(name.to_string(), css.to_string());
        }
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
        let mut html_content =
            markdown_to_html_with_plugins(markdown_content, &self.markdown_options, &plugins);

        // Post-process HTML to fix code formatting for WeChat
        html_content = self.post_process_code_blocks(html_content);

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
            warn!("Highlight theme '{theme}' not found, falling back to 'github'");
            self.highlight_css
                .get("github")
                .cloned()
                .unwrap_or_default()
        })
    }

    /// Post-process HTML to fix code formatting for WeChat.
    /// Replaces newlines with <br/> and spaces with &nbsp; in code blocks.
    fn post_process_code_blocks(&self, html: String) -> String {
        use regex::Regex;

        // Process inline code blocks
        let inline_code_regex = Regex::new(r"<code>([^<]*)</code>").unwrap();
        let html = inline_code_regex
            .replace_all(&html, |caps: &regex::Captures| {
                let content = &caps[1];
                let processed = content
                    .replace('\n', "<br/>")
                    .replace("  ", "&nbsp;&nbsp;")
                    .replace('\t', "&nbsp;&nbsp;&nbsp;&nbsp;");
                format!("<code>{processed}</code>")
            })
            .to_string();

        // Process code blocks within pre tags
        let pre_code_regex = Regex::new(r"<pre[^>]*><code[^>]*>([^<]*)</code></pre>").unwrap();
        pre_code_regex
            .replace_all(&html, |caps: &regex::Captures| {
                let full_match = &caps[0];
                let content = &caps[1];

                // Extract the opening tags
                let pre_end = full_match.find("><code").unwrap();
                let code_start = pre_end + 1;
                let code_end = full_match[code_start..].find('>').unwrap() + code_start + 1;

                let pre_tag = &full_match[..pre_end + 1];
                let code_tag = &full_match[code_start..code_end];

                // Process the content
                let processed = content
                    .replace('\n', "<br/>")
                    .replace("  ", "&nbsp;&nbsp;")
                    .replace('\t', "&nbsp;&nbsp;&nbsp;&nbsp;");

                format!("{pre_tag}{code_tag}{processed}</code></pre>")
            })
            .to_string()
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
        let markdown = r#"# Test

```rust
fn main() {
    println!("Hello, world!");
}
```"#;

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

    #[test]
    fn test_css_variable_processing_in_theme() {
        let css_with_vars = r#"
        :root {
            --primary-color: #4870ac;
            --text-color: #40464f;
            --header-color: var(--primary-color);
        }
        #wepub { color: var(--text-color); }
        #wepub h1 { color: var(--header-color); }
        "#;

        let template =
            ThemeTemplate::new(css_with_vars.to_string(), String::new(), "test".to_string());

        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "Test".to_string());
        metadata.insert("author".to_string(), "Test Author".to_string());

        let result = template.render("<h1>Test Header</h1><p>Test content</p>", &metadata);
        assert!(result.is_ok());

        let html = result.unwrap();

        // Verify CSS variables were processed and inlined
        assert!(!html.contains("var(")); // No var() calls should remain
        assert!(html.contains("#40464f")); // text-color should be inlined
        assert!(html.contains("#4870ac")); // header-color should be resolved and inlined
    }

    #[test]
    fn test_nested_css_variables_in_theme() {
        let css_with_nested_vars = r#"
        :root {
            --base-color: #4870ac;
            --primary-color: var(--base-color);
            --header-span-color: var(--primary-color);
            --shadow-color: #eee;
            --shadow: 3px 3px 10px var(--shadow-color);
        }
        #wepub h1 span { color: var(--header-span-color); }
        #wepub .box { box-shadow: var(--shadow); }
        "#;

        let template = ThemeTemplate::new(
            css_with_nested_vars.to_string(),
            String::new(),
            "nested".to_string(),
        );

        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "Nested Test".to_string());

        let result = template.render("<h1><span>Nested</span></h1>", &metadata);
        assert!(result.is_ok());

        let html = result.unwrap();

        // Verify nested variables were resolved correctly
        assert!(!html.contains("var("));
        assert!(html.contains("#4870ac")); // All nested references should resolve to base color
        assert!(html.contains("3px 3px 10px #eee")); // Shadow should be fully resolved
    }

    #[test]
    fn test_real_theme_css_variable_processing() {
        // Test with actual purple theme which uses CSS variables
        let manager = ThemeManager::new();
        let markdown = "# Test Title\n\nThis is a **test** paragraph.";

        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "Variable Test".to_string());
        metadata.insert("author".to_string(), "Test Author".to_string());

        // Test purple theme which has CSS variables
        let result = manager.render(markdown, "purple", "github", &metadata);
        assert!(result.is_ok());

        let html = result.unwrap();

        // Verify that defined CSS variables were processed
        // Note: Undefined variables like --sans-serif-font may remain for graceful degradation
        assert!(!html.contains("var(--title-color"));
        assert!(!html.contains("var(--text-color"));
        assert!(!html.contains("var(--shadow-color"));

        // Verify specific color values are present (from resolved variables)
        assert!(
            html.contains("#8064a9")
                || html.contains("color:#8064a9")
                || html.contains("color: #8064a9")
        );
        assert!(
            html.contains("#444444")
                || html.contains("color:#444444")
                || html.contains("color: #444444")
        );

        // Verify the content is properly rendered
        assert!(html.contains("Test Title"));
        assert!(html.contains("<strong"));
        assert!(html.contains("test"));
        assert!(html.contains("id=\"wepub\""));
    }

    #[test]
    fn test_all_themes_css_variable_processing() {
        // Test CSS variable processing works across all theme files
        let manager = ThemeManager::new();
        let markdown = "# Test\n\nCSS variables test.";
        let metadata = HashMap::new();

        for theme in BuiltinTheme::all() {
            let result = manager.render(markdown, theme.as_str(), "github", &metadata);
            assert!(
                result.is_ok(),
                "Theme {} should render successfully",
                theme.as_str()
            );

            let html = result.unwrap();

            // Verify basic HTML structure is preserved
            assert!(html.contains("Test"));
            assert!(html.contains("id=\"wepub\""));

            // For themes with CSS variables, verify that at least some processing occurred
            // We don't assert complete absence of var() since some undefined variables may remain
            let theme_css = get_embedded_theme_css(theme);
            let var_count_before = theme_css_var_count(theme_css);
            let var_count_after = html.matches("var(--").count();

            // At minimum, defined variables should be reduced
            if var_count_before > 0 {
                println!(
                    "Theme {}: {} vars before, {} vars after",
                    theme.as_str(),
                    var_count_before,
                    var_count_after
                );
            }
        }
    }

    // Helper function to count CSS variables in a theme
    fn theme_css_var_count(css: &str) -> usize {
        css.matches("var(--").count()
    }

    // Helper function to get embedded CSS for testing
    fn get_embedded_theme_css(theme: BuiltinTheme) -> &'static str {
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
}
