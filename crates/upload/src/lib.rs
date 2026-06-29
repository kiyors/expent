use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::presigning::PresigningConfig;
use bytes::Bytes;
use image::ImageFormat;
use image::imageops::FilterType;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

pub mod optimizer;

pub use optimizer::ImageOptimizer;

#[derive(Debug, Error)]
pub enum UploadError {
    #[error("Failed to process image: {0}")]
    ImageError(#[from] image::ImageError),
    #[error("Unknown file type")]
    UnknownFileType,
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("S3 error: {0}")]
    S3Error(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FileCategory {
    Pdf,
    Image,
    Csv,
    Unknown,
}

/// Configuration for image compression before upload.
#[derive(Debug, Clone)]
pub struct CompressOptions {
    /// WebP quality (1-100). Default: 80
    pub quality: u8,
    /// Maximum width in pixels. Images wider than this get resized proportionally.
    pub max_width: Option<u32>,
    /// Maximum height in pixels. Images taller than this get resized proportionally.
    pub max_height: Option<u32>,
}

impl Default for CompressOptions {
    fn default() -> Self {
        Self {
            quality: 80,
            max_width: None,
            max_height: None,
        }
    }
}

impl CompressOptions {
    /// Preset for avatar images: 512×512 max, 80% quality.
    #[must_use]
    pub fn avatar() -> Self {
        Self {
            quality: 80,
            max_width: Some(512),
            max_height: Some(512),
        }
    }

    /// Preset for receipt/document images: 2048px max dimension, 85% quality.
    #[must_use]
    pub fn receipt() -> Self {
        Self {
            quality: 85,
            max_width: Some(2048),
            max_height: Some(2048),
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct ProcessedFile {
    pub id: Uuid,
    pub original_name: Option<String>,
    pub category: FileCategory,
    pub content_type: String,
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
    pub key: String,
    pub raw_key: Option<String>,
    pub p_hash: Option<String>,
}

#[derive(Clone, Debug)]
pub struct UploadClient {
    s3_client: S3Client,
    bucket_name: String,
    pub optimizer: Arc<ImageOptimizer>,
}

impl UploadClient {
    #[must_use]
    pub fn new(s3_client: S3Client, bucket_name: String) -> Self {
        Self {
            s3_client,
            bucket_name,
            optimizer: Arc::new(ImageOptimizer::new(4)),
        }
    }

    /// Cheap readiness probe — does a `HeadBucket` against the configured S3 bucket.
    /// Used by the `/health/ready` endpoint so the load balancer can drop us out of
    /// rotation if R2/S3 becomes unreachable.
    ///
    /// # Errors
    /// Returns `UploadError::S3Error` if the bucket is missing, unreachable, or
    /// credentials are invalid.
    pub async fn health_check(&self) -> Result<(), UploadError> {
        self.s3_client
            .head_bucket()
            .bucket(&self.bucket_name)
            .send()
            .await
            .map_err(|e| UploadError::S3Error(e.to_string()))?;
        Ok(())
    }

    /// Generates a presigned URL for direct upload.
    ///
    /// # Errors
    /// Returns `UploadError::Internal` if URL presigning fails.
    /// Returns `UploadError::S3Error` if communication with S3 fails.
    pub async fn get_presigned_url(
        &self,
        user_id: &str,
        file_name: &str,
        content_type: &str,
        expires_in: Duration,
    ) -> Result<(String, String), UploadError> {
        let sanitized_name = Path::new(file_name)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed");
        let key = format!("{user_id}/{}-{sanitized_name}", Uuid::now_v7());

        let presigning_config = PresigningConfig::expires_in(expires_in)
            .map_err(|e| UploadError::Internal(e.to_string()))?;

        let presigned_request = self
            .s3_client
            .put_object()
            .bucket(&self.bucket_name)
            .key(&key)
            .content_type(content_type)
            .presigned(presigning_config)
            .await
            .map_err(|e| UploadError::S3Error(format!("{e:#?}")))?;

        Ok((presigned_request.uri().to_string(), key))
    }

    /// Uploads a file directly to S3.
    ///
    /// # Errors
    /// Returns `UploadError::S3Error` if the upload fails.
    /// Returns `UploadError::ImageError` if image processing fails.
    pub async fn upload_direct(
        &self,
        user_id: &str,
        data: Bytes,
        original_name: Option<String>,
        content_type: Option<String>,
        optimize: bool,
    ) -> Result<ProcessedFile, UploadError> {
        let mut final_data = data.clone();
        let mut final_content_type = content_type.clone();
        let mut p_hash = None;
        let mut raw_key = None;

        let category = UploadProcessor::determine_category(
            &final_data,
            original_name.as_deref(),
            content_type.as_deref(),
        );

        if optimize && category == FileCategory::Image {
            // Store raw copy
            let raw_id = Uuid::now_v7();
            let raw_ext = content_type
                .as_ref()
                .and_then(|ct| mime_guess::get_mime_extensions_str(ct))
                .and_then(|exts| exts.first())
                .unwrap_or(&"bin");
            let raw_path = format!("{user_id}/raw/{raw_id}-original.{raw_ext}");

            self.s3_client
                .put_object()
                .bucket(&self.bucket_name)
                .key(&raw_path)
                .content_type(
                    content_type
                        .as_deref()
                        .unwrap_or("application/octet-stream"),
                )
                .body(data.into())
                .send()
                .await
                .map_err(|e| UploadError::S3Error(format!("{e:#?}")))?;

            raw_key = Some(raw_path);

            let (optimized_data, content_type_str, hash) =
                self.optimizer.optimize(final_data, 2048, false).await?;
            final_data = optimized_data;
            final_content_type = Some(content_type_str);
            p_hash = Some(hash);
        }

        let processed =
            UploadProcessor::process(final_data, original_name.clone(), final_content_type, None)?;

        let sanitized_name = original_name
            .as_deref()
            .and_then(|name| Path::new(name).file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed");

        let mut final_name = sanitized_name.to_string();
        if optimize && category == FileCategory::Image {
            // Force .webp extension if optimized
            let base = Path::new(&final_name)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unnamed");
            final_name = format!("{base}.webp");
        }

        let key = format!("{user_id}/{}-{final_name}", processed.id);

        self.s3_client
            .put_object()
            .bucket(&self.bucket_name)
            .key(&key)
            .content_type(&processed.content_type)
            .body(processed.data.clone().into())
            .send()
            .await
            .map_err(|e| UploadError::S3Error(format!("{e:#?}")))?;

        Ok(ProcessedFile {
            id: processed.id,
            original_name: processed.original_name,
            category: processed.category,
            content_type: processed.content_type,
            data: processed.data.to_vec(),
            key,
            raw_key,
            p_hash,
        })
    }

    /// Upload with explicit image compression. All images are compressed to WebP
    /// before being sent to R2, regardless of input format.
    ///
    /// # Errors
    /// Returns `UploadError::S3Error` if the upload fails.
    /// Returns `UploadError::ImageError` if image processing fails.
    pub async fn upload_compressed(
        &self,
        user_id: &str,
        data: Bytes,
        original_name: Option<String>,
        content_type: Option<String>,
        compress_opts: CompressOptions,
    ) -> Result<ProcessedFile, UploadError> {
        let processed = UploadProcessor::process(
            data,
            original_name.clone(),
            content_type.clone(),
            Some(compress_opts),
        )?;

        let sanitized_name = original_name
            .as_deref()
            .and_then(|name| Path::new(name).file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed");
        // For compressed images, replace extension with .webp
        let key_name = if processed.category == FileCategory::Image {
            let base = Path::new(sanitized_name)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unnamed");
            format!("{base}.webp")
        } else {
            sanitized_name.to_string()
        };
        let key = format!("{user_id}/{}-{key_name}", processed.id);

        self.s3_client
            .put_object()
            .bucket(&self.bucket_name)
            .key(&key)
            .content_type(&processed.content_type)
            .body(processed.data.clone().into())
            .send()
            .await
            .map_err(|e| UploadError::S3Error(format!("{e:#?}")))?;

        Ok(ProcessedFile {
            id: processed.id,
            original_name: processed.original_name,
            category: processed.category,
            content_type: processed.content_type,
            data: processed.data.to_vec(),
            key,
            raw_key: None,
            p_hash: None,
        })
    }

    /// Retrieves a file from S3.
    ///
    /// # Errors
    /// Returns `UploadError::S3Error` if retrieval fails.
    pub async fn get_file(&self, key: &str) -> Result<Bytes, UploadError> {
        let response = self
            .s3_client
            .get_object()
            .bucket(&self.bucket_name)
            .key(key)
            .send()
            .await
            .map_err(|e| UploadError::S3Error(format!("{e:#?}")))?;

        let data = response
            .body
            .collect()
            .await
            .map_err(|e| UploadError::Internal(e.to_string()))?
            .into_bytes();

        Ok(data)
    }

    /// Deletes a file from S3.
    ///
    /// # Errors
    /// Returns `UploadError::S3Error` if deletion fails.
    pub async fn delete_file(&self, key: &str) -> Result<(), UploadError> {
        self.s3_client
            .delete_object()
            .bucket(&self.bucket_name)
            .key(key)
            .send()
            .await
            .map_err(|e| UploadError::S3Error(format!("{e:#?}")))?;

        Ok(())
    }
}

pub struct UploadProcessor;

impl UploadProcessor {
    /// Processes a file based on its category and compression options.
    ///
    /// # Errors
    /// Returns `UploadError::ImageError` if image processing fails.
    pub fn process(
        data: Bytes,
        original_name: Option<String>,
        content_type: Option<String>,
        compress_opts: Option<CompressOptions>,
    ) -> Result<RawProcessedFile, UploadError> {
        let category =
            Self::determine_category(&data, original_name.as_deref(), content_type.as_deref());

        let id = Uuid::now_v7();

        // Perform category-specific processing
        match category {
            FileCategory::Image => {
                let (final_data, final_content_type) = if let Some(opts) = compress_opts {
                    // Compress to WebP with the given options
                    let webp_data = Self::compress_to_webp(&data, &opts)?;
                    (webp_data, "image/webp".to_string())
                } else {
                    // No compression — just validate it's a real image
                    let _img = image::load_from_memory(&data)?;
                    (
                        data,
                        content_type.unwrap_or_else(|| "image/png".to_string()),
                    )
                };

                Ok(RawProcessedFile {
                    id,
                    original_name,
                    category,
                    content_type: final_content_type,
                    data: final_data,
                })
            }
            FileCategory::Pdf => Ok(RawProcessedFile {
                id,
                original_name,
                category,
                content_type: "application/pdf".to_string(),
                data,
            }),
            FileCategory::Csv => Ok(RawProcessedFile {
                id,
                original_name,
                category,
                content_type: "text/csv".to_string(),
                data,
            }),
            FileCategory::Unknown => Err(UploadError::UnknownFileType),
        }
    }

    fn determine_category(
        data: &[u8],
        filename: Option<&str>,
        content_type: Option<&str>,
    ) -> FileCategory {
        // 1. Try by content-type
        if let Some(ct) = content_type {
            if ct == "application/pdf" {
                return FileCategory::Pdf;
            }
            if [
                "image/jpeg",
                "image/png",
                "image/webp",
                "image/gif",
                "image/heic",
                "image/heif",
            ]
            .contains(&ct)
            {
                return FileCategory::Image;
            }
            if ct == "text/csv" || ct == "application/csv" {
                return FileCategory::Csv;
            }
        }

        // 2. Try by extension
        if let Some(name) = filename {
            let path = Path::new(name);
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                match ext.to_lowercase().as_str() {
                    "pdf" => return FileCategory::Pdf,
                    "csv" => return FileCategory::Csv,
                    "jpg" | "jpeg" | "png" | "gif" | "webp" | "heic" | "heif" => {
                        return FileCategory::Image;
                    }
                    _ => {}
                }
            }
        }

        // 3. Try by magic bytes
        if let Some(kind) = infer::get(data) {
            match kind.mime_type() {
                "application/pdf" => return FileCategory::Pdf,
                "text/csv" | "application/csv" => return FileCategory::Csv,
                "image/jpeg" | "image/png" | "image/webp" | "image/gif" | "image/heic"
                | "image/heif" => return FileCategory::Image,
                _ => {}
            }
        }

        FileCategory::Unknown
    }

    /// Compress an image to WebP format with optional resizing.
    /// Supports all input formats that the `image` crate handles (PNG, JPEG, GIF, WebP, etc.).
    ///
    /// # Errors
    /// Returns `UploadError::ImageError` if image loading or writing fails.
    pub fn compress_to_webp(data: &[u8], opts: &CompressOptions) -> Result<Bytes, UploadError> {
        let mut img = image::load_from_memory(data)?;

        // Resize if max dimensions are specified
        let needs_resize = opts.max_width.is_some_and(|mw| img.width() > mw)
            || opts.max_height.is_some_and(|mh| img.height() > mh);

        if needs_resize {
            let max_w = opts.max_width.unwrap_or_else(|| img.width());
            let max_h = opts.max_height.unwrap_or_else(|| img.height());
            img = img.resize(max_w, max_h, FilterType::Lanczos3);
        }

        let mut buffer = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buffer, ImageFormat::WebP)?;
        Ok(Bytes::from(buffer.into_inner()))
    }

    /// Legacy: Convert image to PNG (kept for backward compatibility).
    ///
    /// # Errors
    /// Returns `UploadError::ImageError` if image loading or writing fails.
    pub fn convert_to_png(data: &[u8]) -> Result<Bytes, UploadError> {
        let img = image::load_from_memory(data)?;
        let mut buffer = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buffer, ImageFormat::Png)?;
        Ok(Bytes::from(buffer.into_inner()))
    }
}

pub struct RawProcessedFile {
    pub id: Uuid,
    pub original_name: Option<String>,
    pub category: FileCategory,
    pub content_type: String,
    pub data: Bytes,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_category_pdf() {
        let data = b"%PDF-1.4";
        assert_eq!(
            UploadProcessor::determine_category(data, None, None),
            FileCategory::Pdf
        );
    }

    #[test]
    fn test_determine_category_image() {
        // PNG magic bytes
        let data = b"\x89PNG\r\n\x1a\n";
        assert_eq!(
            UploadProcessor::determine_category(data, None, None),
            FileCategory::Image
        );
    }

    #[test]
    fn test_determine_category_csv() {
        assert_eq!(
            UploadProcessor::determine_category(b"col1,col2\nval1,val2", Some("test.csv"), None),
            FileCategory::Csv
        );
        assert_eq!(
            UploadProcessor::determine_category(b"col1,col2\nval1,val2", None, Some("text/csv")),
            FileCategory::Csv
        );
    }

    #[test]
    fn test_determine_category_unknown() {
        assert_eq!(
            UploadProcessor::determine_category(b"unknown data", None, None),
            FileCategory::Unknown
        );
    }

    #[test]
    fn test_process_unknown_file_type_rejected() {
        let result = UploadProcessor::process(
            Bytes::from("unknown data"),
            Some("test.txt".to_string()),
            Some("text/plain".to_string()),
            None,
        );

        assert!(matches!(result, Err(UploadError::UnknownFileType)));
    }

    #[test]
    fn test_s3_key_path_traversal_mitigated() {
        let user_id = "user123";
        let file_name = "../dangerous.txt";
        let id = Uuid::now_v7();

        // Simulate the fixed key generation
        let sanitized_name = Path::new(file_name)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed");
        let key = format!("{user_id}/{id}-{sanitized_name}");

        // The key should NO LONGER contain "../"
        assert!(!key.contains("../"));
        assert_eq!(key, format!("user123/{id}-dangerous.txt"));
    }
}
