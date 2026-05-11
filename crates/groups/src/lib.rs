use db::AppError;
use db::entities;
use db::entities::enums::{GroupRole, LedgerTabType};
use rust_decimal::Decimal;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use transactions::TransactionsManager;
use wallets::WalletsManager;

pub mod groups;
pub mod p2p;

#[derive(Clone)]
pub struct GroupsManager {
    db: DatabaseConnection,
    wallets: Arc<WalletsManager>,
    transactions: Arc<TransactionsManager>,
}

impl GroupsManager {
    #[must_use]
    pub fn new(
        db: DatabaseConnection,
        wallets: Arc<WalletsManager>,
        transactions: Arc<TransactionsManager>,
    ) -> Self {
        Self {
            db,
            wallets,
            transactions,
        }
    }

    // --- Groups API ---

    pub async fn create_group(
        &self,
        user_id: &str,
        name: &str,
        description: Option<String>,
    ) -> Result<entities::groups::Model, AppError> {
        groups::create_group(&self.db, user_id, name, description).await
    }

    pub async fn list_groups(
        &self,
        user_id: &str,
    ) -> Result<Vec<entities::groups::Model>, AppError> {
        groups::list_groups(&self.db, user_id).await
    }

    pub async fn get_group(
        &self,
        user_id: &str,
        group_id: &str,
    ) -> Result<entities::groups::Model, AppError> {
        groups::get_group(&self.db, user_id, group_id).await
    }

    pub async fn invite_to_group(
        &self,
        sender_id: &str,
        receiver_email: &str,
        group_id: &str,
    ) -> Result<entities::p2p_requests::Model, AppError> {
        groups::invite_to_group(&self.db, sender_id, receiver_email, group_id).await
    }

    pub async fn remove_group_member(
        &self,
        admin_id: &str,
        group_id: &str,
        target_user_id: &str,
    ) -> Result<(), AppError> {
        groups::remove_group_member(&self.db, admin_id, group_id, target_user_id).await
    }

    pub async fn update_member_role(
        &self,
        admin_id: &str,
        group_id: &str,
        target_user_id: &str,
        new_role: GroupRole,
    ) -> Result<(), AppError> {
        groups::update_member_role(&self.db, admin_id, group_id, target_user_id, new_role).await
    }

    pub async fn list_group_transactions(
        &self,
        group_id: &str,
    ) -> Result<Vec<entities::transactions::Model>, AppError> {
        groups::list_group_transactions(&self.db, group_id).await
    }

    pub async fn list_group_members(
        &self,
        group_id: &str,
    ) -> Result<Vec<db::GroupMemberDetail>, AppError> {
        groups::members::list_group_members(&self.db, group_id).await
    }

    // --- P2P API ---

    pub async fn create_ledger_tab(
        &self,
        creator_id: &str,
        counterparty_id: Option<String>,
        tab_type: LedgerTabType,
        title: &str,
        description: Option<String>,
        target_amount: Decimal,
    ) -> Result<entities::ledger_tabs::Model, AppError> {
        p2p::create_ledger_tab(
            &self.db,
            creator_id,
            counterparty_id,
            tab_type,
            title,
            description,
            target_amount,
        )
        .await
    }

    pub async fn list_pending_p2p_requests(
        &self,
        email: &str,
    ) -> Result<Vec<db::P2pRequestWithSender>, AppError> {
        p2p::list_pending_p2p_requests(&self.db, email).await
    }

    pub async fn create_p2p_request(
        &self,
        sender_id: &str,
        receiver_email: &str,
        txn_id: &str,
    ) -> Result<entities::p2p_requests::Model, AppError> {
        p2p::create_p2p_request(&self.db, sender_id, receiver_email, txn_id).await
    }

    pub async fn accept_p2p_request(
        &self,
        receiver_id: &str,
        request_id: &str,
    ) -> Result<entities::p2p_requests::Model, AppError> {
        p2p::accept_p2p_request(&self.db, &self.transactions, receiver_id, request_id).await
    }

    pub async fn reject_p2p_request(
        &self,
        user_id: &str,
        request_id: &str,
    ) -> Result<(), AppError> {
        p2p::reject_p2p_request(&self.db, user_id, request_id).await
    }

    pub async fn register_repayment(
        &self,
        user_id: &str,
        tab_id: &str,
        amount: Decimal,
        wallet_id: Option<String>,
    ) -> Result<entities::transactions::Model, AppError> {
        p2p::register_repayment(
            &self.db,
            self.wallets.clone(),
            user_id,
            tab_id,
            amount,
            wallet_id,
        )
        .await
    }
}
