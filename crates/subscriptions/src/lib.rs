use db::AppError;
use db::entities;
use db::entities::enums::AlertChannel;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub mod ops;

#[derive(Clone)]
pub struct SubscriptionsManager {
    db: Arc<DatabaseConnection>,
}

impl SubscriptionsManager {
    #[must_use]
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// Detects potential recurring subscriptions from the user's recent transactions.
    ///
    /// # Errors
    /// Propagates [`AppError::Db`] from [`ops::detect_subscriptions`] if the
    /// transaction history query fails.
    pub async fn detect(
        &self,
        user_id: &str,
    ) -> Result<Vec<entities::subscriptions::Model>, AppError> {
        ops::detect_subscriptions(&self.db, user_id).await
    }

    /// Lists all confirmed subscriptions for the user.
    ///
    /// # Errors
    /// Propagates [`AppError::Db`] from [`ops::list_confirmed_subscriptions`] on query failure.
    pub async fn list_confirmed(
        &self,
        user_id: &str,
    ) -> Result<Vec<entities::subscriptions::Model>, AppError> {
        ops::list_confirmed_subscriptions(&self.db, user_id).await
    }

    /// Confirms (persists) a subscription. Idempotent on (user, name, cycle).
    ///
    /// # Errors
    /// Propagates [`AppError::Db`] from [`ops::confirm_subscription`] if the lookup or insert fails.
    pub async fn confirm(
        &self,
        params: ops::ConfirmSubscriptionParams,
    ) -> Result<entities::subscriptions::Model, AppError> {
        ops::confirm_subscription(&self.db, params).await
    }

    /// Stops tracking a subscription for the user.
    ///
    /// # Errors
    /// Propagates [`AppError::Db`] from [`ops::stop_tracking_subscription`] if the delete fails.
    pub async fn stop_tracking(&self, user_id: &str, sub_id: &str) -> Result<(), AppError> {
        ops::stop_tracking_subscription(&self.db, user_id, sub_id).await
    }

    /// Configures an alert for an existing subscription.
    ///
    /// # Errors
    /// Propagates [`AppError::Db`] from [`ops::configure_subscription_alert`] if the
    /// alert insert fails.
    pub async fn configure_alert(
        &self,
        sub_id: &str,
        days_before: i32,
        channel: AlertChannel,
    ) -> Result<entities::sub_alerts::Model, AppError> {
        ops::configure_subscription_alert(&self.db, sub_id, days_before, channel).await
    }
}
