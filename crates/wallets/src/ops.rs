use chrono::Utc;
use db::AppError;
use db::entities;
use db::entities::enums::WalletType;
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter,
    Set,
};

pub async fn create_wallet(
    db: &DatabaseConnection,
    user_id: &str,
    name: &str,
    wallet_type: WalletType,
    initial_balance: Decimal,
) -> Result<entities::wallets::Model, AppError> {
    let wallet = entities::wallets::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        user_id: Set(user_id.to_string()),
        name: Set(name.to_string()),
        r#type: Set(wallet_type),
        balance: Set(initial_balance),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    };

    wallet.insert(db).await.map_err(AppError::from)
}

pub async fn list_wallets(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<Vec<entities::wallets::Model>, AppError> {
    entities::wallets::Entity::find()
        .filter(entities::wallets::Column::UserId.eq(user_id))
        .all(db)
        .await
        .map_err(AppError::from)
}

pub async fn delete_wallet(
    db: &DatabaseConnection,
    user_id: &str,
    wallet_id: &str,
) -> Result<u64, AppError> {
    let result = entities::wallets::Entity::delete_many()
        .filter(entities::wallets::Column::UserId.eq(user_id))
        .filter(entities::wallets::Column::Id.eq(wallet_id))
        .exec(db)
        .await?;

    Ok(result.rows_affected)
}

pub async fn update_wallet(
    db: &DatabaseConnection,
    user_id: &str,
    wallet_id: &str,
    name: Option<String>,
    balance: Option<Decimal>,
) -> Result<entities::wallets::Model, AppError> {
    let mut wallet: entities::wallets::ActiveModel = entities::wallets::Entity::find()
        .filter(entities::wallets::Column::UserId.eq(user_id))
        .filter(entities::wallets::Column::Id.eq(wallet_id))
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Wallet not found"))?
        .into();

    if let Some(n) = name {
        wallet.name = Set(n);
    }
    if let Some(b) = balance {
        wallet.balance = Set(b);
    }
    wallet.updated_at = Set(Utc::now().into());

    wallet.update(db).await.map_err(AppError::from)
}

pub async fn get_balance(db: &DatabaseConnection, wallet_id: &str) -> Result<Decimal, AppError> {
    let wallet = entities::wallets::Entity::find_by_id(wallet_id.to_string())
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Wallet not found"))?;

    Ok(wallet.balance)
}

/// Adjusts the balance of a wallet atomically using database-level expressions.
pub async fn adjust_balance<C>(
    db: &C,
    wallet_id: &str,
    amount: Decimal,
    allow_negative: bool,
) -> Result<(), AppError>
where
    C: ConnectionTrait,
{
    if !allow_negative && amount.is_sign_negative() {
        let current_balance = entities::wallets::Entity::find_by_id(wallet_id.to_string())
            .one(db)
            .await?
            .ok_or_else(|| AppError::not_found("Wallet not found"))?
            .balance;

        if current_balance + amount < Decimal::ZERO {
            return Err(AppError::validation("Insufficient funds in wallet"));
        }
    }

    let result = entities::wallets::Entity::update_many()
        .col_expr(
            entities::wallets::Column::Balance,
            sea_orm::sea_query::Expr::col(entities::wallets::Column::Balance).add(amount),
        )
        .col_expr(
            entities::wallets::Column::UpdatedAt,
            sea_orm::sea_query::Expr::value(Utc::now()),
        )
        .filter(entities::wallets::Column::Id.eq(wallet_id))
        .exec(db)
        .await?;

    if result.rows_affected == 0 {
        return Err(AppError::not_found("Wallet not found"));
    }

    Ok(())
}

pub async fn get_wallet(
    db: &DatabaseConnection,
    user_id: &str,
    wallet_id: &str,
) -> Result<entities::wallets::Model, AppError> {
    entities::wallets::Entity::find()
        .filter(entities::wallets::Column::UserId.eq(user_id))
        .filter(entities::wallets::Column::Id.eq(wallet_id))
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Wallet not found"))
}

pub struct ResolveWalletParams {
    pub bank_name: String,
    pub account_number: Option<String>,
}

pub async fn resolve_wallet<C>(
    db: &C,
    user_id: &str,
    params: ResolveWalletParams,
) -> Result<entities::wallets::Model, AppError>
where
    C: ConnectionTrait,
{
    use sea_orm::Condition;

    // 1. Try to find by account number if provided
    if let Some(ref acc_num) = params.account_number {
        let existing = entities::wallets::Entity::find()
            .filter(entities::wallets::Column::UserId.eq(user_id))
            .filter(entities::wallets::Column::AccountNumber.eq(acc_num))
            .one(db)
            .await?;

        if let Some(wallet) = existing {
            return Ok(wallet);
        }
    }

    // 2. Try to find by bank name (fuzzy/partial match)
    let existing = entities::wallets::Entity::find()
        .filter(entities::wallets::Column::UserId.eq(user_id))
        .filter(
            Condition::any()
                .add(
                    entities::wallets::Column::Name
                        .like(format!("%{}%", params.bank_name.to_lowercase())),
                )
                .add(
                    entities::wallets::Column::BankName
                        .like(format!("%{}%", params.bank_name.to_lowercase())),
                ),
        )
        .one(db)
        .await?;

    if let Some(mut wallet) = existing {
        // Update account number if it was missing
        if wallet.account_number.is_none() && params.account_number.is_some() {
            let mut active: entities::wallets::ActiveModel = wallet.into();
            active.account_number = Set(params.account_number);
            active.updated_at = Set(Utc::now().into());
            wallet = active.update(db).await?;
        }
        return Ok(wallet);
    }

    // 3. Create new wallet if not found
    let wallet = entities::wallets::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        user_id: Set(user_id.to_string()),
        name: Set(params.bank_name.clone()),
        r#type: Set(WalletType::Bank),
        balance: Set(Decimal::ZERO), // Default to zero, user can adjust
        bank_name: Set(Some(params.bank_name)),
        account_number: Set(params.account_number),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    };

    wallet.insert(db).await.map_err(AppError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{DatabaseBackend, MockDatabase};

    #[tokio::test]
    async fn test_list_wallets() {
        let user_id = "user_123";
        let mock_wallets = vec![
            entities::wallets::Model {
                id: "wallet_1".to_string(),
                user_id: user_id.to_string(),
                name: "Cash".to_string(),
                r#type: WalletType::Cash,
                balance: Decimal::from(100),
                bank_name: None,
                account_number: None,
                created_at: Utc::now().into(),
                updated_at: Utc::now().into(),
            },
            entities::wallets::Model {
                id: "wallet_2".to_string(),
                user_id: user_id.to_string(),
                name: "Bank".to_string(),
                r#type: WalletType::Bank,
                balance: Decimal::from(5000),
                bank_name: None,
                account_number: None,
                created_at: Utc::now().into(),
                updated_at: Utc::now().into(),
            },
        ];

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![mock_wallets.clone()])
            .into_connection();

        let result = list_wallets(&db, user_id).await.unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, "wallet_1");
        assert_eq!(result[1].id, "wallet_2");
        assert_eq!(result[0].user_id, user_id);
        assert_eq!(result[1].user_id, user_id);

        /*
        // Verify the query was filtered by user_id
        let log = db.into_transaction_log();
        assert_eq!(log.len(), 1);
        let query = &log[0];
        if let sea_orm::Transaction::Query(q) = query {
            assert!(q.sql.contains("\"user_id\" = $1"));
            assert_eq!(q.values, vec![user_id.into()]);
        } else {
            panic!("Expected a query");
        }
        */
    }
}
