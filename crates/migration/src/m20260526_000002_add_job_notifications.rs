use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // 1. Function and Trigger for background_jobs
        db.execute_unprepared(
            r#"
            CREATE OR REPLACE FUNCTION notify_background_job() RETURNS trigger AS $$
            BEGIN
                PERFORM pg_notify('background_jobs_channel', NEW.id);
                RETURN NEW;
            END;
            $$ LANGUAGE plpgsql;

            DROP TRIGGER IF EXISTS trigger_notify_background_job ON background_jobs;
            CREATE TRIGGER trigger_notify_background_job
            AFTER INSERT ON background_jobs
            FOR EACH ROW EXECUTE FUNCTION notify_background_job();
            "#
        ).await?;

        // 2. Function and Trigger for ocr_jobs
        db.execute_unprepared(
            r#"
            CREATE OR REPLACE FUNCTION notify_ocr_job() RETURNS trigger AS $$
            BEGIN
                PERFORM pg_notify('ocr_jobs_channel', NEW.id);
                RETURN NEW;
            END;
            $$ LANGUAGE plpgsql;

            DROP TRIGGER IF EXISTS trigger_notify_ocr_job ON ocr_jobs;
            CREATE TRIGGER trigger_notify_ocr_job
            AFTER INSERT ON ocr_jobs
            FOR EACH ROW EXECUTE FUNCTION notify_ocr_job();
            "#
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            r#"
            DROP TRIGGER IF EXISTS trigger_notify_background_job ON background_jobs;
            DROP FUNCTION IF EXISTS notify_background_job();
            
            DROP TRIGGER IF EXISTS trigger_notify_ocr_job ON ocr_jobs;
            DROP FUNCTION IF EXISTS notify_ocr_job();
            "#
        ).await?;

        Ok(())
    }
}
