//! Integration tests for `PostgresAdapter`'s `SessionOps` implementation.
//!
//! Sessions are foreign-keyed to users, so each test first creates a user via
//! the user adapter and then exercises the session paths against that user.

mod common;

use better_auth::types_mod::{CreateSession, CreateUser, SessionOps, UserOps};
use chrono::{Duration, Utc};
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

fn new_session(user_id: &str) -> CreateSession {
    CreateSession {
        user_id: user_id.to_string(),
        expires_at: Utc::now() + Duration::hours(1),
        ip_address: Some("127.0.0.1".to_string()),
        user_agent: Some("test-agent".to_string()),
        active_organization_id: None,
        impersonated_by: None,
    }
}

#[tokio::test]
async fn create_then_get_by_token() {
    let adapter = setup_adapter().await;
    let user_id = seed_user(&adapter, "alice@example.com").await;

    let session = adapter
        .create_session(new_session(&user_id))
        .await
        .expect("create_session");
    assert!(!session.token.is_empty());
    assert_eq!(session.user_id, user_id);

    let fetched = adapter
        .get_session(&session.token)
        .await
        .expect("get_session")
        .expect("session should exist after create");
    assert_eq!(fetched.id, session.id);
    assert_eq!(fetched.user_id, user_id);
}

#[tokio::test]
async fn get_by_unknown_token_returns_none() {
    let adapter = setup_adapter().await;
    let result = adapter
        .get_session("session_does-not-exist")
        .await
        .expect("query should succeed");
    assert!(result.is_none());
}

#[tokio::test]
async fn get_user_sessions_returns_all_for_user() {
    let adapter = setup_adapter().await;
    let user_id = seed_user(&adapter, "bob@example.com").await;
    let other_id = seed_user(&adapter, "other@example.com").await;

    let mut tokens = Vec::new();
    for _ in 0..3 {
        tokens.push(
            adapter
                .create_session(new_session(&user_id))
                .await
                .expect("create_session")
                .token,
        );
    }
    // A session for a different user shouldn't leak into the result.
    adapter
        .create_session(new_session(&other_id))
        .await
        .expect("create_session");

    let sessions = adapter
        .get_user_sessions(&user_id)
        .await
        .expect("get_user_sessions");
    assert_eq!(sessions.len(), 3);
    for s in &sessions {
        assert_eq!(s.user_id, user_id);
        assert!(tokens.contains(&s.token));
    }
}

#[tokio::test]
async fn update_session_expiry_persists() {
    let adapter = setup_adapter().await;
    let user_id = seed_user(&adapter, "carol@example.com").await;
    let session = adapter
        .create_session(new_session(&user_id))
        .await
        .expect("create_session");

    let new_expiry = Utc::now() + Duration::hours(24);
    adapter
        .update_session_expiry(&session.token, new_expiry)
        .await
        .expect("update_session_expiry");

    let refetched = adapter
        .get_session(&session.token)
        .await
        .expect("get_session")
        .expect("session still exists");
    // Round-trip can lose sub-second precision through the DB, so compare to
    // the second.
    assert_eq!(
        refetched.expires_at.timestamp(),
        new_expiry.timestamp(),
        "expiry should be the freshly-set value"
    );
}

#[tokio::test]
async fn delete_session_removes_only_that_token() {
    let adapter = setup_adapter().await;
    let user_id = seed_user(&adapter, "dave@example.com").await;
    let keep = adapter
        .create_session(new_session(&user_id))
        .await
        .expect("create_session");
    let drop = adapter
        .create_session(new_session(&user_id))
        .await
        .expect("create_session");

    adapter
        .delete_session(&drop.token)
        .await
        .expect("delete_session");

    assert!(
        adapter
            .get_session(&drop.token)
            .await
            .expect("query")
            .is_none(),
        "deleted session is gone"
    );
    assert!(
        adapter
            .get_session(&keep.token)
            .await
            .expect("query")
            .is_some(),
        "other session is untouched"
    );
}

#[tokio::test]
async fn delete_user_sessions_clears_all_for_user() {
    let adapter = setup_adapter().await;
    let user_id = seed_user(&adapter, "eve@example.com").await;
    let other_id = seed_user(&adapter, "frank@example.com").await;
    for _ in 0..2 {
        adapter
            .create_session(new_session(&user_id))
            .await
            .expect("create_session");
    }
    let other_session = adapter
        .create_session(new_session(&other_id))
        .await
        .expect("create_session");

    adapter
        .delete_user_sessions(&user_id)
        .await
        .expect("delete_user_sessions");

    assert!(
        adapter
            .get_user_sessions(&user_id)
            .await
            .expect("query")
            .is_empty()
    );
    // Cross-user isolation: the other user's session must survive.
    assert!(
        adapter
            .get_session(&other_session.token)
            .await
            .expect("query")
            .is_some()
    );
}

#[tokio::test]
async fn delete_expired_sessions_removes_only_expired() {
    let adapter = setup_adapter().await;
    let user_id = seed_user(&adapter, "gina@example.com").await;

    // One expired (yesterday) + one still valid (tomorrow).
    let expired = adapter
        .create_session(CreateSession {
            expires_at: Utc::now() - Duration::hours(24),
            ..new_session(&user_id)
        })
        .await
        .expect("create_session");
    let valid = adapter
        .create_session(new_session(&user_id))
        .await
        .expect("create_session");

    let removed = adapter
        .delete_expired_sessions()
        .await
        .expect("delete_expired_sessions");
    assert_eq!(removed, 1, "exactly one expired session was deleted");

    assert!(
        adapter
            .get_session(&expired.token)
            .await
            .expect("query")
            .is_none()
    );
    assert!(
        adapter
            .get_session(&valid.token)
            .await
            .expect("query")
            .is_some()
    );
}
