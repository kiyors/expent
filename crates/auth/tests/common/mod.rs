//! Shared fixtures for the auth adapter integration tests.
//!
//! Defaults to an in-memory SQLite database so the suite runs cheaply
//! locally; honours `TEST_DATABASE_URL` (or `DATABASE_URL`) when set so the
//! CI Postgres job exercises the same tests against a real Postgres instance.

use auth::adapter::PostgresAdapter;
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};
use std::sync::Arc;

/// Build a freshly-migrated `PostgresAdapter` for use in tests.
///
/// SQLite databases are namespaced per call via a UUID query string so parallel
/// `#[tokio::test]` runs don't share state. For Postgres, callers are expected
/// to run against a dedicated test database (the CI job uses `expent_test`).
pub async fn setup_adapter() -> PostgresAdapter {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        // Each `sqlite::memory:` connection gets its own private database, so
        // parallel `#[tokio::test]` runs are naturally isolated. No
        // namespacing needed.
        .unwrap_or_else(|_| "sqlite::memory:".to_string());

    let db: DatabaseConnection = Database::connect(&database_url)
        .await
        .expect("connect to test database");
    Migrator::up(&db, None).await.expect("run migrations");
    PostgresAdapter::new(Arc::new(db))
}
