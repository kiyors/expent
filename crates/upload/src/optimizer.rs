use crate::UploadError;
use bytes::Bytes;
use image::imageops::FilterType;
use image::{DynamicImage, ImageFormat};
use image_hasher::{HashAlg, HasherConfig};
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Debug)]
pub struct ImageOptimizer {
    semaphore: Arc<Semaphore>,
}

impl ImageOptimizer {
    #[must_use]
    pub fn new(max_concurrent_tasks: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent_tasks)),
        }
    }

    /// Process an image: resize, normalize, strip EXIF, compute pHash.
    /// Returns (`processed_data`, `content_type`, `p_hash`)
    ///
    /// # Errors
    /// Returns `UploadError::Internal` if semaphore acquisition fails or blocking task fails.
    /// Returns `UploadError::ImageError` if image processing fails.
    pub async fn optimize(
        &self,
        data: Bytes,
        max_dimension: u32,
        to_grayscale: bool,
    ) -> Result<(Bytes, String, String), UploadError> {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| UploadError::Internal(format!("Semaphore acquisition failed: {e}")))?;

        tokio::task::spawn_blocking(move || {
            let mut img = image::load_from_memory(&data)?;

            // 1. Strip EXIF by virtue of image::load_from_memory returning a DynamicImage
            // which doesn't keep metadata unless explicitly asked.

            // 2. Resize if necessary
            if img.width() > max_dimension || img.height() > max_dimension {
                img = img.resize(max_dimension, max_dimension, FilterType::Lanczos3);
            }

            // 3. Grayscale if requested
            if to_grayscale {
                img = DynamicImage::ImageLuma8(img.into_luma8());
            }

            // 4. Compute pHash
            let hasher = HasherConfig::new()
                .hash_alg(HashAlg::Gradient)
                .hash_size(8, 8)
                .to_hasher();
            let hash = hasher.hash_image(&img);
            let p_hash = hash.to_base64();

            // 5. Normalize to WebP (80% quality for excellent balance)
            let mut buffer = std::io::Cursor::new(Vec::new());
            img.write_to(&mut buffer, ImageFormat::WebP)?;
            let final_data = Bytes::from(buffer.into_inner());

            Ok((final_data, "image/webp".to_string(), p_hash))
        })
        .await
        .map_err(|e| UploadError::Internal(format!("Blocking task failed: {e}")))?
    }
}
