use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Index for transactions filtering by user and excluding deleted
        manager
            .create_index(
                Index::create()
                    .name("idx-transactions-user-not-deleted")
                    .table(Transactions::Table)
                    .col(Transactions::UserId)
                    .col(Transactions::DeletedAt)
                    .to_owned(),
            )
            .await?;

        // Index for txn_parties role filtering
        manager
            .create_index(
                Index::create()
                    .name("idx-txn_parties-role-contact")
                    .table(TxnParties::Table)
                    .col(TxnParties::Role)
                    .col(TxnParties::ContactId)
                    .to_owned(),
            )
            .await?;

        // Index for p2p_requests status filtering
        manager
            .create_index(
                Index::create()
                    .name("idx-p2p_requests-receiver-status")
                    .table(P2PRequests::Table)
                    .col(P2PRequests::ReceiverEmail)
                    .col(P2PRequests::Status)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx-transactions-user-not-deleted")
                    .table(Transactions::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name("idx-txn_parties-role-contact")
                    .table(TxnParties::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name("idx-p2p_requests-receiver-status")
                    .table(P2PRequests::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Transactions {
    Table,
    UserId,
    DeletedAt,
}

#[derive(DeriveIden)]
enum TxnParties {
    Table,
    Role,
    ContactId,
}

#[derive(DeriveIden)]
enum P2PRequests {
    Table,
    ReceiverEmail,
    Status,
}
