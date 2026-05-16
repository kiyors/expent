use anyhow::Result;
use reqwest::multipart;
use sea_orm::DatabaseConnection;
use serde_json::Value;
use std::sync::Arc;
use tracing::info;

pub mod ops;
pub mod strategies;
pub mod utils;
pub mod worker;

pub use ops::confirm::{confirm_ocr_job, resolve_contact_collision};
pub use ops::lifecycle::{create_ocr_job, get_ocr_job, list_pending_ocr_jobs, update_ocr_job};
pub use ops::process::{OcrProcessor, process_job};
pub use worker::{start_processor_worker, start_recovery_worker};

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct OcrUpdate {
    pub user_id: String,
    pub job_id: String,
    pub status: String,
    pub trace_id: Option<String>,
}

#[derive(Debug)]
pub struct OcrService {
    worker_url: String,
    client: reqwest::Client,
}

impl OcrService {
    pub async fn new(worker_url: Option<String>) -> Result<Self> {
        let mut url = worker_url.unwrap_or_else(|| {
            std::env::var("OCR_WORKER_URL").unwrap_or_else(|_| "http://localhost:8090".to_string())
        });

        if !url.contains("/extract") && !url.contains("/ocr") && !url.contains("/process") {
            if !url.ends_with('/') {
                url.push('/');
            }
            url.push_str("extract");
        }

        let client = reqwest::Client::new();
        info!("🚀 OCR Service (Proxy) initialized with worker at: {}", url);
        Ok(Self {
            worker_url: url,
            client,
        })
    }

    pub async fn process_file(
        &self,
        file_bytes: &[u8],
        filename: &str,
        mime_type: &str,
    ) -> Result<Value> {
        info!(
            "📄 Forwarding file '{}' ({}) for extraction ({} bytes)",
            filename,
            mime_type,
            file_bytes.len()
        );

        let part = multipart::Part::bytes(file_bytes.to_vec())
            .file_name(filename.to_string())
            .mime_str(mime_type)?;

        let form = multipart::Form::new().part("file", part);

        let res = self
            .client
            .post(&self.worker_url)
            .multipart(form)
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;

        info!("📝 Extraction completed successfully via Python worker.");
        Ok(res)
    }

    pub async fn process_image(&self, image_bytes: &[u8]) -> Result<Value> {
        self.process_file(image_bytes, "upload.png", "image/png")
            .await
    }
}

/// Central manager for the OCR lifecycle.
#[derive(Clone)]
pub struct OcrManager {
    pub service: Arc<OcrService>,
    pub db: Arc<DatabaseConnection>,
    pub upload: upload::UploadClient,
    pub ocr_tx: tokio::sync::broadcast::Sender<OcrUpdate>,
    pub semaphore: Arc<tokio::sync::Semaphore>,
}

pub use ops::lifecycle::OcrJobCreateParams;

impl OcrManager {
    pub fn new(
        service: Arc<OcrService>,
        db: Arc<DatabaseConnection>,
        upload: upload::UploadClient,
        ocr_tx: tokio::sync::broadcast::Sender<OcrUpdate>,
    ) -> Self {
        Self {
            service,
            db,
            upload,
            ocr_tx,
            semaphore: Arc::new(tokio::sync::Semaphore::new(10)),
        }
    }

    pub async fn start_job(
        &self,
        params: OcrJobCreateParams,
    ) -> Result<db::entities::ocr_jobs::Model, db::AppError> {
        ops::lifecycle::create_ocr_job(
            &*self.db,
            ops::lifecycle::OcrJobCreateParams {
                user_id: params.user_id,
                trace_id: params.trace_id,
                key: params.key,
                raw_key: params.raw_key,
                p_hash: params.p_hash,
                auto_confirm: params.auto_confirm,
                wallet_id: params.wallet_id,
                category_id: params.category_id,
            },
        )
        .await
    }

    pub async fn process_immediately(&self, processor: Arc<dyn OcrProcessor>, job_id: String) {
        let db = Arc::clone(&self.db);
        let service = Arc::clone(&self.service);
        let upload = self.upload.clone();
        let ocr_tx = self.ocr_tx.clone();
        let semaphore = self.semaphore.clone();

        tokio::spawn(async move {
            let _permit = semaphore.acquire().await.ok();
            if let Err(e) =
                ops::process::process_job(&*db, service, &upload, ocr_tx, processor, job_id).await
            {
                tracing::error!("❌ Immediate OCR processing failed: {}", e);
            }
        });
    }

    pub fn spawn_workers(&self, processor: Arc<dyn OcrProcessor>) {
        tokio::spawn(worker::start_recovery_worker(Arc::clone(&self.db)));
        tokio::spawn(worker::start_processor_worker(
            Arc::clone(&self.db),
            Arc::clone(&self.service),
            self.upload.clone(),
            self.ocr_tx.clone(),
            processor,
            self.semaphore.clone(),
        ));
    }

    pub async fn confirm_job(
        &self,
        processor: Arc<dyn OcrProcessor>,
        user_id: &str,
        job_id: &str,
        manual_data: Option<db::ProcessedOcr>,
    ) -> Result<db::OcrTransactionResponse, db::AppError> {
        let data = if let Some(d) = manual_data {
            serde_json::to_value(d)
                .map_err(|e| db::AppError::Ocr(format!("Serialization failed: {}", e)))?
        } else {
            let job = ops::lifecycle::get_ocr_job(&*self.db, job_id)
                .await?
                .ok_or_else(|| db::AppError::not_found("OCR Job not found"))?;
            job.processed_data
                .ok_or_else(|| db::AppError::validation("Job has no processed data"))?
        };

        ops::confirm::confirm_ocr_job(
            &*self.db,
            self.ocr_tx.clone(),
            processor,
            user_id,
            job_id,
            data,
        )
        .await
    }

    pub async fn resolve_collision(
        &self,
        processor: Arc<dyn OcrProcessor>,
        user_id: &str,
        job_id: &str,
        contact_id: &str,
    ) -> Result<db::OcrTransactionResponse, db::AppError> {
        ops::confirm::resolve_contact_collision(
            &*self.db,
            self.ocr_tx.clone(),
            processor,
            user_id,
            job_id,
            contact_id,
        )
        .await
    }
}
