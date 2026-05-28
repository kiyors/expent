use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{ConnectionTrait, Statement};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Partial index on active transactions, keyed by user and date descending.
        // Targets the monthly-trends, list, and summary queries which filter by
        // (user_id, date range) and exclude soft-deleted rows. Partial-index
        // syntax is identical on Postgres and SQLite.
        let backend = manager.get_database_backend();
        let conn = manager.get_connection();
        let sql = r#"CREATE INDEX IF NOT EXISTS "idx-transactions-user-date-active"
            ON transactions (user_id, date DESC)
            WHERE deleted_at IS NULL"#;
        conn.execute(Statement::from_string(backend, sql.to_owned()))
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let backend = manager.get_database_backend();
        let conn = manager.get_connection();
        let sql = r#"DROP INDEX IF EXISTS "idx-transactions-user-date-active""#;
        conn.execute(Statement::from_string(backend, sql.to_owned()))
            .await?;
        Ok(())
    }
}
