# 微信公众号 Rust SDK 设计文档

## 1. SDK 目标和范围

### 1.1 项目目标

构建一个简单易用、高性能的微信公众号 Rust SDK，专注于核心的图片上传和图文草稿功能。

**核心价值主张**：

- **极简API**：一个函数完成整个发布流程 `wx.upload("./article.md", "theme1")`
- **零配置**：内置主题系统，开箱即用
- **高性能**：利用 Rust 的零成本抽象和异步特性
- **类型安全**：编译时错误检测，运行时稳定可靠

### 1.2 功能范围

**包含功能**：

- ✅ Markdown 解析和图片提取
- ✅ 图片自动上传和URL替换
- ✅ 内置主题系统 (Markdown → HTML)
- ✅ 图文草稿创建和管理
- ✅ 访问令牌自动管理
- ✅ 错误处理和重试机制

**不包含功能**：

- ❌ 消息推送和接收
- ❌ 用户管理
- ❌ 菜单管理
- ❌ 其他高级功能

### 1.3 目标用户

- 内容创作者和博客作者
- 自媒体运营者
- 开发者和技术写作者
- 需要自动化发布流程的团队

## 2. API 设计

### 2.1 核心 API

```rust
// 最简单的使用方式
wx.upload("./article.md", "default").await?;

// 带选项的使用方式
wx.upload_with_options("./article.md", UploadOptions {
    theme: "github",
    title: Some("自定义标题".to_string()),
    author: Some("作者名".to_string()),
    cover_image: Some("./cover.jpg"),
    show_cover: true,
    enable_comments: true,
}).await?;
```

### 2.2 完整 API 规范

```rust
use wechat_pub::{WeChatClient, UploadOptions, Result};

// 客户端初始化
let client = WeChatClient::new("app_id", "app_secret").await?;

// 核心上传方法
impl WeChatClient {
    // 简化方法：自动检测标题、作者等信息
    pub async fn upload(&self, markdown_path: &str, theme: &str) -> Result<String>;

    // 完整方法：提供所有选项
    pub async fn upload_with_options(&self, markdown_path: &str, options: UploadOptions) -> Result<String>;

    // 草稿管理方法
    pub async fn get_draft(&self, media_id: &str) -> Result<Draft>;
    pub async fn update_draft(&self, media_id: &str, markdown_path: &str, theme: &str) -> Result<()>;
    pub async fn delete_draft(&self, media_id: &str) -> Result<()>;
    pub async fn list_drafts(&self, offset: u32, count: u32) -> Result<Vec<DraftInfo>>;

    // 底层方法（高级用户）
    pub async fn upload_image(&self, image_path: &str) -> Result<String>;
    pub async fn create_draft(&self, articles: Vec<Article>) -> Result<String>;
}

// 配置结构体
#[derive(Debug, Clone)]
pub struct UploadOptions {
    pub theme: String,                    // 主题名称
    pub title: Option<String>,            // 文章标题（默认从MD提取）
    pub author: Option<String>,           // 作者名称
    pub cover_image: Option<String>,      // 封面图片路径
    pub show_cover: bool,                 // 是否显示封面（默认true）
    pub enable_comments: bool,            // 是否开启评论（默认false）
    pub fans_only_comments: bool,         // 是否仅粉丝可评论（默认false）
    pub source_url: Option<String>,       // 原文链接
}
```

### 2.3 主题系统

```rust
// 内置主题
pub enum BuiltinTheme {
    Default,    // 简洁默认样式
    Github,     // GitHub风格
    Zhihu,      // 知乎风格
    Juejin,     // 掘金风格
    Wechat,     // 微信原生风格
}

// 自定义主题支持
pub struct CustomTheme {
    pub css: String,              // CSS样式
    pub template: String,         // HTML模板
    pub code_theme: String,       // 代码高亮主题
}
```

## 3. 核心流程设计

### 3.1 主流程图

```
输入Markdown文件
        ↓
    解析MD内容
        ↓
    提取元数据（标题、作者等）
        ↓
    查找图片引用
        ↓
    并发上传图片
        ↓
    替换图片URL
        ↓
    应用主题渲染HTML
        ↓
    上传封面图（如果有）
        ↓
    创建图文草稿
        ↓
    返回草稿ID
```

### 3.2 详细处理步骤

#### 步骤1：Markdown 解析

```rust
struct MarkdownContent {
    title: Option<String>,           // 从 # 或 front-matter 提取
    author: Option<String>,          // 从 front-matter 提取
    content: String,                 // 正文内容
    images: Vec<ImageRef>,           // 图片引用列表
    metadata: HashMap<String, String>, // 其他元数据
}

struct ImageRef {
    alt_text: String,                // 图片alt文本
    original_url: String,            // 原始URL或路径
    position: (usize, usize),        // 在文本中的位置
}
```

#### 步骤2：图片处理

```rust
async fn process_images(&self, images: Vec<ImageRef>, base_path: &Path) -> Result<Vec<ProcessedImage>> {
    let tasks: Vec<_> = images.into_iter().map(|img| {
        let client = self.clone();
        let base_path = base_path.to_owned();
        tokio::spawn(async move {
            client.upload_single_image(img, &base_path).await
        })
    }).collect();

    // 并发上传所有图片
    let results = futures::future::try_join_all(tasks).await?;
    Ok(results.into_iter().collect::<Result<Vec<_>, _>>()?)
}
```

#### 步骤3：主题渲染

```rust
struct ThemeRenderer {
    template_engine: Tera,
    css_processor: CssProcessor,
    syntax_highlighter: SyntaxHighlighter,
}

impl ThemeRenderer {
    async fn render(&self, content: &MarkdownContent, theme: &str) -> Result<String> {
        let html = self.markdown_to_html(&content.content).await?;
        let styled_html = self.apply_theme(html, theme).await?;
        Ok(styled_html)
    }
}
```

### 3.3 错误恢复机制

```rust
// 重试策略
struct RetryConfig {
    max_attempts: u32,               // 最大重试次数
    base_delay: Duration,            // 基础延迟
    max_delay: Duration,             // 最大延迟
    backoff_factor: f64,             // 退避因子
}

// 支持的错误类型
#[derive(Debug, thiserror::Error)]
pub enum WeChatError {
    #[error("网络错误: {0}")]
    Network(#[from] reqwest::Error),

    #[error("认证失败: {message}")]
    Authentication { message: String },

    #[error("图片上传失败: {path}, 原因: {reason}")]
    ImageUpload { path: String, reason: String },

    #[error("Markdown解析错误: {0}")]
    MarkdownParse(String),

    #[error("主题渲染错误: {theme}, 原因: {reason}")]
    ThemeRender { theme: String, reason: String },
}
```

## 4. 技术架构设计

### 4.1 整体架构

```
┌─────────────────────────────────────────────┐
│                 Public API                  │
├─────────────────────────────────────────────┤
│              Business Logic                 │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────┐ │
│  │   Markdown  │ │   Theme     │ │  Draft  │ │
│  │   Parser    │ │   Renderer  │ │ Manager │ │
│  └─────────────┘ └─────────────┘ └─────────┘ │
├─────────────────────────────────────────────┤
│               Core Services                 │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────┐ │
│  │    Image    │ │   Access    │ │  HTTP   │ │
│  │  Uploader   │ │    Token    │ │ Client  │ │
│  └─────────────┘ └─────────────┘ └─────────┘ │
├─────────────────────────────────────────────┤
│              Infrastructure                 │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────┐ │
│  │   Config    │ │    Cache    │ │ Logger  │ │
│  │  Manager    │ │   Manager   │ │ Manager │ │
│  └─────────────┘ └─────────────┘ └─────────┘ │
└─────────────────────────────────────────────┘
```

### 4.2 核心组件设计

#### 4.2.1 HTTP 客户端

```rust
pub struct WeChatHttpClient {
    client: reqwest::Client,
    base_url: String,
    token_manager: Arc<TokenManager>,
    retry_config: RetryConfig,
}

impl WeChatHttpClient {
    // 自动添加access_token的GET请求
    pub async fn get_with_token(&self, endpoint: &str) -> Result<Response>;

    // 自动添加access_token的POST请求
    pub async fn post_json_with_token<T: Serialize>(&self, endpoint: &str, body: &T) -> Result<Response>;

    // 文件上传请求
    pub async fn upload_file(&self, endpoint: &str, field_name: &str, file_data: Vec<u8>, filename: &str) -> Result<Response>;
}
```

#### 4.2.2 访问令牌管理

```rust
pub struct TokenManager {
    app_id: String,
    app_secret: String,
    token_cache: Arc<RwLock<Option<AccessToken>>>,
    http_client: reqwest::Client,
}

struct AccessToken {
    token: String,
    expires_at: Instant,
}

impl TokenManager {
    pub async fn get_access_token(&self) -> Result<String> {
        // 检查缓存
        if let Some(token) = self.get_cached_token().await {
            return Ok(token);
        }

        // 获取新令牌
        self.refresh_token().await
    }

    async fn refresh_token(&self) -> Result<String> {
        // 防止并发刷新
        let _guard = self.refresh_lock.lock().await;

        // 双重检查
        if let Some(token) = self.get_cached_token().await {
            return Ok(token);
        }

        // 调用API获取新令牌
        // ...
    }
}
```

#### 4.2.3 图片上传器

```rust
pub struct ImageUploader {
    http_client: Arc<WeChatHttpClient>,
    semaphore: Semaphore, // 限制并发数
}

impl ImageUploader {
    pub async fn upload_images(&self, images: Vec<ImageRef>, base_path: &Path) -> Result<Vec<UploadResult>> {
        let tasks: Vec<_> = images.into_iter().map(|img| {
            let client = self.http_client.clone();
            let permit = self.semaphore.acquire();
            let base_path = base_path.to_owned();

            async move {
                let _permit = permit.await?;
                self.upload_single_image(img, &base_path).await
            }
        }).collect();

        futures::future::try_join_all(tasks).await
    }

    async fn upload_single_image(&self, image_ref: ImageRef, base_path: &Path) -> Result<UploadResult> {
        // 支持本地文件和URL
        let image_data = if image_ref.original_url.starts_with("http") {
            self.download_image(&image_ref.original_url).await?
        } else {
            let path = base_path.join(&image_ref.original_url);
            tokio::fs::read(path).await?
        };

        // 上传到微信
        self.upload_to_wechat(image_data, &image_ref.alt_text).await
    }
}
```

#### 4.2.4 主题系统

```rust
pub struct ThemeManager {
    templates: HashMap<String, ThemeTemplate>,
    markdown_renderer: MarkdownRenderer,
}

struct ThemeTemplate {
    css: String,
    html_template: String,
    code_theme: String,
}

impl ThemeManager {
    pub fn new() -> Self {
        let mut manager = Self {
            templates: HashMap::new(),
            markdown_renderer: MarkdownRenderer::new(),
        };

        // 加载内置主题
        manager.load_builtin_themes();
        manager
    }

    pub async fn render(&self, content: &str, theme: &str, metadata: &HashMap<String, String>) -> Result<String> {
        let template = self.templates.get(theme)
            .ok_or_else(|| WeChatError::ThemeNotFound(theme.to_string()))?;

        // Markdown -> HTML
        let html_content = self.markdown_renderer.render(content)?;

        // 应用模板
        let mut context = tera::Context::new();
        context.insert("content", &html_content);
        context.insert("metadata", metadata);

        let result = template.render(&context)?;
        Ok(result)
    }
}
```

### 4.3 并发和性能优化

#### 4.3.1 异步处理

```rust
// 图片上传采用并发处理
pub async fn upload_images_concurrently(&self, images: Vec<ImageRef>) -> Result<Vec<String>> {
    const MAX_CONCURRENT_UPLOADS: usize = 5;

    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_UPLOADS));
    let tasks: Vec<_> = images.into_iter().map(|img| {
        let semaphore = semaphore.clone();
        let client = self.clone();

        tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            client.upload_image(&img).await
        })
    }).collect();

    let results = futures::future::try_join_all(tasks).await?;
    Ok(results.into_iter().collect::<Result<Vec<_>, _>>()?)
}
```

#### 4.3.2 缓存策略

```rust
pub struct CacheManager {
    token_cache: Arc<RwLock<Option<AccessToken>>>,
    image_cache: Arc<RwLock<HashMap<String, String>>>, // URL -> WeChat URL
    template_cache: Arc<RwLock<HashMap<String, ThemeTemplate>>>,
}

impl CacheManager {
    // 令牌缓存：自动过期管理
    pub async fn get_cached_token(&self) -> Option<String>;

    // 图片缓存：避免重复上传相同图片
    pub async fn get_cached_image_url(&self, local_path: &str) -> Option<String>;

    // 模板缓存：避免重复编译模板
    pub async fn get_cached_template(&self, theme: &str) -> Option<ThemeTemplate>;
}
```

## 5. 错误处理策略

### 5.1 错误分类和处理

```rust
#[derive(Debug, thiserror::Error)]
pub enum WeChatError {
    // 网络相关错误 - 可重试
    #[error("网络请求失败: {0}")]
    Network(#[from] reqwest::Error),

    #[error("请求超时")]
    Timeout,

    // 认证相关错误 - 可重试一次
    #[error("访问令牌无效")]
    InvalidToken,

    #[error("应用凭据无效")]
    InvalidCredentials,

    // 业务逻辑错误 - 不可重试
    #[error("文件不存在: {path}")]
    FileNotFound { path: String },

    #[error("Markdown解析失败: {reason}")]
    MarkdownParseError { reason: String },

    #[error("主题不存在: {theme}")]
    ThemeNotFound { theme: String },

    // 微信API错误 - 根据错误码决定是否重试
    #[error("微信API错误 [{code}]: {message}")]
    WeChatApi { code: i32, message: String },
}

impl WeChatError {
    pub fn is_retryable(&self) -> bool {
        match self {
            WeChatError::Network(_) => true,
            WeChatError::Timeout => true,
            WeChatError::InvalidToken => true,
            WeChatError::WeChatApi { code, .. } => {
                match code {
                    40001 | 40014 | 42001 => true,  // token相关错误
                    _ => false,
                }
            },
            _ => false,
        }
    }
}
```

### 5.2 重试机制

```rust
pub struct RetryExecutor {
    config: RetryConfig,
}

impl RetryExecutor {
    pub async fn execute_with_retry<F, T, E>(&self, mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> Pin<Box<dyn Future<Output = Result<T, E>> + Send>>,
        E: IsRetryable,
    {
        let mut delay = self.config.base_delay;

        for attempt in 1..=self.config.max_attempts {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if attempt == self.config.max_attempts || !error.is_retryable() {
                        return Err(error);
                    }

                    tokio::time::sleep(delay).await;
                    delay = std::cmp::min(
                        Duration::from_millis((delay.as_millis() as f64 * self.config.backoff_factor) as u64),
                        self.config.max_delay,
                    );
                }
            }
        }

        unreachable!()
    }
}
```

### 5.3 错误恢复和回滚

```rust
pub struct UploadTransaction {
    uploaded_images: Vec<String>, // 已上传的图片URL
    draft_id: Option<String>,     // 已创建的草稿ID
}

impl UploadTransaction {
    pub async fn rollback(&self, client: &WeChatClient) -> Result<()> {
        // 删除已创建的草稿
        if let Some(draft_id) = &self.draft_id {
            let _ = client.delete_draft(draft_id).await; // 忽略删除失败
        }

        // 注意：微信不支持删除已上传的图片，这里只做记录
        if !self.uploaded_images.is_empty() {
            warn!("无法删除已上传的图片: {:?}", self.uploaded_images);
        }

        Ok(())
    }
}
```

## 6. 实现计划

### 6.1 开发阶段

#### 阶段1：基础框架 (2周)

- [x] 项目结构搭建
- [ ] HTTP客户端实现
- [ ] 访问令牌管理
- [ ] 基础错误处理
- [ ] 配置管理系统

**里程碑**: 能够成功调用微信API并获取访问令牌

#### 阶段2：核心功能 (3周)

- [ ] Markdown解析器
- [ ] 图片上传功能
- [ ] 草稿创建和管理
- [ ] 基础主题系统
- [ ] 单元测试覆盖

**里程碑**: 能够上传简单的Markdown文章到微信草稿

#### 阶段3：高级功能 (2周)

- [ ] 并发图片上传
- [ ] 高级主题系统
- [ ] 错误恢复机制
- [ ] 性能优化
- [ ] 集成测试

**里程碑**: 支持复杂文章发布，性能达到预期目标

#### 阶段4：完善和发布 (1周)

- [ ] 文档完善
- [ ] 示例代码
- [ ] 性能基准测试
- [ ] 发布准备

**里程碑**: 1.0版本发布

### 6.2 技术栈选择

#### 核心依赖

```toml
[dependencies]
# HTTP客户端
reqwest = { version = "0.11", features = ["json", "multipart"] }

# 异步运行时
tokio = { version = "1.0", features = ["full"] }
futures = "0.3"

# 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Markdown处理
pulldown-cmark = "0.9"
pulldown-cmark-to-cmark = "10.0"

# 模板引擎
tera = "1.19"

# 错误处理
thiserror = "1.0"
anyhow = "1.0"

# 日志
log = "0.4"
env_logger = "0.10"

# 时间处理
chrono = { version = "0.4", features = ["serde"] }

# 配置管理
config = "0.13"

# 文件处理
mime_guess = "2.0"
```

#### 开发工具

```toml
[dev-dependencies]
tokio-test = "0.4"
mockito = "1.2"
criterion = "0.5"
proptest = "1.3"
```

### 6.3 质量保证

#### 6.3.1 测试策略

```rust
// 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_markdown_parsing() {
        // 测试Markdown解析功能
    }

    #[tokio::test]
    async fn test_image_upload() {
        // 测试图片上传功能
    }

    #[tokio::test]
    async fn test_theme_rendering() {
        // 测试主题渲染功能
    }
}

// 集成测试
#[cfg(test)]
mod integration_tests {
    #[tokio::test]
    async fn test_full_upload_workflow() {
        // 测试完整的上传流程
    }
}

// 性能测试
mod benches {
    use criterion::{black_box, criterion_group, criterion_main, Criterion};

    fn benchmark_upload(c: &mut Criterion) {
        c.bench_function("upload_large_article", |b| {
            b.iter(|| {
                // 性能基准测试
            })
        });
    }
}
```

#### 6.3.2 代码质量

- **测试覆盖率**: >80%
- **文档覆盖率**: >90%
- **Clippy检查**: 无警告
- **格式化**: rustfmt标准
- **性能要求**:
  - 10张图片并发上传 <30秒
  - 内存使用 <50MB
  - CPU使用率 <50%

### 6.4 项目结构

```
wechat-pub-rs/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs                 # 公共API入口
│   ├── client.rs              # 主要客户端实现
│   ├── config.rs              # 配置管理
│   ├── error.rs               # 错误类型定义
│   ├── http/                  # HTTP客户端模块
│   │   ├── mod.rs
│   │   ├── client.rs
│   │   └── retry.rs
│   ├── auth/                  # 认证模块
│   │   ├── mod.rs
│   │   └── token_manager.rs
│   ├── upload/                # 上传模块
│   │   ├── mod.rs
│   │   ├── image_uploader.rs
│   │   └── draft_manager.rs
│   ├── markdown/              # Markdown处理
│   │   ├── mod.rs
│   │   ├── parser.rs
│   │   └── image_extractor.rs
│   ├── theme/                 # 主题系统
│   │   ├── mod.rs
│   │   ├── manager.rs
│   │   ├── renderer.rs
│   │   └── builtin/           # 内置主题
│   │       ├── default.css
│   │       ├── github.css
│   │       └── template.html
│   └── utils/                 # 工具函数
│       ├── mod.rs
│       ├── cache.rs
│       └── file.rs
├── tests/                     # 集成测试
│   ├── integration_test.rs
│   └── fixtures/
│       ├── sample.md
│       └── test_images/
├── benches/                   # 性能测试
│   └── upload_benchmark.rs
├── examples/                  # 示例代码
│   ├── simple_upload.rs
│   └── advanced_usage.rs
└── docs/                      # 文档
    ├── api.md
    ├── themes.md
    └── troubleshooting.md
```

### 6.5 发布计划

#### 版本规划

- **v0.1.0**: 基础功能MVP
- **v0.2.0**: 主题系统完善
- **v0.3.0**: 性能优化和错误处理
- **v1.0.0**: 稳定版本发布

#### 发布检查清单

- [ ] 所有测试通过
- [ ] 文档完整
- [ ] 性能基准达标
- [ ] 安全审计通过
- [ ] 兼容性测试完成
- [ ] 示例代码验证

## 7. 总结

这个设计文档为微信公众号Rust SDK提供了全面的技术规划。核心设计原则是：

1. **简单易用**: 一行代码完成复杂的发布流程
2. **高性能**: 利用Rust的零成本抽象和异步特性
3. **可靠性**: 完善的错误处理和重试机制
4. **可扩展**: 模块化设计，支持自定义主题和配置

通过分阶段实施，我们将在8周内交付一个稳定、高性能的微信公众号Rust SDK，为Rust生态系统填补这一空白。
