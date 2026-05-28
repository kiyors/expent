use anyhow::Result;
use sea_orm::DatabaseConnection;
use serde_json::{Value, json};
use std::sync::Arc;
use tracing::info;

pub mod gemini;
pub mod ops;
pub mod schema;
pub mod strategies;
pub mod utils;
pub mod worker;

pub use gemini::GeminiOcrClient;
pub use ops::confirm::{confirm_ocr_job, resolve_contact_collision};
pub use ops::lifecycle::{
    create_ocr_job, get_ocr_job, get_ocr_job_by_idempotency_key, list_pending_ocr_jobs,
    update_ocr_job,
};
pub use ops::merge::merge_ocr_results;
pub use ops::process::{OcrProcessor, process_job};
pub use worker::{start_processor_worker, start_recovery_worker};

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct OcrUpdate {
    pub user_id: String,
    pub job_id: String,
    pub status: String,
    pub trace_id: Option<String>,
}

#[derive(Clone)]
pub struct OcrService {
    gemini: Arc<GeminiOcrClient>,
}

impl OcrService {
    pub async fn new(api_key: Option<String>) -> Result<Self> {
        let key = api_key
            .or_else(|| std::env::var("GOOGLE_API_KEY").ok())
            .ok_or_else(|| anyhow::anyhow!("GOOGLE_API_KEY not found"))?;

        let gemini = GeminiOcrClient::new(key);
        info!("🚀 OCR Service (Native) initialized with Gemini client.");
        Ok(Self {
            gemini: Arc::new(gemini),
        })
    }

    pub async fn process_file(
        &self,
        file_bytes: &[u8],
        filename: &str,
        _mime_type: &str,
    ) -> Result<Value> {
        info!(
            "📄 Processing file '{}' for extraction ({} bytes) via Gemini",
            filename,
            file_bytes.len()
        );

        let media_type = utils::get_media_type(filename);

        // --- MULTI-PAGE PARALLEL PROCESSING ---
        let mut extractions = if media_type == "application/pdf" {
            let pages = utils::split_pdf(file_bytes)?;
            if pages.len() > 1 {
                info!(
                    "📂 Multi-page PDF detected ({} pages), processing in parallel",
                    pages.len()
                );
                let batch = pages
                    .into_iter()
                    .enumerate()
                    .map(|(i, p)| (p, format!("{}_page_{}.pdf", filename, i + 1)))
                    .collect();
                self.gemini.extract_batch(batch).await?
            } else {
                vec![self.gemini.extract_from_bytes(file_bytes, filename).await?]
            }
        } else {
            vec![self.gemini.extract_from_bytes(file_bytes, filename).await?]
        };

        // If only one extraction, proceed as normal
        if extractions.len() == 1 {
            let extracted = extractions.remove(0);
            return self.format_extraction(extracted);
        }

        // --- PROGRAMMATIC MERGING ---
        info!(
            "🤝 Merging {} extraction results deterministically",
            extractions.len()
        );

        // We only support merging GENERIC types for now as multi-page receipts
        let generic_results: Vec<db::OcrResult> = extractions
            .iter()
            .filter_map(|e| e.generic_data.clone())
            .collect();

        if generic_results.is_empty() {
            // Fallback to the first result if no generic data found (e.g. all bank statements)
            return self.format_extraction(extractions.remove(0));
        }

        let merged = ops::merge::merge_ocr_results(generic_results);

        Ok(json!({
            "doc_type": "GENERIC",
            "data": merged,
        }))
    }

    fn format_extraction(&self, extracted: crate::schema::UnifiedExtraction) -> Result<Value> {
        // Map UnifiedExtraction to the ProcessedOcr format Rust expects
        let doc_type = match extracted.doc_type {
            schema::DocType::Gpay => "GPAY",
            schema::DocType::BankStatement => "BANK_STATEMENT",
            schema::DocType::Generic => "GENERIC",
        };

        let data = match extracted.doc_type {
            schema::DocType::Gpay => serde_json::to_value(
                extracted
                    .gpay_data
                    .ok_or_else(|| anyhow::anyhow!("GPAY data missing"))?,
            )?,
            schema::DocType::BankStatement => serde_json::to_value(db::BankExtractionResult {
                raw_text: extracted.raw_text.clone().unwrap_or_default(),
                doc_type: "bank_statement".to_string(),
                confidence_score: extracted.confidence_score,
                bank_data: extracted
                    .bank_data
                    .ok_or_else(|| anyhow::anyhow!("Bank data missing"))?,
            })?,
            schema::DocType::Generic => serde_json::to_value(
                extracted
                    .generic_data
                    .ok_or_else(|| anyhow::anyhow!("Generic data missing"))?,
            )?,
        };

        Ok(json!({
            "doc_type": doc_type,
            "data": data,
        }))
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
    pub cancellation_token: tokio_util::sync::CancellationToken,
    pub task_tracker: tokio_util::task::TaskTracker,
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
            cancellation_token: tokio_util::sync::CancellationToken::new(),
            task_tracker: tokio_util::task::TaskTracker::new(),
        }
    }

    pub fn with_cancellation_token(mut self, token: tokio_util::sync::CancellationToken) -> Self {
        self.cancellation_token = token;
        self
    }

    pub async fn start_job(
        &self,
        params: OcrJobCreateParams,
    ) -> Result<db::entities::ocr_jobs::Model, db::AppError> {
        ops::lifecycle::create_ocr_job(
            &self.db,
            ops::lifecycle::OcrJobCreateParams {
                user_id: params.user_id,
                trace_id: params.trace_id,
                key: params.key,
                raw_key: params.raw_key,
                p_hash: params.p_hash,
                auto_confirm: params.auto_confirm,
                wallet_id: params.wallet_id,
                category_id: params.category_id,
                batch_id: params.batch_id,
                idempotency_key: params.idempotency_key,
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
                ops::process::process_job(&db, service, &upload, ocr_tx, processor, job_id).await
            {
                tracing::error!("❌ Immediate OCR processing failed: {}", e);
            }
        });
    }

    pub fn spawn_workers(&self, processor: Arc<dyn OcrProcessor>) {
        tokio::spawn(worker::start_recovery_worker(
            Arc::clone(&self.db),
            self.cancellation_token.clone(),
        ));
        tokio::spawn(worker::start_processor_worker(
            Arc::clone(&self.db),
            Arc::clone(&self.service),
            self.upload.clone(),
            self.ocr_tx.clone(),
            processor,
            self.semaphore.clone(),
            self.cancellation_token.clone(),
            self.task_tracker.clone(),
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
            let job = ops::lifecycle::get_ocr_job(&self.db, job_id)
                .await?
                .ok_or_else(|| db::AppError::not_found("OCR Job not found"))?;
            job.processed_data
                .ok_or_else(|| db::AppError::validation("Job has no processed data"))?
        };

        ops::confirm::confirm_ocr_job(
            &self.db,
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
            &self.db,
            self.ocr_tx.clone(),
            processor,
            user_id,
            job_id,
            contact_id,
        )
        .await
    }
}
