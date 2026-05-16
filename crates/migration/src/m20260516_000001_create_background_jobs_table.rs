use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(BackgroundJobs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(BackgroundJobs::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(BackgroundJobs::JobType).string().not_null())
                    .col(ColumnDef::new(BackgroundJobs::Payload).json().not_null())
                    .col(ColumnDef::new(BackgroundJobs::Status).string().not_null())
                    .col(
                        ColumnDef::new(BackgroundJobs::Attempts)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(BackgroundJobs::MaxAttempts)
                            .integer()
                            .not_null()
                            .default(3),
                    )
                    .col(ColumnDef::new(BackgroundJobs::RunAt).date_time().not_null())
                    .col(
                        ColumnDef::new(BackgroundJobs::CreatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .col(ColumnDef::new(BackgroundJobs::CompletedAt).date_time())
                    .col(ColumnDef::new(BackgroundJobs::Error).string())
                    .col(ColumnDef::new(BackgroundJobs::UserId).string())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(BackgroundJobs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum BackgroundJobs {
    Table,
    Id,
    JobType,
    Payload,
    Status,
    Attempts,
    MaxAttempts,
    RunAt,
    CreatedAt,
    CompletedAt,
    Error,
    UserId,
}
