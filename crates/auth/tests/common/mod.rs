//! Shared fixtures for the auth adapter integration tests.
//!
//! Defaults to an in-memory SQLite database so the suite runs cheaply
//! locally; honours `TEST_DATABASE_URL` (or `DATABASE_URL`) when set so the
//! CI Postgres job exercises the same tests against a real Postgres instance.

use auth::adapter::PostgresAdapter;
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseBackend, Statement};
use std::sync::Arc;

/// Build a freshly-migrated `PostgresAdapter` for use in tests.
///
/// SQLite databases are isolated per-connection automatically, so parallel
/// `#[tokio::test]` runs don't share state.
///
/// Postgres has one physical database in CI but many concurrent test threads,
/// which would otherwise race on `CREATE TYPE` (Postgres enums aren't
/// idempotent) and leak data across tests. We work around both by giving each
/// fixture call its own unique schema and pointing the connection pool at it,
/// so each test ends up with a fully private set of tables, types, and rows.
pub async fn setup_adapter() -> PostgresAdapter {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_else(|_| "sqlite::memory:".to_string());

    let db = if database_url.starts_with("postgres") {
        // 1. Create a uniquely-named schema using a short-lived admin
        //    connection. UUIDv7 is unique enough that parallel CREATE SCHEMA
        //    calls can't collide.
        let schema_name = format!("test_{}", uuid::Uuid::now_v7().simple());
        let admin = Database::connect(&database_url)
            .await
            .expect("connect (admin) to Postgres for schema setup");
        admin
            .execute(Statement::from_string(
                DatabaseBackend::Postgres,
                format!("CREATE SCHEMA \"{schema_name}\""),
            ))
            .await
            .expect("create per-test schema");
        // Drop the admin connection so it doesn't hold a pool slot.
        drop(admin);

        // 2. Build the real test pool with `search_path` pinned to that
        //    schema. SeaORM applies the `SET search_path` to every connection
        //    in the pool, so every query (migrations, app code, …) sees its
        //    own private namespace.
        let mut opts = ConnectOptions::new(database_url);
        opts.set_schema_search_path(&schema_name);
        Database::connect(opts)
            .await
            .expect("connect to Postgres with private schema")
    } else {
        Database::connect(&database_url)
            .await
            .expect("connect to SQLite")
    };

    Migrator::up(&db, None).await.expect("run migrations");
    PostgresAdapter::new(Arc::new(db))
}
