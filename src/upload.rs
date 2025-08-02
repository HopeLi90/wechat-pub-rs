//! Upload module for handling image uploads and draft management.

use crate::auth::TokenManager;
use crate::error::{Result, WeChatError};
use crate::http::{
    DraftResponse, ImageUploadResponse, MaterialUploadResponse, WeChatHttpClient, WeChatResponse,
};
use crate::markdown::ImageRef;
use futures::future::try_join_all;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Semaphore;
use blake3;

/// Maximum concurrent image uploads to prevent overwhelming the server
const MAX_CONCURRENT_UPLOADS: usize = 5;

/// Represents the result of an image upload operation.
#[derive(Debug, Clone)]
pub struct UploadResult {
    /// Original image reference
    pub image_ref: ImageRef,
    /// WeChat media ID for the uploaded image
    pub media_id: String,
    /// WeChat URL for the uploaded image
    pub url: String,
}

/// Represents a WeChat article for draft creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    /// Article title
    pub title: String,
    /// Article author
    pub author: String,
    /// Article content (HTML)
    pub content: String,
    /// Content source URL (optional)
    pub content_source_url: Option<String>,
    /// Digest (summary) of the article
    pub digest: String,
    /// Show cover picture in content (0: no, 1: yes)
    pub show_cover_pic: u8,
    /// Thumb media ID for cover image
    pub thumb_media_id: Option<String>,
    /// Need open comment (0: no, 1: yes)
    pub need_open_comment: u8,
    /// Only fans can comment (0: no, 1: yes)
    pub only_fans_can_comment: u8,
}

impl Article {
    /// Creates a new article with required fields.
    pub fn new(title: String, author: String, content: String) -> Self {
        Self {
            title,
            author,
            content,
            content_source_url: None,
            digest: String::new(),
            show_cover_pic: 1,
            thumb_media_id: None,
            need_open_comment: 0,
            only_fans_can_comment: 0,
        }
    }

    /// Sets the article digest (summary).
    pub fn with_digest(mut self, digest: String) -> Self {
        self.digest = digest;
        self
    }

    /// Sets the cover image media ID.
    pub fn with_cover_image(mut self, thumb_media_id: String) -> Self {
        self.thumb_media_id = Some(thumb_media_id);
        self
    }

    /// Sets whether to show cover image in content.
    pub fn with_show_cover(mut self, show: bool) -> Self {
        self.show_cover_pic = if show { 1 } else { 0 };
        self
    }

    /// Sets comment settings.
    pub fn with_comments(mut self, enable_comments: bool, fans_only: bool) -> Self {
        self.need_open_comment = if enable_comments { 1 } else { 0 };
        self.only_fans_can_comment = if fans_only { 1 } else { 0 };
        self
    }

    /// Sets the content source URL.
    pub fn with_source_url(mut self, url: String) -> Self {
        self.content_source_url = Some(url);
        self
    }
}

/// Request body for creating a draft.
#[derive(Debug, Serialize)]
struct DraftRequest {
    articles: Vec<Article>,
}

/// Draft information from WeChat API.
#[derive(Debug, Deserialize)]
pub struct DraftInfo {
    pub media_id: String,
    pub content: DraftContent,
    pub update_time: u64,
}

/// Content of a draft.
#[derive(Debug, Deserialize)]
pub struct DraftContent {
    pub news_item: Vec<Article>,
}

/// List drafts response.
#[derive(Debug, Deserialize)]
pub struct DraftListResponse {
    pub total_count: u32,
    pub item_count: u32,
    pub item: Vec<DraftInfo>,
}

/// Material item in the list response.
#[derive(Debug, Deserialize)]
pub struct MaterialItem {
    pub media_id: String,
    pub name: String,
    pub update_time: u64,
    pub url: String,
}

/// List materials response.
#[derive(Debug, Deserialize)]
pub struct MaterialListResponse {
    pub total_count: u32,
    pub item_count: u32,
    pub item: Vec<MaterialItem>,
}

/// Image uploader with concurrent upload capabilities.
#[derive(Debug)]
pub struct ImageUploader {
    http_client: Arc<WeChatHttpClient>,
    token_manager: Arc<TokenManager>,
    semaphore: Arc<Semaphore>,
}

impl ImageUploader {
    /// Creates a new image uploader.
    pub fn new(http_client: Arc<WeChatHttpClient>, token_manager: Arc<TokenManager>) -> Self {
        Self {
            http_client,
            token_manager,
            semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_UPLOADS)),
        }
    }

    /// Uploads multiple images concurrently.
    pub async fn upload_images(
        &self,
        images: Vec<ImageRef>,
        base_path: &Path,
    ) -> Result<Vec<UploadResult>> {
        if images.is_empty() {
            return Ok(Vec::new());
        }

        log::info!("Uploading {} images concurrently", images.len());

        // Create upload tasks
        let tasks: Vec<_> = images
            .into_iter()
            .map(|image_ref| {
                let uploader = self.clone();
                let base_path = base_path.to_owned();

                tokio::spawn(
                    async move { uploader.upload_single_image(image_ref, &base_path).await },
                )
            })
            .collect();

        // Execute all tasks and collect results
        let results = try_join_all(tasks)
            .await
            .map_err(|e| WeChatError::Internal(anyhow::anyhow!("Task join error: {}", e)))?;

        // Convert task results to upload results
        let upload_results: Result<Vec<_>> = results.into_iter().collect();
        let uploads = upload_results?;

        log::info!("Successfully uploaded {} images", uploads.len());
        Ok(uploads)
    }

    /// Uploads a single image.
    async fn upload_single_image(
        &self,
        image_ref: ImageRef,
        base_path: &Path,
    ) -> Result<UploadResult> {
        // Acquire semaphore permit to limit concurrency
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| WeChatError::Internal(anyhow::anyhow!("Semaphore error: {}", e)))?;

        log::debug!("Processing image: {}", image_ref.original_url);

        // Load image data
        let image_data = if image_ref.is_local {
            let image_path = image_ref.resolve_path(base_path);
            self.load_local_image(&image_path).await?
        } else {
            self.download_remote_image(&image_ref.original_url).await?
        };

        // Calculate BLAKE3 hash of the image content
        let hash = blake3::hash(&image_data);
        let hash_str = hash.to_hex().to_string();
        log::debug!("Image hash: {}", hash_str);

        // Check if this image already exists by searching materials
        if let Some(existing_url) = self.find_existing_image_by_hash(&hash_str).await? {
            log::info!(
                "Image already exists with hash {}, reusing URL: {}",
                hash_str,
                existing_url
            );
            
            // Extract media_id from URL (format: https://mmbiz.qpic.cn/mmbiz_jpg/{media_id}/0)
            let media_id = existing_url
                .strip_prefix("https://mmbiz.qpic.cn/mmbiz_jpg/")
                .and_then(|s| s.strip_suffix("/0"))
                .unwrap_or(&hash_str)
                .to_string();

            return Ok(UploadResult {
                image_ref,
                media_id,
                url: existing_url,
            });
        }

        // Use hash as filename with appropriate extension
        let extension = self.get_image_extension(&image_ref.original_url, &image_data);
        let filename = format!("{}.{}", hash_str, extension);
        log::debug!("Uploading new image with filename: {}", filename);

        // Upload to WeChat
        let access_token = self.token_manager.get_access_token().await?;
        let response = self
            .http_client
            .upload_file(
                "/cgi-bin/media/uploadimg",
                &access_token,
                "media",
                image_data,
                &filename,
            )
            .await?;

        // Parse response
        let upload_response: WeChatResponse<ImageUploadResponse> = response.json().await?;
        let image_upload = upload_response.into_result()?;

        // WeChat returns the image URL in a different format for permanent images
        let url = format!(
            "https://mmbiz.qpic.cn/mmbiz_jpg/{}/0",
            image_upload.media_id
        );

        log::info!(
            "Successfully uploaded new image: {} -> {} (hash: {})",
            image_ref.original_url,
            url,
            hash_str
        );

        Ok(UploadResult {
            image_ref,
            media_id: image_upload.media_id,
            url,
        })
    }

    /// Loads image data from local file.
    async fn load_local_image(&self, path: &Path) -> Result<Vec<u8>> {
        fs::read(path).await.map_err(|e| WeChatError::ImageUpload {
            path: path.display().to_string(),
            reason: format!("Failed to read local file: {e}"),
        })
    }

    /// Downloads image data from remote URL.
    async fn download_remote_image(&self, url: &str) -> Result<Vec<u8>> {
        self.http_client
            .download(url)
            .await
            .map_err(|e| WeChatError::ImageUpload {
                path: url.to_string(),
                reason: format!("Failed to download remote image: {e}"),
            })
    }


    /// Gets the image extension based on URL and content.
    fn get_image_extension(&self, url: &str, image_data: &[u8]) -> String {
        // First try to get from URL
        if let Some(ext) = Path::new(url)
            .extension()
            .and_then(|e| e.to_str())
            .filter(|e| matches!(*e, "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp"))
        {
            return ext.to_string();
        }

        // Otherwise, detect from content
        if image_data.len() >= 4 {
            match &image_data[0..4] {
                [0xFF, 0xD8, 0xFF, _] => return "jpg".to_string(),
                [0x89, 0x50, 0x4E, 0x47] => return "png".to_string(),
                [0x47, 0x49, 0x46, _] => return "gif".to_string(),
                [0x42, 0x4D, _, _] => return "bmp".to_string(),
                _ => {}
            }
        }
        
        // Check for WebP
        if image_data.len() >= 12 && &image_data[0..4] == b"RIFF" && &image_data[8..12] == b"WEBP" {
            return "webp".to_string();
        }

        // Default to jpg
        "jpg".to_string()
    }

    /// Searches for an existing image by its hash in recent materials.
    async fn find_existing_image_by_hash(&self, hash: &str) -> Result<Option<String>> {
        log::debug!("Checking for existing image with hash: {}", hash);
        
        // Check the most recent 20 materials
        let access_token = self.token_manager.get_access_token().await?;
        
        let request = serde_json::json!({
            "type": "image",
            "offset": 0,
            "count": 20
        });
        
        let response = self
            .http_client
            .post_json_with_token("/cgi-bin/material/batchget_material", &access_token, &request)
            .await
            .map_err(|e| {
                log::warn!("Failed to list materials: {}", e);
                e
            });
        
        // If we can't list materials, just proceed with upload
        let response = match response {
            Ok(resp) => resp,
            Err(_) => return Ok(None),
        };
        
        let materials_result = response.json::<WeChatResponse<MaterialListResponse>>().await;
        
        match materials_result {
            Ok(materials_response) => {
                if let Ok(material_list) = materials_response.into_result() {
                    // Check if any material name starts with our hash
                    for item in material_list.item {
                        if item.name.starts_with(hash) {
                            log::info!("Found existing image with hash {}: {}", hash, item.url);
                            return Ok(Some(item.url));
                        }
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to parse material list response: {}", e);
            }
        }
        
        log::debug!("No existing image found with hash: {}", hash);
        Ok(None)
    }

    /// Uploads a cover image as permanent material.
    pub async fn upload_cover_material(&self, cover_path: &Path) -> Result<String> {
        log::info!(
            "Uploading cover image as permanent material: {}",
            cover_path.display()
        );

        // Load image data
        let image_data = self.load_local_image(cover_path).await?;
        
        // Calculate BLAKE3 hash for the cover image
        let hash = blake3::hash(&image_data);
        let hash_str = hash.to_hex().to_string();
        
        // Use hash as filename with appropriate extension
        let extension = self.get_image_extension(&cover_path.to_string_lossy(), &image_data);
        let filename = format!("{}.{}", hash_str, extension);
        log::debug!("Uploading cover image with hash: {}", hash_str);

        // Upload as permanent material
        let access_token = self.token_manager.get_access_token().await?;
        let response = self
            .http_client
            .upload_material(&access_token, "image", image_data, &filename)
            .await?;

        // Parse response
        let upload_response: WeChatResponse<MaterialUploadResponse> = response.json().await?;
        let material = upload_response.into_result()?;

        log::info!(
            "Successfully uploaded cover image: {} -> media_id: {}",
            cover_path.display(),
            material.media_id
        );

        Ok(material.media_id)
    }
}

impl Clone for ImageUploader {
    fn clone(&self) -> Self {
        Self {
            http_client: Arc::clone(&self.http_client),
            token_manager: Arc::clone(&self.token_manager),
            semaphore: Arc::clone(&self.semaphore),
        }
    }
}

/// Draft manager for creating and managing article drafts.
#[derive(Debug)]
pub struct DraftManager {
    http_client: Arc<WeChatHttpClient>,
    token_manager: Arc<TokenManager>,
}

impl DraftManager {
    /// Creates a new draft manager.
    pub fn new(http_client: Arc<WeChatHttpClient>, token_manager: Arc<TokenManager>) -> Self {
        Self {
            http_client,
            token_manager,
        }
    }

    /// Creates a new draft with articles.
    pub async fn create_draft(&self, articles: Vec<Article>) -> Result<String> {
        if articles.is_empty() {
            return Err(WeChatError::config_error(
                "At least one article is required",
            ));
        }

        log::info!("Creating draft with {} articles", articles.len());

        let request = DraftRequest { articles };
        let access_token = self.token_manager.get_access_token().await?;

        let response = self
            .http_client
            .post_json_with_token("/cgi-bin/draft/add", &access_token, &request)
            .await?;

        let draft_response: WeChatResponse<DraftResponse> = response.json().await?;
        let draft = draft_response.into_result()?;

        log::info!(
            "Successfully created draft with media_id: {}",
            draft.media_id
        );
        Ok(draft.media_id)
    }

    /// Gets a draft by media ID.
    pub async fn get_draft(&self, media_id: &str) -> Result<DraftInfo> {
        log::debug!("Getting draft: {media_id}");

        let access_token = self.token_manager.get_access_token().await?;
        let request = serde_json::json!({ "media_id": media_id });

        let response = self
            .http_client
            .post_json_with_token("/cgi-bin/draft/get", &access_token, &request)
            .await?;

        let draft_response: WeChatResponse<DraftInfo> = response.json().await?;
        draft_response.into_result()
    }

    /// Updates a draft.
    pub async fn update_draft(&self, media_id: &str, articles: Vec<Article>) -> Result<()> {
        if articles.is_empty() {
            return Err(WeChatError::config_error(
                "At least one article is required",
            ));
        }

        log::info!(
            "Updating draft {} with {} articles",
            media_id,
            articles.len()
        );

        let request = serde_json::json!({
            "media_id": media_id,
            "index": 0,
            "articles": articles
        });

        let access_token = self.token_manager.get_access_token().await?;

        let response = self
            .http_client
            .post_json_with_token("/cgi-bin/draft/update", &access_token, &request)
            .await?;

        let update_response: WeChatResponse<serde_json::Value> = response.json().await?;
        update_response.into_result()?;

        log::info!("Successfully updated draft: {media_id}");
        Ok(())
    }

    /// Deletes a draft.
    pub async fn delete_draft(&self, media_id: &str) -> Result<()> {
        log::info!("Deleting draft: {media_id}");

        let request = serde_json::json!({ "media_id": media_id });
        let access_token = self.token_manager.get_access_token().await?;

        let response = self
            .http_client
            .post_json_with_token("/cgi-bin/draft/delete", &access_token, &request)
            .await?;

        let delete_response: WeChatResponse<serde_json::Value> = response.json().await?;
        delete_response.into_result()?;

        log::info!("Successfully deleted draft: {media_id}");
        Ok(())
    }

    /// Lists drafts with pagination.
    pub async fn list_drafts(&self, offset: u32, count: u32) -> Result<Vec<DraftInfo>> {
        log::debug!("Listing drafts: offset={offset}, count={count}");

        let request = serde_json::json!({
            "offset": offset,
            "count": count,
            "no_content": 0
        });

        let access_token = self.token_manager.get_access_token().await?;

        let response = self
            .http_client
            .post_json_with_token("/cgi-bin/draft/batchget", &access_token, &request)
            .await?;

        let list_response: WeChatResponse<DraftListResponse> = response.json().await?;
        let drafts = list_response.into_result()?;

        log::debug!("Found {} drafts", drafts.item.len());
        Ok(drafts.item)
    }

    /// Creates URL mapping from upload results.
    pub fn create_url_mapping(&self, upload_results: &[UploadResult]) -> HashMap<String, String> {
        upload_results
            .iter()
            .map(|result| (result.image_ref.original_url.clone(), result.url.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::TokenManager;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_article_creation() {
        let article = Article::new(
            "Test Title".to_string(),
            "Test Author".to_string(),
            "<h1>Test Content</h1>".to_string(),
        );

        assert_eq!(article.title, "Test Title");
        assert_eq!(article.author, "Test Author");
        assert_eq!(article.content, "<h1>Test Content</h1>");
        assert_eq!(article.show_cover_pic, 1);
        assert_eq!(article.need_open_comment, 0);
    }

    #[tokio::test]
    async fn test_article_builder_methods() {
        let article = Article::new(
            "Title".to_string(),
            "Author".to_string(),
            "Content".to_string(),
        )
        .with_digest("Test digest".to_string())
        .with_cover_image("cover_media_id".to_string())
        .with_show_cover(false)
        .with_comments(true, true)
        .with_source_url("https://example.com".to_string());

        assert_eq!(article.digest, "Test digest");
        assert_eq!(article.thumb_media_id, Some("cover_media_id".to_string()));
        assert_eq!(article.show_cover_pic, 0);
        assert_eq!(article.need_open_comment, 1);
        assert_eq!(article.only_fans_can_comment, 1);
        assert_eq!(
            article.content_source_url,
            Some("https://example.com".to_string())
        );
    }

    #[tokio::test]
    async fn test_image_uploader_creation() {
        let http_client = Arc::new(WeChatHttpClient::new().unwrap());
        let token_manager = Arc::new(TokenManager::new(
            "test_app_id",
            "test_secret",
            Arc::clone(&http_client),
        ));

        let uploader = ImageUploader::new(http_client, token_manager);
        assert_eq!(
            uploader.semaphore.available_permits(),
            MAX_CONCURRENT_UPLOADS
        );
    }

    #[tokio::test]
    async fn test_draft_manager_creation() {
        let http_client = Arc::new(WeChatHttpClient::new().unwrap());
        let token_manager = Arc::new(TokenManager::new(
            "test_app_id",
            "test_secret",
            Arc::clone(&http_client),
        ));

        let _manager = DraftManager::new(http_client, token_manager);
        // Just test that creation works
    }

    #[test]
    fn test_image_extension_detection() {
        let http_client = Arc::new(WeChatHttpClient::new().unwrap());
        let token_manager = Arc::new(TokenManager::new(
            "test_app_id",
            "test_secret",
            Arc::clone(&http_client),
        ));
        let uploader = ImageUploader::new(http_client, token_manager);

        // Test URL-based extension detection
        assert_eq!(uploader.get_image_extension("test.jpg", &[]), "jpg");
        assert_eq!(uploader.get_image_extension("test.png", &[]), "png");
        assert_eq!(uploader.get_image_extension("test.webp", &[]), "webp");
        
        // Test content-based detection for JPEG
        let jpeg_header = vec![0xFF, 0xD8, 0xFF, 0xE0];
        assert_eq!(uploader.get_image_extension("noext", &jpeg_header), "jpg");
        
        // Test content-based detection for PNG
        let png_header = vec![0x89, 0x50, 0x4E, 0x47];
        assert_eq!(uploader.get_image_extension("noext", &png_header), "png");
    }

    #[test]
    fn test_url_mapping_creation() {
        let http_client = Arc::new(WeChatHttpClient::new().unwrap());
        let token_manager = Arc::new(TokenManager::new(
            "test_app_id",
            "test_secret",
            Arc::clone(&http_client),
        ));
        let manager = DraftManager::new(http_client, token_manager);

        let image_ref = ImageRef::new("Alt".to_string(), "./test.jpg".to_string(), (0, 10));
        let upload_result = UploadResult {
            image_ref,
            media_id: "media123".to_string(),
            url: "https://wechat.com/image123".to_string(),
        };

        let mapping = manager.create_url_mapping(&[upload_result]);
        assert_eq!(
            mapping.get("./test.jpg"),
            Some(&"https://wechat.com/image123".to_string())
        );
    }
}
