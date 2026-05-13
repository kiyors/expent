use super::PostgresAdapter;
use async_trait::async_trait;
use better_auth::types_mod::{
    AuthError, AuthResult, CreateVerification, Verification, VerificationOps,
};
use chrono::Utc;
use db::entities::verifications;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

#[async_trait]
impl VerificationOps for PostgresAdapter {
    type Verification = Verification;

    async fn create_verification(
        &self,
        data: CreateVerification,
    ) -> AuthResult<Self::Verification> {
        let id = uuid::Uuid::now_v7().to_string();
        let now = Utc::now();

        let active_model = verifications::ActiveModel {
            id: Set(id.clone()),
            identifier: Set(data.identifier.clone()),
            value: Set(data.value.clone()),
            expires_at: Set(data.expires_at.into()),
            created_at: Set(Some(now.into())),
            updated_at: Set(Some(now.into())),
        };

        active_model.insert(&*self.db).await.map_err(|e| {
            AuthError::Database(better_auth::types_mod::DatabaseError::Query(e.to_string()))
        })?;

        Ok(Verification {
            id,
            identifier: data.identifier,
            value: data.value,
            expires_at: data.expires_at,
            created_at: now,
            updated_at: now,
        })
    }

    async fn get_verification(
        &self,
        identifier: &str,
        value: &str,
    ) -> AuthResult<Option<Self::Verification>> {
        let model = verifications::Entity::find()
            .filter(verifications::Column::Identifier.eq(identifier))
            .filter(verifications::Column::Value.eq(value))
            .one(&*self.db)
            .await
            .map_err(|e| {
                AuthError::Database(better_auth::types_mod::DatabaseError::Query(e.to_string()))
            })?;

        Ok(model.map(map_model_to_verification))
    }

    async fn get_verification_by_value(
        &self,
        value: &str,
    ) -> AuthResult<Option<Self::Verification>> {
        let model = verifications::Entity::find()
            .filter(verifications::Column::Value.eq(value))
            .one(&*self.db)
            .await
            .map_err(|e| {
                AuthError::Database(better_auth::types_mod::DatabaseError::Query(e.to_string()))
            })?;

        Ok(model.map(map_model_to_verification))
    }

    async fn consume_verification(
        &self,
        identifier: &str,
        value: &str,
    ) -> AuthResult<Option<Self::Verification>> {
        let v = self.get_verification(identifier, value).await?;
        if let Some(ref ver) = v {
            self.delete_verification(&ver.id).await?;
        }
        Ok(v)
    }

    async fn delete_verification(&self, id: &str) -> AuthResult<()> {
        verifications::Entity::delete_by_id(id.to_string())
            .exec(&*self.db)
            .await
            .map_err(|e| {
                AuthError::Database(better_auth::types_mod::DatabaseError::Query(e.to_string()))
            })?;
        Ok(())
    }

    async fn delete_expired_verifications(&self) -> AuthResult<usize> {
        let res = verifications::Entity::delete_many()
            .filter(verifications::Column::ExpiresAt.lt(Utc::now()))
            .exec(&*self.db)
            .await
            .map_err(|e| {
                AuthError::Database(better_auth::types_mod::DatabaseError::Query(e.to_string()))
            })?;
        Ok(res.rows_affected as usize)
    }
}

fn map_model_to_verification(m: verifications::Model) -> Verification {
    Verification {
        id: m.id,
        identifier: m.identifier,
        value: m.value,
        expires_at: m.expires_at.into(),
        created_at: m.created_at.map(|d| d.into()).unwrap_or_else(Utc::now),
        updated_at: m.updated_at.map(|d| d.into()).unwrap_or_else(Utc::now),
    }
}
