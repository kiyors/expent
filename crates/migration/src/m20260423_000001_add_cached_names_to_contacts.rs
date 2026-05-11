use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Contacts::Table)
                    .add_column(ColumnDef::new(Contacts::NormalizedName).string().null())
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Contacts::Table)
                    .add_column(ColumnDef::new(Contacts::PhoneticName).string().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Contacts::Table)
                    .drop_column(Contacts::NormalizedName)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Contacts::Table)
                    .drop_column(Contacts::PhoneticName)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Contacts {
    Table,
    NormalizedName,
    PhoneticName,
}
