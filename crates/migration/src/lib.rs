pub use sea_orm_migration::prelude::*;

pub mod m20220101_000001_create_table;
pub mod m20260331_092335_create_groups_table;
pub mod m20260331_181001_add_missing_user_fields;
pub mod m20260331_185523_add_associated_contact_id;
pub mod m20260401_000001_add_indexes;
pub mod m20260403_000001_financial_refactor;
pub mod m20260404_000001_create_categories_table;
pub mod m20260404_000002_add_notes_to_transactions;
pub mod m20260404_000003_add_description_to_ledger_tabs;
pub mod m20260404_000004_fix_reconciliation_schema;
pub mod m20260404_000005_add_category_to_transactions;
pub mod m20260408_000001_create_ocr_jobs;
pub mod m20260417_000001_enhance_ocr_jobs;
pub mod m20260417_000002_add_ocr_retries;
pub mod m20260417_000003_add_ocr_scheduled_at;
pub mod m20260417_000004_add_ocr_raw_key;
pub mod m20260417_000005_add_ocr_schema_version;
pub mod m20260417_000006_add_contact_staging;
pub mod m20260417_000007_create_ocr_job_edits;
pub mod m20260417_000008_add_ocr_trace_id;
pub mod m20260422_000001_create_budgets_table;
pub mod m20260423_000001_add_cached_names_to_contacts;
pub mod m20260511_114743_update_wallet_type_enum_values;
pub mod m20260512_000001_add_bank_details_to_wallets;
pub mod m20260516_000001_create_background_jobs_table;
pub mod m20260522_000001_add_ocr_jobs_batch_and_idempotency;
pub mod m20260526_000001_enhance_background_jobs_table;
pub mod m20260526_000002_add_job_notifications;
pub mod m20260526_000003_add_performance_indexes;
pub mod m20260529_000001_add_transaction_date_index;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20260331_092335_create_groups_table::Migration),
            Box::new(m20260331_181001_add_missing_user_fields::Migration),
            Box::new(m20260331_185523_add_associated_contact_id::Migration),
            Box::new(m20260401_000001_add_indexes::Migration),
            Box::new(m20260403_000001_financial_refactor::Migration),
            Box::new(m20260404_000001_create_categories_table::Migration),
            Box::new(m20260404_000002_add_notes_to_transactions::Migration),
            Box::new(m20260404_000003_add_description_to_ledger_tabs::Migration),
            Box::new(m20260404_000004_fix_reconciliation_schema::Migration),
            Box::new(m20260404_000005_add_category_to_transactions::Migration),
            Box::new(m20260408_000001_create_ocr_jobs::Migration),
            Box::new(m20260417_000001_enhance_ocr_jobs::Migration),
            Box::new(m20260417_000002_add_ocr_retries::Migration),
            Box::new(m20260417_000003_add_ocr_scheduled_at::Migration),
            Box::new(m20260417_000004_add_ocr_raw_key::Migration),
            Box::new(m20260417_000005_add_ocr_schema_version::Migration),
            Box::new(m20260417_000006_add_contact_staging::Migration),
            Box::new(m20260417_000007_create_ocr_job_edits::Migration),
            Box::new(m20260417_000008_add_ocr_trace_id::Migration),
            Box::new(m20260422_000001_create_budgets_table::Migration),
            Box::new(m20260423_000001_add_cached_names_to_contacts::Migration),
            Box::new(m20260511_114743_update_wallet_type_enum_values::Migration),
            Box::new(m20260512_000001_add_bank_details_to_wallets::Migration),
            Box::new(m20260516_000001_create_background_jobs_table::Migration),
            Box::new(m20260522_000001_add_ocr_jobs_batch_and_idempotency::Migration),
            Box::new(m20260526_000001_enhance_background_jobs_table::Migration),
            Box::new(m20260526_000002_add_job_notifications::Migration),
            Box::new(m20260526_000003_add_performance_indexes::Migration),
            Box::new(m20260529_000001_add_transaction_date_index::Migration),
        ]
    }
}
