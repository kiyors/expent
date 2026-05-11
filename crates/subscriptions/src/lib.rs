use chrono::{DateTime, FixedOffset};
use db::AppError;
use db::entities;
use db::entities::enums::{AlertChannel, SubscriptionCycle};
use rust_decimal::Decimal;
use sea_orm::DatabaseConnection;

pub mod ops;

#[derive(Clone)]
pub struct SubscriptionsManager {
    db: DatabaseConnection,
}

impl SubscriptionsManager {
    #[must_use]
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn detect(
        &self,
        user_id: &str,
    ) -> Result<Vec<entities::subscriptions::Model>, AppError> {
        ops::detect_subscriptions(&self.db, user_id).await
    }

    pub async fn list_confirmed(
        &self,
        user_id: &str,
    ) -> Result<Vec<entities::subscriptions::Model>, AppError> {
        ops::list_confirmed_subscriptions(&self.db, user_id).await
    }

    pub async fn confirm(
        &self,
        params: ops::ConfirmSubscriptionParams,
    ) -> Result<entities::subscriptions::Model, AppError> {
        ops::confirm_subscription(&self.db, params).await
    }

    pub async fn stop_tracking(&self, user_id: &str, sub_id: &str) -> Result<(), AppError> {
        ops::stop_tracking_subscription(&self.db, user_id, sub_id).await
    }

    pub async fn configure_alert(
        &self,
        sub_id: &str,
        days_before: i32,
        channel: AlertChannel,
    ) -> Result<entities::sub_alerts::Model, AppError> {
        ops::configure_subscription_alert(&self.db, sub_id, days_before, channel).await
    }
}
