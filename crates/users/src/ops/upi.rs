use db::AppError;
use db::entities;
use regex::Regex;
use sea_orm::prelude::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    Set,
};
use std::sync::LazyLock;

static UPI_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9.\-_]{2,256}@[a-zA-Z]{2,64}$").unwrap());

#[allow(clippy::missing_errors_doc)]
pub async fn list_user_upi(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<Vec<entities::user_upi_ids::Model>, AppError> {
    entities::user_upi_ids::Entity::find()
        .filter(entities::user_upi_ids::Column::UserId.eq(user_id))
        .all(db)
        .await
        .map_err(AppError::from)
}

#[allow(clippy::missing_errors_doc)]
pub async fn add_user_upi(
    db: &DatabaseConnection,
    user_id: &str,
    upi_id: String,
    label: Option<String>,
) -> Result<entities::user_upi_ids::Model, AppError> {
    // 1. Basic UPI Format Validation (handle@bank)
    if !UPI_REGEX.is_match(&upi_id) {
        // Use the lazy_static regex
        return Err(AppError::validation(format!(
            "Invalid UPI ID format: '{upi_id}'"
        )));
    }

    // 2. Auto-Primary Check: If this is the first UPI, make it primary
    let existing_count = entities::user_upi_ids::Entity::find()
        .filter(entities::user_upi_ids::Column::UserId.eq(user_id))
        .count(db)
        .await?;

    let upi = entities::user_upi_ids::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        user_id: Set(user_id.to_string()),
        upi_id: Set(upi_id),
        is_primary: Set(existing_count == 0),
        label: Set(label),
    };
    upi.insert(db).await.map_err(AppError::from)
}

#[allow(clippy::missing_errors_doc)]
pub async fn make_primary_upi(
    db: &DatabaseConnection,
    user_id: &str,
    upi_id: &str,
) -> Result<(), AppError> {
    // Unset current primary
    entities::user_upi_ids::Entity::update_many()
        .col_expr(
            entities::user_upi_ids::Column::IsPrimary,
            Expr::value(false),
        )
        .filter(entities::user_upi_ids::Column::UserId.eq(user_id))
        .exec(db)
        .await?;

    // Set new primary
    entities::user_upi_ids::Entity::update_many()
        .col_expr(entities::user_upi_ids::Column::IsPrimary, Expr::value(true))
        .filter(entities::user_upi_ids::Column::UserId.eq(user_id))
        .filter(entities::user_upi_ids::Column::Id.eq(upi_id))
        .exec(db)
        .await?;

    Ok(())
}
