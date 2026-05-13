use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub mod account;
pub mod others;
pub mod session;
pub mod user;
pub mod verification;

#[derive(Clone)]
pub struct PostgresAdapter {
    pub db: Arc<DatabaseConnection>,
}

impl PostgresAdapter {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}
