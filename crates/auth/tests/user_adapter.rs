//! Integration tests for `PostgresAdapter`'s `UserOps` implementation.
//!
//! These exercise the real SeaORM path against a migrated database so the
//! adapter layer can't silently drift from the entity schema. The fixture
//! defaults to in-memory SQLite for fast local runs.

mod common;

use better_auth::types_mod::{CreateUser, ListUsersParams, UpdateUser, UserOps};
use common::setup_adapter;

/// Convenience constructor — most tests only care about email + name.
fn new_user(email: &str, name: &str) -> CreateUser {
    CreateUser {
        id: None,
        name: Some(name.to_string()),
        email: Some(email.to_string()),
        email_verified: Some(false),
        image: None,
        password: None,
        role: None,
        username: None,
        display_username: None,
        metadata: None,
    }
}

#[tokio::test]
async fn create_then_get_by_id_round_trips() {
    let adapter = setup_adapter().await;

    let created = adapter
        .create_user(new_user("alice@example.com", "Alice"))
        .await
        .expect("create_user");
    assert!(!created.id.is_empty(), "create should assign an id");
    assert_eq!(created.email.as_deref(), Some("alice@example.com"));
    assert_eq!(created.name.as_deref(), Some("Alice"));

    let fetched = adapter
        .get_user_by_id(&created.id)
        .await
        .expect("get_user_by_id")
        .expect("user should exist after create");
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.email, created.email);
}

#[tokio::test]
async fn get_by_id_returns_none_for_unknown() {
    let adapter = setup_adapter().await;
    let result = adapter
        .get_user_by_id("does-not-exist")
        .await
        .expect("query should succeed");
    assert!(result.is_none(), "unknown id should map to None, not error");
}

#[tokio::test]
async fn get_by_email_finds_the_user() {
    let adapter = setup_adapter().await;
    let created = adapter
        .create_user(new_user("bob@example.com", "Bob"))
        .await
        .expect("create_user");

    let found = adapter
        .get_user_by_email("bob@example.com")
        .await
        .expect("get_user_by_email")
        .expect("user should be findable by email");
    assert_eq!(found.id, created.id);

    let missing = adapter
        .get_user_by_email("nope@example.com")
        .await
        .expect("query should succeed");
    assert!(missing.is_none());
}

#[tokio::test]
async fn update_user_persists_changes() {
    let adapter = setup_adapter().await;
    let created = adapter
        .create_user(new_user("carol@example.com", "Carol"))
        .await
        .expect("create_user");

    let updated = adapter
        .update_user(
            &created.id,
            UpdateUser {
                name: Some("Carol Updated".to_string()),
                email_verified: Some(true),
                ..Default::default()
            },
        )
        .await
        .expect("update_user");
    assert_eq!(updated.name.as_deref(), Some("Carol Updated"));
    assert!(updated.email_verified);

    // Re-fetching confirms the row, not just the in-memory return value, was
    // updated — guards against a stale `update_user` returning the input data.
    let refetched = adapter
        .get_user_by_id(&created.id)
        .await
        .expect("get_user_by_id")
        .expect("user still exists");
    assert_eq!(refetched.name.as_deref(), Some("Carol Updated"));
    assert!(refetched.email_verified);
}

#[tokio::test]
async fn update_user_errors_for_unknown_id() {
    let adapter = setup_adapter().await;
    let err = adapter
        .update_user(
            "does-not-exist",
            UpdateUser {
                name: Some("Ghost".to_string()),
                ..Default::default()
            },
        )
        .await
        .expect_err("update_user on missing id should error");
    // The adapter must surface this as UserNotFound rather than a generic DB
    // error — better-auth callers branch on the variant.
    assert!(
        matches!(err, better_auth::types_mod::AuthError::UserNotFound),
        "expected UserNotFound, got {err:?}"
    );
}

#[tokio::test]
async fn delete_user_removes_the_row() {
    let adapter = setup_adapter().await;
    let created = adapter
        .create_user(new_user("dave@example.com", "Dave"))
        .await
        .expect("create_user");

    adapter.delete_user(&created.id).await.expect("delete_user");

    let after = adapter
        .get_user_by_id(&created.id)
        .await
        .expect("query should succeed");
    assert!(after.is_none(), "deleted user should not be re-fetchable");
}

#[tokio::test]
async fn list_users_returns_all_created_with_count() {
    let adapter = setup_adapter().await;
    for (email, name) in [
        ("eve@example.com", "Eve"),
        ("frank@example.com", "Frank"),
        ("gina@example.com", "Gina"),
    ] {
        adapter
            .create_user(new_user(email, name))
            .await
            .expect("create_user");
    }

    let (users, count) = adapter
        .list_users(ListUsersParams::default())
        .await
        .expect("list_users");
    assert_eq!(count, 3);
    assert_eq!(users.len(), 3);
    // list_users orders by created_at DESC — Gina was inserted last.
    assert_eq!(users[0].email.as_deref(), Some("gina@example.com"));
}
