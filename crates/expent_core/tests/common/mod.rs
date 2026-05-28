use expent_core::{Core, CoreConfig};
use migration::{Migrator, MigratorTrait};

pub async fn setup_test_core() -> Core {
    // Honour `DATABASE_URL` when set so CI can run the same integration suite
    // against Postgres (where the plpgsql LISTEN/NOTIFY migration actually
    // executes). Falls back to sqlite::memory locally for fast dev tests.
    let database_url = std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_else(|_| "sqlite::memory:".to_string());

    let config = CoreConfig {
        database_url,
        s3_endpoint: "http://localhost:9000".to_string(),
        s3_access_key_id: "test".to_string(),
        s3_secret_access_key: "test".to_string(),
        s3_bucket_name: "test".to_string(),
        google_api_key: Some("test_key".to_string()),
        better_auth_secret: "test_secret_key_at_least_32_chars_long_12345".to_string(),
        better_auth_base_url: "http://localhost:3000".to_string(),
        shutdown_token: None,
    };

    let (tx, _) = tokio::sync::broadcast::channel(100);
    let core = Core::init(config, tx).await.expect("Failed to init core");

    // Run migrations
    Migrator::up(&*core.db, None)
        .await
        .expect("Failed to run migrations");

    core
}

pub async fn create_test_user(core: &Core) -> String {
    use chrono::Utc;
    use db::entities::users;
    use sea_orm::{ActiveModelTrait, Set};

    let user_id = uuid::Uuid::now_v7().to_string();
    let user = users::ActiveModel {
        id: Set(user_id.clone()),
        email: Set(format!("test-{user_id}@example.com")),
        name: Set("Test User".to_string()),
        email_verified: Set(true),
        is_active: Set(true),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    };
    user.insert(&*core.db).await.expect("Failed to insert user");
    user_id
}

pub async fn create_test_wallet(
    core: &Core,
    user_id: &str,
    balance: rust_decimal::Decimal,
) -> String {
    use chrono::Utc;
    use db::entities::wallets;
    use sea_orm::{ActiveModelTrait, Set};

    let wallet_id = uuid::Uuid::now_v7().to_string();
    let wallet = wallets::ActiveModel {
        id: Set(wallet_id.clone()),
        user_id: Set(user_id.to_string()),
        name: Set("Test Wallet".to_string()),
        balance: Set(balance),
        r#type: Set(db::entities::enums::WalletType::Cash),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    };
    wallet
        .insert(&*core.db)
        .await
        .expect("Failed to insert wallet");
    wallet_id
}
