use super::PostgresAdapter;
use async_trait::async_trait;
use better_auth::types_mod::{AuthError, AuthResult, CreateSession, Session, SessionOps};
use chrono::{DateTime, Utc};
use db::entities::sessions;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, prelude::Expr};

#[async_trait]
impl SessionOps for PostgresAdapter {
    type Session = Session;

    async fn create_session(&self, data: CreateSession) -> AuthResult<Self::Session> {
        let id = uuid::Uuid::now_v7().to_string();
        let token = format!("session_{}", uuid::Uuid::now_v7());
        let now = Utc::now();

        let active_model = sessions::ActiveModel {
            id: Set(id.clone()),
            expires_at: Set(data.expires_at.into()),
            token: Set(token.clone()),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
            ip_address: Set(data.ip_address.clone()),
            user_agent: Set(data.user_agent.clone()),
            user_id: Set(data.user_id.clone()),
        };

        active_model.insert(&*self.db).await.map_err(|e| {
            AuthError::Database(better_auth::types_mod::DatabaseError::Query(e.to_string()))
        })?;

        Ok(Session {
            id,
            user_id: data.user_id,
            token,
            expires_at: data.expires_at,
            created_at: now,
            updated_at: now,
            ip_address: data.ip_address,
            user_agent: data.user_agent,
            active_organization_id: data.active_organization_id,
            impersonated_by: data.impersonated_by,
            active: true,
        })
    }

    async fn get_session(&self, token: &str) -> AuthResult<Option<Self::Session>> {
        let model = sessions::Entity::find()
            .filter(sessions::Column::Token.eq(token))
            .one(&*self.db)
            .await
            .map_err(|e| {
                AuthError::Database(better_auth::types_mod::DatabaseError::Query(e.to_string()))
            })?;

        Ok(model.map(map_model_to_session))
    }

    async fn get_user_sessions(&self, user_id: &str) -> AuthResult<Vec<Self::Session>> {
        let models = sessions::Entity::find()
            .filter(sessions::Column::UserId.eq(user_id))
            .all(&*self.db)
            .await
            .map_err(|e| {
                AuthError::Database(better_auth::types_mod::DatabaseError::Query(e.to_string()))
            })?;

        Ok(models.into_iter().map(map_model_to_session).collect())
    }

    async fn update_session_expiry(
        &self,
        token: &str,
        expires_at: DateTime<Utc>,
    ) -> AuthResult<()> {
        sessions::Entity::update_many()
            .col_expr(sessions::Column::ExpiresAt, Expr::value(expires_at))
            .col_expr(sessions::Column::UpdatedAt, Expr::value(Utc::now()))
            .filter(sessions::Column::Token.eq(token))
            .exec(&*self.db)
            .await
            .map_err(|e| {
                AuthError::Database(better_auth::types_mod::DatabaseError::Query(e.to_string()))
            })?;
        Ok(())
    }

    async fn delete_session(&self, token: &str) -> AuthResult<()> {
        sessions::Entity::delete_many()
            .filter(sessions::Column::Token.eq(token))
            .exec(&*self.db)
            .await
            .map_err(|e| {
                AuthError::Database(better_auth::types_mod::DatabaseError::Query(e.to_string()))
            })?;
        Ok(())
    }

    async fn delete_user_sessions(&self, user_id: &str) -> AuthResult<()> {
        sessions::Entity::delete_many()
            .filter(sessions::Column::UserId.eq(user_id))
            .exec(&*self.db)
            .await
            .map_err(|e| {
                AuthError::Database(better_auth::types_mod::DatabaseError::Query(e.to_string()))
            })?;
        Ok(())
    }

    async fn delete_expired_sessions(&self) -> AuthResult<usize> {
        let res = sessions::Entity::delete_many()
            .filter(sessions::Column::ExpiresAt.lt(Utc::now()))
            .exec(&*self.db)
            .await
            .map_err(|e| {
                AuthError::Database(better_auth::types_mod::DatabaseError::Query(e.to_string()))
            })?;
        Ok(res.rows_affected as usize)
    }

    async fn update_session_active_organization(
        &self,
        _token: &str,
        _organization_id: Option<&str>,
    ) -> AuthResult<Self::Session> {
        Err(AuthError::NotImplemented("OrganizationOps".to_string()))
    }
}

fn map_model_to_session(m: sessions::Model) -> Session {
    Session {
        id: m.id,
        user_id: m.user_id,
        token: m.token,
        expires_at: m.expires_at.into(),
        created_at: m.created_at.into(),
        updated_at: m.updated_at.into(),
        ip_address: m.ip_address,
        user_agent: m.user_agent,
        active_organization_id: None,
        impersonated_by: None,
        active: true,
    }
}
