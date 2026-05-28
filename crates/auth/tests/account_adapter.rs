//! Integration tests for `PostgresAdapter`'s `AccountOps` implementation.
//!
//! Accounts are the oauth-provider linkage layer (one row per provider/account
//! pair for a given user). Each test seeds a user first.

mod common;

use better_auth::types_mod::{AccountOps, CreateAccount, CreateUser, UserOps};
use common::setup_adapter;

async fn seed_user(adapter: &auth::adapter::PostgresAdapter, email: &str) -> String {
    let user = adapter
        .create_user(CreateUser {
            id: None,
            name: Some("Test".to_string()),
            email: Some(email.to_string()),
            email_verified: Some(false),
            image: None,
            password: None,
            role: None,
            username: None,
            display_username: None,
            metadata: None,
        })
        .await
        .expect("create_user");
    user.id
}

fn new_account(user_id: &str, provider: &str, account_id: &str) -> CreateAccount {
    CreateAccount {
        user_id: user_id.to_string(),
        account_id: account_id.to_string(),
        provider_id: provider.to_string(),
        access_token: Some("access-token".to_string()),
        refresh_token: Some("refresh-token".to_string()),
        id_token: None,
        access_token_expires_at: None,
        refresh_token_expires_at: None,
        scope: Some("openid email".to_string()),
        password: None,
    }
}

#[tokio::test]
async fn create_then_get_by_provider_and_account_id() {
    let adapter = setup_adapter().await;
    let user_id = seed_user(&adapter, "alice@example.com").await;

    let created = adapter
        .create_account(new_account(&user_id, "google", "g-12345"))
        .await
        .expect("create_account");
    assert_eq!(created.user_id, user_id);

    let fetched = adapter
        .get_account("google", "g-12345")
        .await
        .expect("get_account")
        .expect("account should exist after create");
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.provider_id, "google");
    assert_eq!(fetched.account_id, "g-12345");
}

#[tokio::test]
async fn get_account_returns_none_for_unknown() {
    let adapter = setup_adapter().await;
    let result = adapter
        .get_account("google", "unknown")
        .await
        .expect("query should succeed");
    assert!(result.is_none());
}

#[tokio::test]
async fn get_user_accounts_returns_all_providers_for_user() {
    let adapter = setup_adapter().await;
    let user_id = seed_user(&adapter, "bob@example.com").await;
    let other_id = seed_user(&adapter, "other@example.com").await;

    adapter
        .create_account(new_account(&user_id, "google", "g-1"))
        .await
        .expect("create_account");
    adapter
        .create_account(new_account(&user_id, "github", "gh-1"))
        .await
        .expect("create_account");
    // Cross-user isolation guard.
    adapter
        .create_account(new_account(&other_id, "google", "g-other"))
        .await
        .expect("create_account");

    let accounts = adapter
        .get_user_accounts(&user_id)
        .await
        .expect("get_user_accounts");
    assert_eq!(accounts.len(), 2);
    let providers: Vec<&str> = accounts.iter().map(|a| a.provider_id.as_str()).collect();
    assert!(providers.contains(&"google"));
    assert!(providers.contains(&"github"));
}

#[tokio::test]
async fn delete_account_removes_only_that_row() {
    let adapter = setup_adapter().await;
    let user_id = seed_user(&adapter, "carol@example.com").await;
    let drop = adapter
        .create_account(new_account(&user_id, "google", "g-drop"))
        .await
        .expect("create_account");
    let keep = adapter
        .create_account(new_account(&user_id, "github", "gh-keep"))
        .await
        .expect("create_account");

    adapter
        .delete_account(&drop.id)
        .await
        .expect("delete_account");

    assert!(
        adapter
            .get_account("google", "g-drop")
            .await
            .expect("query")
            .is_none()
    );
    assert!(
        adapter
            .get_account("github", "gh-keep")
            .await
            .expect("query")
            .is_some(),
        "the other provider is untouched: {keep:?}"
    );
}
