use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // SQLite rejects multiple ALTER options in one statement, so each column
        // is added in its own alter_table call.
        manager
            .alter_table(
                Table::alter()
                    .table(BackgroundJobs::Table)
                    .add_column(ColumnDef::new(BackgroundJobs::StartedAt).date_time())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(BackgroundJobs::Table)
                    .add_column(
                        ColumnDef::new(BackgroundJobs::UpdatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(BackgroundJobs::Table)
                    .drop_column(BackgroundJobs::StartedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(BackgroundJobs::Table)
                    .drop_column(BackgroundJobs::UpdatedAt)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum BackgroundJobs {
    Table,
    StartedAt,
    UpdatedAt,
}
