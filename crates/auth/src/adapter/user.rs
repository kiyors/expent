use super::PostgresAdapter;
use async_trait::async_trait;
use better_auth::types_mod::{
    AuthError, AuthResult, CreateUser, ListUsersParams, UpdateUser, User, UserOps,
};
use chrono::{DateTime, FixedOffset, Utc};
use db::entities::enums::GroupRole;
use db::entities::users;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use std::str::FromStr;

#[async_trait]
impl UserOps for PostgresAdapter {
    type User = User;

    async fn create_user(&self, data: CreateUser) -> AuthResult<Self::User> {
        let id = data
            .id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
        let now: DateTime<FixedOffset> = Utc::now().into();

        let role = data
            .role
            .and_then(|r| GroupRole::from_str(&r.to_uppercase()).ok())
            .or(Some(GroupRole::Member));

        let active_model = users::ActiveModel {
            id: Set(id.clone()),
            name: Set(data.name.unwrap_or_default()),
            email: Set(data.email.unwrap_or_default()),
            email_verified: Set(data.email_verified.unwrap_or(false)),
            image: Set(data.image),
            phone: Set(None),
            is_active: Set(true),
            created_at: Set(now),
            updated_at: Set(now),
            username: Set(data.username),
            display_username: Set(data.display_username),
            role: Set(role),
            banned: Set(Some(false)),
            ban_reason: Set(None),
            ban_expires: Set(None),
            two_factor_enabled: Set(Some(false)),
            phone_number: Set(None),
            phone_number_verified: Set(Some(false)),
            associated_contact_id: Set(None),
            metadata: Set(data.metadata),
        };

        active_model.insert(&*self.db).await.map_err(|e| {
            AuthError::Database(better_auth::types_mod::DatabaseError::Query(e.to_string()))
        })?;

        self.get_user_by_id(&id)
            .await?
            .ok_or(AuthError::UserNotFound)
    }

    async fn get_user_by_id(&self, id: &str) -> AuthResult<Option<Self::User>> {
        let model = users::Entity::find_by_id(id.to_string())
            .one(&*self.db)
            .await
            .map_err(|e| {
                AuthError::Database(better_auth::types_mod::DatabaseError::Query(e.to_string()))
            })?;

        Ok(model.map(map_model_to_user))
    }

    async fn get_user_by_email(&self, email: &str) -> AuthResult<Option<Self::User>> {
        let model = users::Entity::find()
            .filter(users::Column::Email.eq(email))
            .one(&*self.db)
            .await
            .map_err(|e| {
                AuthError::Database(better_auth::types_mod::DatabaseError::Query(e.to_string()))
            })?;

        Ok(model.map(map_model_to_user))
    }

    async fn get_user_by_username(&self, username: &str) -> AuthResult<Option<Self::User>> {
        let model = users::Entity::find()
            .filter(users::Column::Username.eq(username))
            .one(&*self.db)
            .await
            .map_err(|e| {
                AuthError::Database(better_auth::types_mod::DatabaseError::Query(e.to_string()))
            })?;

        Ok(model.map(map_model_to_user))
    }

    async fn update_user(&self, id: &str, update: UpdateUser) -> AuthResult<Self::User> {
        let model = users::Entity::find_by_id(id.to_string())
            .one(&*self.db)
            .await
            .map_err(|e| {
                AuthError::Database(better_auth::types_mod::DatabaseError::Query(e.to_string()))
            })?
            .ok_or(AuthError::UserNotFound)?;

        let mut active_model: users::ActiveModel = model.into();
        active_model.updated_at = Set(Utc::now().into());

        if let Some(name) = update.name {
            active_model.name = Set(name);
        }
        if let Some(email) = update.email {
            active_model.email = Set(email);
        }
        if let Some(ev) = update.email_verified {
            active_model.email_verified = Set(ev);
        }
        if let Some(image) = update.image {
            active_model.image = Set(Some(image));
        }
        if let Some(username) = update.username {
            active_model.username = Set(Some(username));
        }
        if let Some(role) = update.role {
            active_model.role = Set(GroupRole::from_str(&role.to_uppercase()).ok());
        }
        if let Some(banned) = update.banned {
            active_model.banned = Set(Some(banned));
        }
        if let Some(ban_reason) = update.ban_reason {
            active_model.ban_reason = Set(Some(ban_reason));
        }
        if let Some(ban_expires) = update.ban_expires {
            active_model.ban_expires = Set(Some(ban_expires.into()));
        }

        active_model.update(&*self.db).await.map_err(|e| {
            AuthError::Database(better_auth::types_mod::DatabaseError::Query(e.to_string()))
        })?;

        self.get_user_by_id(id)
            .await?
            .ok_or(AuthError::UserNotFound)
    }

    async fn delete_user(&self, id: &str) -> AuthResult<()> {
        users::Entity::delete_by_id(id.to_string())
            .exec(&*self.db)
            .await
            .map_err(|e| {
                AuthError::Database(better_auth::types_mod::DatabaseError::Query(e.to_string()))
            })?;
        Ok(())
    }

    async fn list_users(&self, _params: ListUsersParams) -> AuthResult<(Vec<Self::User>, usize)> {
        let models = users::Entity::find()
            .order_by_desc(users::Column::CreatedAt)
            .all(&*self.db)
            .await
            .map_err(|e| {
                AuthError::Database(better_auth::types_mod::DatabaseError::Query(e.to_string()))
            })?;

        let count = models.len();
        let users = models.into_iter().map(map_model_to_user).collect();
        Ok((users, count))
    }
}

fn map_model_to_user(m: users::Model) -> User {
    let mut metadata = m.metadata.unwrap_or_default();

    // Inject associated_contact_id into metadata for better-auth to see it
    if let (Some(contact_id), Some(obj)) = (m.associated_contact_id, metadata.as_object_mut()) {
        obj.insert(
            "associated_contact_id".to_string(),
            serde_json::Value::String(contact_id),
        );
    }

    User {
        id: m.id,
        name: Some(m.name),
        email: Some(m.email),
        email_verified: m.email_verified,
        image: m.image,
        created_at: m.created_at.into(),
        updated_at: m.updated_at.into(),
        username: m.username,
        display_username: m.display_username,
        two_factor_enabled: m.two_factor_enabled.unwrap_or(false),
        role: m.role.map(|r| r.to_string()),
        banned: m.banned.unwrap_or(false),
        ban_reason: m.ban_reason,
        ban_expires: m.ban_expires.map(|d| d.into()),
        metadata,
    }
}
