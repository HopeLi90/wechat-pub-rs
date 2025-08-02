# WeChat Official Account SDK Analysis

## Executive Summary

This analysis examines the WeChat Go SDK implementation to understand the architecture and API patterns for image uploads and draft article management, which will inform the Rust implementation design.

## Core Architecture Overview

### Authentication System
- **Access Token Management**: Central authentication via `access_token` parameter
- **Token Types**: Standard and stable access tokens with different caching strategies
- **Cache Integration**: Token caching with expiration (expires_in - 1500 seconds safety margin)
- **Thread Safety**: Mutex-protected token refresh to prevent concurrent API calls

### HTTP Client Foundation
- **Base URL Pattern**: All APIs use `https://api.weixin.qq.com/cgi-bin/` prefix
- **Request Format**: Consistent pattern of `{endpoint}?access_token={token}` 
- **Content Types**: JSON for metadata, multipart/form-data for file uploads
- **Error Handling**: Structured error responses with `errcode` and `errmsg` fields

## Image Upload Implementation

### 1. Temporary Media Upload (`MediaUpload`)
**Endpoint**: `https://api.weixin.qq.com/cgi-bin/media/upload`

**Request Parameters**:
- `access_token`: Authentication token (query parameter)
- `type`: Media type (`image`, `voice`, `video`, `thumb`)
- File data: Multipart form field named "media"

**Response Structure**:
```json
{
  "type": "image",
  "media_id": "MEDIA_ID", 
  "created_at": 1234567890
}
```

**Key Implementation Details**:
- Uses `util.PostFile()` for file upload via multipart/form-data
- Supports both file path and `io.Reader` input sources
- Media IDs are temporary (3 days retention)
- File size limits: images ≤10MB, voice ≤2MB, video ≤10MB, thumb ≤64KB

### 2. Permanent Material Upload (`AddMaterial`)
**Endpoint**: `https://api.weixin.qq.com/cgi-bin/material/add_material`

**Request Parameters**:
- `access_token`: Authentication token (query parameter)
- `type`: Material type parameter
- File data: Multipart form field named "media"

**Response Structure**:
```json
{
  "media_id": "PERMANENT_MEDIA_ID",
  "url": "http://mmbiz.qpic.cn/..."
}
```

**Key Implementation Details**:
- Returns both `media_id` and permanent `url`
- Video uploads require additional metadata (title, introduction)
- Uses `util.PostMultipartForm()` for complex multipart requests
- Permanent storage with unlimited retention

### 3. Direct Image Upload (`ImageUpload`)
**Endpoint**: `https://api.weixin.qq.com/cgi-bin/media/uploadimg`

**Purpose**: Upload images for use in article content (HTML `<img>` tags)

**Response Structure**:
```json
{
  "url": "http://mmbiz.qpic.cn/direct_image_url"
}
```

**Key Implementation Details**:
- Returns direct URL for embedding in article content
- No media_id returned - URL is the primary identifier
- Used specifically for article content images

## Draft Article Management

### 1. Article Data Structure
```go
type Article struct {
    Title              string `json:"title"`                 // Article title
    Author             string `json:"author"`                // Author name
    Digest             string `json:"digest"`                // Article summary
    Content            string `json:"content"`               // HTML content (<2万 characters, <1MB)
    ContentSourceURL   string `json:"content_source_url"`    // "Read more" URL
    ThumbMediaID       string `json:"thumb_media_id"`        // Cover image media_id (permanent)
    ShowCoverPic       uint   `json:"show_cover_pic"`        // Display cover (0/1)
    NeedOpenComment    uint   `json:"need_open_comment"`     // Enable comments (0/1)
    OnlyFansCanComment uint   `json:"only_fans_can_comment"` // Fans-only comments (0/1)
}
```

### 2. Draft Operations

#### Create Draft (`AddDraft`)
**Endpoint**: `https://api.weixin.qq.com/cgi-bin/draft/add`
**Method**: POST with JSON body

**Request Body**:
```json
{
  "articles": [Article, ...]
}
```

**Response**:
```json
{
  "media_id": "DRAFT_MEDIA_ID"
}
```

#### Retrieve Draft (`GetDraft`)
**Endpoint**: `https://api.weixin.qq.com/cgi-bin/draft/get`

**Request Body**:
```json
{
  "media_id": "DRAFT_MEDIA_ID"
}
```

**Response**:
```json
{
  "news_item": [Article, ...]
}
```

#### Update Draft (`UpdateDraft`)
**Endpoint**: `https://api.weixin.qq.com/cgi-bin/draft/update`

**Request Body**:
```json
{
  "media_id": "DRAFT_MEDIA_ID",
  "index": 0,
  "articles": Article
}
```

#### Delete Draft (`DeleteDraft`)
**Endpoint**: `https://api.weixin.qq.com/cgi-bin/draft/delete`

**Request Body**:
```json
{
  "media_id": "DRAFT_MEDIA_ID"
}
```

#### List Drafts (`PaginateDraft`)
**Endpoint**: `https://api.weixin.qq.com/cgi-bin/draft/batchget`

**Request Body**:
```json
{
  "offset": 0,
  "count": 20,
  "no_content": false
}
```

**Response Structure**:
```json
{
  "total_count": 100,
  "item_count": 20,
  "item": [
    {
      "media_id": "DRAFT_MEDIA_ID",
      "content": {
        "news_item": [Article, ...]
      },
      "update_time": 1234567890
    }
  ]
}
```

## Data Flow Patterns

### Typical Article Creation Workflow
1. **Upload Cover Image**: `AddMaterial` → get permanent `media_id`
2. **Upload Content Images**: `ImageUpload` → get URLs for HTML content
3. **Create Draft**: `AddDraft` with article data including cover `media_id`
4. **Update/Edit**: `UpdateDraft` as needed
5. **Publish**: Use publishing APIs (not in draft module)

### Error Handling Pattern
All APIs return consistent error structure:
```json
{
  "errcode": 40001,
  "errmsg": "invalid credential"
}
```

Success responses have `errcode: 0` or omit error fields entirely.

## Key Technical Insights for Rust Implementation

### 1. HTTP Client Requirements
- **Multipart Form Support**: Essential for file uploads
- **JSON Serialization**: All metadata exchanges use JSON
- **Query Parameter Handling**: Access tokens always in query string
- **Stream Processing**: Support for `io.Reader` equivalent (Rust streams)

### 2. Authentication Architecture
- **Token Caching Layer**: Implement with configurable cache backend
- **Automatic Refresh**: Handle token expiration transparently
- **Thread Safety**: Concurrent access protection for token refresh
- **Multiple Token Types**: Support both standard and stable tokens

### 3. Error Handling Strategy
- **Structured Errors**: Parse `errcode`/`errmsg` from all responses
- **API-Specific Context**: Include operation name in error context
- **HTTP Status Codes**: Check both HTTP status and WeChat error codes

### 4. Data Validation Requirements
- **Content Limits**: Enforce <20k characters, <1MB for article content
- **Media Type Validation**: Validate file types and sizes before upload
- **Required Fields**: Enforce mandatory fields like `thumb_media_id` for articles

### 5. Async/Streaming Considerations
- **File Upload Streaming**: Support large file uploads without full memory buffering
- **Concurrent Operations**: Handle multiple uploads/draft operations simultaneously
- **Rate Limiting**: Implement WeChat API rate limit compliance

## Recommended Rust Architecture

### Core Traits
```rust
pub trait AccessTokenProvider {
    async fn get_access_token(&self) -> Result<String, Error>;
}

pub trait MediaUploader {
    async fn upload_temp_media(&self, media_type: MediaType, data: impl AsyncRead) -> Result<MediaResponse, Error>;
    async fn upload_permanent_material(&self, media_type: MediaType, data: impl AsyncRead) -> Result<MaterialResponse, Error>;
    async fn upload_image(&self, data: impl AsyncRead) -> Result<ImageResponse, Error>;
}

pub trait DraftManager {
    async fn create_draft(&self, articles: Vec<Article>) -> Result<String, Error>;
    async fn get_draft(&self, media_id: &str) -> Result<Vec<Article>, Error>;
    async fn update_draft(&self, media_id: &str, index: usize, article: Article) -> Result<(), Error>;
    async fn delete_draft(&self, media_id: &str) -> Result<(), Error>;
    async fn list_drafts(&self, offset: u32, count: u32, no_content: bool) -> Result<DraftList, Error>;
}
```

### HTTP Client Configuration
- Use `reqwest` with multipart support
- Implement retry logic with exponential backoff
- Configure appropriate timeouts for large file uploads
- Support custom CA certificates for enterprise environments

### Caching Strategy
- Abstract cache trait for different backends (Redis, in-memory, file-based)
- Implement cache key namespacing for multi-tenant scenarios
- Handle cache serialization/deserialization efficiently

This analysis provides the foundation for implementing a robust WeChat Official Account SDK in Rust that maintains compatibility with the established API patterns while leveraging Rust's safety and performance advantages.