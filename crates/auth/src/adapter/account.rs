use super::PostgresAdapter;
use async_trait::async_trait;
use better_auth::types_mod::{
    Account, AccountOps, AuthError, AuthResult, CreateAccount, UpdateAccount,
};
use chrono::Utc;
use db::entities::accounts;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

#[async_trait]
impl AccountOps for PostgresAdapter {
    type Account = Account;

    async fn create_account(&self, data: CreateAccount) -> AuthResult<Self::Account> {
        let id = uuid::Uuid::now_v7().to_string();
        let now = Utc::now().into();

        let active_model = accounts::ActiveModel {
            id: Set(id.clone()),
            account_id: Set(data.account_id.clone()),
            provider_id: Set(data.provider_id.clone()),
            user_id: Set(data.user_id.clone()),
            access_token: Set(data.access_token.clone()),
            refresh_token: Set(data.refresh_token.clone()),
            id_token: Set(data.id_token.clone()),
            access_token_expires_at: Set(data.access_token_expires_at.map(|d| d.into())),
            refresh_token_expires_at: Set(data.refresh_token_expires_at.map(|d| d.into())),
            scope: Set(data.scope.clone()),
            password: Set(data.password.clone()),
            created_at: Set(now),
            updated_at: Set(now),
        };

        active_model.insert(&*self.db).await.map_err(|e| {
            AuthError::Database(better_auth::types_mod::DatabaseError::Query(e.to_string()))
        })?;

        Ok(Account {
            id,
            user_id: data.user_id,
            account_id: data.account_id,
            provider_id: data.provider_id,
            access_token: data.access_token,
            refresh_token: data.refresh_token,
            id_token: data.id_token,
            access_token_expires_at: data.access_token_expires_at,
            refresh_token_expires_at: data.refresh_token_expires_at,
            scope: data.scope,
            password: data.password,
            created_at: now.into(),
            updated_at: now.into(),
        })
    }

    async fn get_account(
        &self,
        provider: &str,
        account_id: &str,
    ) -> AuthResult<Option<Self::Account>> {
        let model = accounts::Entity::find()
            .filter(accounts::Column::ProviderId.eq(provider))
            .filter(accounts::Column::AccountId.eq(account_id))
            .one(&*self.db)
            .await
            .map_err(|e| {
                AuthError::Database(better_auth::types_mod::DatabaseError::Query(e.to_string()))
            })?;

        Ok(model.map(map_model_to_account))
    }

    async fn get_user_accounts(&self, user_id: &str) -> AuthResult<Vec<Self::Account>> {
        let models = accounts::Entity::find()
            .filter(accounts::Column::UserId.eq(user_id))
            .all(&*self.db)
            .await
            .map_err(|e| {
                AuthError::Database(better_auth::types_mod::DatabaseError::Query(e.to_string()))
            })?;

        Ok(models.into_iter().map(map_model_to_account).collect())
    }

    async fn update_account(&self, _id: &str, _update: UpdateAccount) -> AuthResult<Self::Account> {
        Err(AuthError::NotImplemented("AccountOps".to_string()))
    }

    async fn delete_account(&self, id: &str) -> AuthResult<()> {
        accounts::Entity::delete_by_id(id.to_string())
            .exec(&*self.db)
            .await
            .map_err(|e| {
                AuthError::Database(better_auth::types_mod::DatabaseError::Query(e.to_string()))
            })?;
        Ok(())
    }
}

fn map_model_to_account(m: accounts::Model) -> Account {
    Account {
        id: m.id,
        user_id: m.user_id,
        account_id: m.account_id,
        provider_id: m.provider_id,
        access_token: m.access_token,
        refresh_token: m.refresh_token,
        id_token: m.id_token,
        access_token_expires_at: m.access_token_expires_at.map(|d| d.into()),
        refresh_token_expires_at: m.refresh_token_expires_at.map(|d| d.into()),
        scope: m.scope,
        password: m.password,
        created_at: m.created_at.into(),
        updated_at: m.updated_at.into(),
    }
}
