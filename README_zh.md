# wechat-pub-rs

![构建状态](https://github.com/tyrchen/wechat-pub-rs/workflows/CI/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/wechat-pub-rs.svg)](https://crates.io/crates/wechat-pub-rs)
[![文档](https://docs.rs/wechat-pub-rs/badge.svg)](https://docs.rs/wechat-pub-rs)
[![许可证: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

简洁、高性能的微信公众号 Rust SDK，专注于文章上传和草稿管理功能。

## 特性

- **简洁 API**：一个函数完成整个文章上传流程：`client.upload("./article.md").await?`
- **智能去重**：
  - 基于 BLAKE3 内容哈希的图片去重
  - 基于标题的草稿去重（更新已存在的草稿）
- **可靠性**：全面的错误处理和重试机制
- **高性能**：异步/等待模式，支持并发图片上传
- **类型安全**：编译时保证和运行时可靠性
- **主题系统**：内置主题，支持语法高亮
- **Markdown 支持**：完整的 Markdown 解析，支持 frontmatter

## 快速开始

将此依赖添加到您的 `Cargo.toml`：

```toml
[dependencies]
wechat-pub-rs = "0.2"
tokio = { version = "1.0", features = ["full"] }
```

### 基础用法

```rust
use wechat_pub_rs::{WeChatClient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // 使用您的微信公众号凭据创建客户端
    let client = WeChatClient::new("your_app_id", "your_app_secret").await?;
    
    // 上传 Markdown 文件（主题和元数据从 frontmatter 读取）
    let draft_id = client.upload("./article.md").await?;
    
    println!("创建草稿，ID: {}", draft_id);
    Ok(())
}
```

### 高级用法

```rust
use wechat_pub_rs::{WeChatClient, UploadOptions, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = WeChatClient::new("your_app_id", "your_app_secret").await?;
    
    // 使用自定义选项上传
    let options = UploadOptions::with_theme("lapis")
        .title("自定义标题")
        .author("自定义作者")
        .cover_image("./cover.jpg")
        .show_cover(true)
        .comments(true, false)
        .source_url("https://example.com");
    
    let draft_id = client.upload_with_options("./article.md", options).await?;
    
    // 管理草稿
    let draft_info = client.get_draft(&draft_id).await?;
    client.update_draft(&draft_id, "./updated_article.md").await?;
    
    // 列出所有草稿
    let drafts = client.list_drafts(0, 10).await?;
    
    println!("创建草稿: {}", draft_id);
    Ok(())
}
```

## Markdown 格式

您的 Markdown 文件应包含带有元数据的 frontmatter：

```markdown
---
title: "您的文章标题"
author: "作者姓名"
cover: "images/cover.jpg"    # 必需：封面图片路径
theme: "lapis"               # 可选：主题名称
code: "github"               # 可选：代码高亮主题
---

# 您的文章内容

在这里编写您的 Markdown 内容，包含图片：

![替代文本](images/example.jpg)

## 代码块

```rust
fn main() {
    println!("你好，微信！");
}
```
```

## 可用主题

| 主题 | 描述 |
|------|------|
| `default` | 简洁的默认主题 |
| `lapis` | 蓝色调优雅样式 |
| `maize` | 温暖的黄色调 |
| `orangeheart` | 橙色调 |
| `phycat` | 独特的 phycat 样式 |
| `pie` | 甜美的饼图主题 |
| `purple` | 紫色调 |
| `rainbow` | 彩虹多彩主题 |

## 代码高亮主题

| 主题 | 描述 |
|------|------|
| `github` | GitHub 样式（浅色） |
| `github-dark` | GitHub 样式（深色） |
| `vscode` | VS Code 样式 |
| `atom-one-light` | Atom One Light |
| `atom-one-dark` | Atom One Dark |
| `solarized-light` | Solarized Light |
| `solarized-dark` | Solarized Dark |
| `monokai` | Monokai |
| `dracula` | Dracula |
| `xcode` | Xcode |

## API 参考

### WeChatClient

#### 主要方法

```rust
// 创建新客户端
pub async fn new(app_id: impl Into<String>, app_secret: impl Into<String>) -> Result<Self>

// 上传 Markdown 文件
pub async fn upload(&self, markdown_path: &str) -> Result<String>

// 使用自定义选项上传
pub async fn upload_with_options(&self, markdown_path: &str, options: UploadOptions) -> Result<String>
```

#### 草稿管理

```rust
// 获取草稿信息
pub async fn get_draft(&self, media_id: &str) -> Result<DraftInfo>

// 更新现有草稿
pub async fn update_draft(&self, media_id: &str, markdown_path: &str) -> Result<()>

// 删除草稿
pub async fn delete_draft(&self, media_id: &str) -> Result<()>

// 分页列出草稿
pub async fn list_drafts(&self, offset: u32, count: u32) -> Result<Vec<DraftInfo>>
```

#### 实用方法

```rust
// 上传单个图片
pub async fn upload_image(&self, image_path: &str) -> Result<String>

// 获取可用主题
pub fn available_themes(&self) -> Vec<&String>

// 检查主题是否存在
pub fn has_theme(&self, theme: &str) -> bool

// 获取令牌信息（用于调试）
pub async fn get_token_info(&self) -> Option<TokenInfo>
```

### UploadOptions

```rust
pub struct UploadOptions {
    pub theme: String,                    // 主题名称
    pub title: Option<String>,            // 自定义标题
    pub author: Option<String>,           // 自定义作者
    pub cover_image: Option<String>,      // 封面图片路径
    pub show_cover: bool,                 // 在内容中显示封面
    pub enable_comments: bool,            // 启用评论
    pub fans_only_comments: bool,         // 仅粉丝可评论
    pub source_url: Option<String>,       // 原文链接
}
```

构建器方法：

```rust
UploadOptions::with_theme("lapis")
    .title("自定义标题")
    .author("作者")
    .cover_image("cover.jpg")
    .show_cover(true)
    .comments(true, false)
    .source_url("https://example.com")
```

## 环境变量

运行示例时，请设置以下环境变量：

```bash
export WECHAT_APP_ID="你的微信应用ID"
export WECHAT_APP_SECRET="你的微信应用密钥"
```

## 错误处理

库提供了全面的错误处理：

```rust
use wechat_pub_rs::{WeChatError, Result};

match client.upload("article.md").await {
    Ok(draft_id) => println!("成功: {}", draft_id),
    Err(WeChatError::FileNotFound { path }) => {
        eprintln!("文件未找到: {}", path);
    }
    Err(WeChatError::ThemeNotFound { theme }) => {
        eprintln!("主题未找到: {}", theme);
    }
    Err(WeChatError::Network(err)) => {
        eprintln!("网络错误: {}", err);
    }
    Err(err) => eprintln!("其他错误: {}", err),
}
```

## 性能

- **并发上传**：图片并发上传（最多 5 个并发）
- **去重**：使用 BLAKE3 哈希进行图片去重
- **内存高效**：流式文件操作
- **异步**：全程非阻塞操作

## 系统要求

- Rust 1.70+
- 具有 API 访问权限的微信公众号
- 有效的 App ID 和 App Secret

## 示例

查看 `examples/` 目录获取更多使用示例：

```bash
# 运行简单示例
WECHAT_APP_ID=your_id WECHAT_APP_SECRET=your_secret cargo run --example simple
```

## 贡献

1. Fork 仓库
2. 创建您的功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交您的更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 开启 Pull Request

## 测试

```bash
# 运行单元测试
cargo test

# 运行带输出的测试
cargo test -- --nocapture

# 运行特定测试
cargo test test_name
```

## 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE.md](LICENSE.md) 文件。

## 更新日志

查看 [CHANGELOG.md](CHANGELOG.md) 了解变更和版本历史。

---

**注意**：此 SDK 专注于微信公众号的文章发布功能。对于其他微信功能（消息、用户管理等），请考虑使用更全面的微信 SDK。