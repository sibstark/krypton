use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Subscriptions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Subscriptions::TelegramId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Subscriptions::ChannelId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Subscriptions::TransactionId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Subscriptions::Status).string().not_null())
                    .col(
                        ColumnDef::new(Subscriptions::TimeFrom)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Subscriptions::TimeTo)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Subscriptions::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("now()")),
                    )
                    .col(
                        ColumnDef::new(Subscriptions::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("now()")),
                    )
                    .primary_key(
                        Index::create()
                            .col(Subscriptions::ChannelId)
                            .col(Subscriptions::TelegramId)
                            .col(Subscriptions::TransactionId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Subscriptions::Table, Subscriptions::TelegramId)
                            .to(Users::Table, Users::TelegramId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Subscriptions::Table, Subscriptions::ChannelId)
                            .to(Channels::Table, Channels::ChannelId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Subscriptions::Table, Subscriptions::TransactionId)
                            .to(PaymentTransactions::Table, PaymentTransactions::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Subscriptions::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Subscriptions {
    Table,
    TelegramId,
    ChannelId,
    TransactionId,
    Status,
    TimeFrom,
    TimeTo,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Channels {
    Table,
    ChannelId,
}
#[derive(DeriveIden)]
enum Users {
    Table,
    TelegramId,
}

#[derive(DeriveIden)]
enum PaymentTransactions {
    Table,
    Id,
}
