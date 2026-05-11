use crate::ops::{
    confirm_ocr_job, create_ocr_job, process_job, resolve_contact_collision, OcrProcessor,
};
use crate::service::OcrService;
use crate::workers::{start_processor_worker, start_recovery_worker};
use anyhow::Result;
use db::entities;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::Semaphore;
use upload::UploadClient;

/// Central manager for the OCR lifecycle.
#[derive(Clone)]
pub struct OcrManager {
    pub service: Arc<OcrService>,
    pub db: DatabaseConnection,
    pub upload: UploadClient,
    pub ocr_tx: tokio::sync::broadcast::Sender<crate::OcrUpdate>,
    pub semaphore: Arc<Semaphore>,
}

impl OcrManager {
    pub fn new(
        service: Arc<OcrService>,
        db: DatabaseConnection,
        upload: UploadClient,
        ocr_tx: tokio::sync::broadcast::Sender<crate::OcrUpdate>,
    ) -> Self {
        Self {
            service,
            db,
            upload,
            ocr_tx,
            semaphore: Arc::new(Semaphore::new(4)), // Limit to 4 concurrent OCR tasks
        }
    }

    pub fn spawn_workers(&self, processor: Arc<dyn OcrProcessor>) {
        tokio::spawn(start_recovery_worker(self.db.clone()));
        tokio::spawn(start_processor_worker(
            self.db.clone(),
            Arc::clone(&self.service),
            self.upload.clone(),
            self.ocr_tx.clone(),
            processor,
            Arc::clone(&self.semaphore),
        ));
    }

    pub async fn start_job(
        &self,
        user_id: &str,
        trace_id: Option<String>,
        key: &str,
        raw_key: Option<String>,
        p_hash: Option<String>,
        auto_confirm: bool,
        wallet_id: Option<String>,
        category_id: Option<String>,
    ) -> Result<entities::ocr_jobs::Model, db::AppError> {
        create_ocr_job(
            &self.db,
            user_id,
            trace_id,
            key,
            raw_key,
            p_hash,
            auto_confirm,
            wallet_id,
            category_id,
        )
        .await
    }

    pub async fn process_immediately(&self, processor: Arc<dyn OcrProcessor>, job_id: String) {
        let db = self.db.clone();
        let service = Arc::clone(&self.service);
        let upload = self.upload.clone();
        let ocr_tx = self.ocr_tx.clone();
        let semaphore = Arc::clone(&self.semaphore);

        tokio::spawn(async move {
            let _permit = match semaphore.acquire().await {
                Ok(p) => p,
                Err(_) => return,
            };

            if let Err(e) = process_job(&db, service, &upload, ocr_tx, processor, job_id).await {
                tracing::error!("❌ Immediate OCR processing failed: {}", e);
            }
        });
    }

    pub async fn confirm_job(
        &self,
        processor: Arc<dyn OcrProcessor>,
        user_id: &str,
        job_id: &str,
        manual_data: Option<db::ProcessedOcr>,
    ) -> Result<db::OcrTransactionResponse, db::AppError> {
        confirm_ocr_job(
            &self.db,
            &self.upload,
            processor,
            user_id,
            job_id,
            manual_data,
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
        resolve_contact_collision(
            &self.db,
            &self.upload,
            processor,
            user_id,
            job_id,
            contact_id,
        )
        .await
    }
}
