use anyhow::Result;
use async_trait::async_trait;
use expent_core::Core;
use jobs::JobHandler;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct BulkConfirmOcrJobPayload {
    pub user_id: String,
    pub job_ids: Vec<String>,
}

pub struct BulkConfirmOcrJobHandler {
    pub core: Arc<Core>,
}

#[async_trait]
impl JobHandler for BulkConfirmOcrJobHandler {
    async fn handle(&self, payload: serde_json::Value) -> Result<()> {
        let payload: BulkConfirmOcrJobPayload = serde_json::from_value(payload)?;

        // Use bounded concurrency (like we had in the API handler before)
        use futures::StreamExt;
        let stream = futures::stream::iter(payload.job_ids).map(|job_id| {
            let core = self.core.clone();
            let user_id = payload.user_id.clone();
            async move {
                let res = core
                    .ocr_manager
                    .confirm_job(core.clone(), &user_id, &job_id, None)
                    .await;
                (job_id, res)
            }
        });

        let mut results = stream.buffer_unordered(5);

        while let Some((job_id, result)) = results.next().await {
            match result {
                Ok(_) => tracing::info!("✅ Background bulk-confirm succeeded for job: {}", job_id),
                Err(e) => tracing::error!(
                    "❌ Background bulk-confirm failed for job {}: {}",
                    job_id,
                    e
                ),
            }
        }

        Ok(())
    }
}
