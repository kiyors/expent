use chrono::Utc;
use db::entities;
use db::entities::enums::{P2pRequestStatus, TransactionDirection, TransactionSource};
use db::{AppError, P2pRequestWithSender};
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
    TransactionTrait,
};
use std::str::FromStr;
use transactions::TransactionsManager;

#[allow(clippy::missing_errors_doc)]
pub async fn list_pending_p2p_requests(
    db: &DatabaseConnection,
    email: &str,
) -> Result<Vec<P2pRequestWithSender>, AppError> {
    let results = entities::p2p_requests::Entity::find()
        .filter(entities::p2p_requests::Column::ReceiverEmail.eq(email))
        .filter(entities::p2p_requests::Column::Status.is_in(["PENDING", "GROUP_INVITE"]))
        .find_also_related(entities::users::Entity)
        .all(db)
        .await?;

    Ok(results
        .into_iter()
        .map(|(request, user)| P2pRequestWithSender {
            request,
            sender_name: user.map(|u| u.name),
        })
        .collect())
}

#[allow(clippy::missing_errors_doc)]
pub async fn create_p2p_request(
    db: &DatabaseConnection,
    sender_id: &str,
    receiver_email: &str,
    txn_id: &str,
) -> Result<entities::p2p_requests::Model, AppError> {
    let txn = entities::transactions::Entity::find_by_id(txn_id.to_string())
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Transaction not found"))?;

    // Security check: Ensure the transaction belongs to the sender
    if txn.user_id != sender_id {
        return Err(AppError::unauthorized(
            "You can only split your own transactions",
        ));
    }

    let txn_json = serde_json::to_value(&txn)
        .map_err(|e| AppError::Generic(format!("Failed to serialize transaction: {e}")))?;

    let request = entities::p2p_requests::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        sender_user_id: Set(sender_id.to_string()),
        receiver_email: Set(receiver_email.to_string()),
        transaction_data: Set(txn_json),
        status: Set(P2pRequestStatus::Pending),
        linked_txn_id: Set(None),
    };

    request.insert(db).await.map_err(AppError::from)
}

#[allow(clippy::missing_errors_doc)]
pub async fn accept_p2p_request(
    db: &DatabaseConnection,
    transactions: &TransactionsManager,
    receiver_id: &str,
    receiver_email: &str,
    request_id: &str,
) -> Result<entities::p2p_requests::Model, AppError> {
    let receiver_id = receiver_id.to_string();
    let receiver_email = receiver_email.to_string();
    let request_id = request_id.to_string();

    db.transaction::<_, entities::p2p_requests::Model, AppError>(|txn_db| {
        let transactions = transactions.clone();
        let receiver_id = receiver_id.clone();
        let receiver_email = receiver_email.clone();
        let request_id = request_id.clone();
        Box::pin(async move {
            let request = entities::p2p_requests::Entity::find_by_id(request_id)
                .one(txn_db)
                .await?
                .ok_or_else(|| AppError::not_found("Request not found"))?;

            if receiver_email != request.receiver_email {
                return Err(AppError::unauthorized(
                    "Not authorized to accept this request",
                ));
            }

            if request.status != P2pRequestStatus::Pending
                && request.status != P2pRequestStatus::GroupInvite
            {
                return Err(AppError::unauthorized("Request is not pending"));
            }

            if request.status == P2pRequestStatus::GroupInvite {
                let metadata: serde_json::Value =
                    serde_json::from_value(request.transaction_data.clone()).map_err(|e| {
                        AppError::Generic(format!("Failed to parse invite data: {e}"))
                    })?;

                let group_id = metadata["group_id"]
                    .as_str()
                    .ok_or_else(|| AppError::Generic("Missing group_id in invite".to_string()))?;

                let user_group = entities::user_groups::ActiveModel {
                    user_id: Set(receiver_id.clone()),
                    group_id: Set(group_id.to_string()),
                    role: Set(db::entities::enums::GroupRole::Member),
                };
                user_group.insert(txn_db).await?;

                let mut request: entities::p2p_requests::ActiveModel = request.into();
                request.status = Set(P2pRequestStatus::Approved);
                return request.update(txn_db).await.map_err(AppError::from);
            }

            let original_txn: serde_json::Value =
                serde_json::from_value(request.transaction_data.clone()).map_err(|e| {
                    AppError::Generic(format!("Failed to parse transaction data: {e}"))
                })?;

            let amount = original_txn["amount"]
                .as_str()
                .and_then(|s| Decimal::from_str(s).ok())
                .unwrap_or(Decimal::ZERO);

            let date = original_txn["date"]
                .as_str()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map_or_else(|| Utc::now().into(), |d| d.with_timezone(&Utc).into());

            let purpose = original_txn["purpose"]
                .as_str()
                .map(std::string::ToString::to_string);

            let result_txn = transactions
                .create(
                    &receiver_id,
                    amount,
                    TransactionDirection::In,
                    date,
                    TransactionSource::P2p,
                    purpose,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .await?;

            let mut request: entities::p2p_requests::ActiveModel = request.into();
            request.status = Set(P2pRequestStatus::Mapped);
            request.linked_txn_id = Set(Some(result_txn.id));

            request.update(txn_db).await.map_err(AppError::from)
        })
    })
    .await
    .map_err(|e| match e {
        sea_orm::TransactionError::Connection(ce) => AppError::Db(ce),
        sea_orm::TransactionError::Transaction(te) => te,
    })
}

#[allow(clippy::missing_errors_doc)]
pub async fn reject_p2p_request(
    db: &DatabaseConnection,
    user_id: &str,
    user_email: &str,
    request_id: &str,
) -> Result<(), AppError> {
    let request_model = entities::p2p_requests::Entity::find_by_id(request_id.to_string())
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Request not found"))?;

    if request_model.receiver_email != user_email && request_model.sender_user_id != user_id {
        return Err(AppError::unauthorized(
            "Not authorized to reject this request",
        ));
    }

    let mut request: entities::p2p_requests::ActiveModel = request_model.into();
    request.status = Set(P2pRequestStatus::Rejected);
    request.update(db).await?;
    Ok(())
}
