use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(OcrJobs::Table)
                    .add_column(ColumnDef::new(OcrJobs::BatchId).string().null())
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(OcrJobs::Table)
                    .add_column(ColumnDef::new(OcrJobs::IdempotencyKey).string().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-ocr-jobs-idempotency-key")
                    .table(OcrJobs::Table)
                    .col(OcrJobs::IdempotencyKey)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx-ocr-jobs-idempotency-key")
                    .table(OcrJobs::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(OcrJobs::Table)
                    .drop_column(OcrJobs::BatchId)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(OcrJobs::Table)
                    .drop_column(OcrJobs::IdempotencyKey)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum OcrJobs {
    Table,
    BatchId,
    IdempotencyKey,
}
