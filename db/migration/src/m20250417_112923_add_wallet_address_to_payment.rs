use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(PaymentTransactions::Table)
                    .add_column(ColumnDef::new(PaymentTransactions::WalletAddress).text().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(PaymentTransactions::Table)
                    .rename_column(
                        PaymentTransactions::CompleatedAt,
                        PaymentTransactions::CompletedAt,
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(PaymentTransactions::Table)
                    .drop_column(PaymentTransactions::WalletAddress)
                    .to_owned(),
            )
            .await?;

        // 2. Переименовываем обратно
        manager
            .alter_table(
                Table::alter()
                    .table(PaymentTransactions::Table)
                    .rename_column(
                        PaymentTransactions::CompletedAt,
                        PaymentTransactions::CompleatedAt,
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum PaymentTransactions {
    Table,
    WalletAddress,
    CompleatedAt,
    CompletedAt,
}
