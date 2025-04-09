use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Таблица users
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        big_integer()
                            .not_null()
                            .primary_key()
                            .name(Users::TelegramId),
                    )
                    .col(string().not_null().name(Users::Username))
                    .col(string().null().name(Users::FirstName))
                    .col(string().null().name(Users::LastName))
                    .col(
                        timestamp()
                            .not_null()
                            .default(Expr::cust("now()"))
                            .name(Users::CreatedAt),
                    )
                    .col(
                        timestamp()
                            .not_null()
                            .default(Expr::cust("now()"))
                            .name(Users::LastActiveAt),
                    )
                    .to_owned(),
            )
            .await?;

        // Таблица channels
        manager
            .create_table(
                Table::create()
                    .table(Channels::Table)
                    .if_not_exists()
                    .col(
                        big_integer()
                            .not_null()
                            .primary_key()
                            .name(Channels::ChannelId),
                    )
                    .col(big_integer().null().name(Channels::LinkedChannelId))
                    .col(big_integer().not_null().name(Channels::OwnerTelegramId))
                    .col(string().not_null().name(Channels::Title))
                    .col(string().null().name(Channels::Description))
                    .col(decimal().null().name(Channels::MonthlyPrice))
                    .col(timestamp().not_null().name(Channels::BotAddedAt))
                    .col(
                        timestamp()
                            .not_null()
                            .default(Expr::cust("now()"))
                            .name(Channels::CreatedAt),
                    )
                    .col(
                        json()
                            .not_null()
                            .default(Expr::val(json!({})))
                            .name(Channels::Settings),
                    )
                    .col(boolean().not_null().default(true).name(Channels::IsActive))
                    .col(
                        timestamp()
                            .not_null()
                            .default(Expr::cust("now()"))
                            .name(Channels::LastCheckDate),
                    )
                    .col(string().not_null().name(Channels::CryptoAddress))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Channels::Table, Channels::OwnerTelegramId)
                            .to(Users::Table, Users::TelegramId),
                    )
                    .to_owned(),
            )
            .await?;

        // Таблица settings
        manager
            .create_table(
                Table::create()
                    .table(Settings::Table)
                    .if_not_exists()
                    .col(string().not_null().primary_key().name(Settings::Key))
                    .col(
                        json()
                            .not_null()
                            .default(Expr::val(json!({})))
                            .name(Settings::Settings),
                    )
                    .to_owned(),
            )
            .await?;

        // Таблица invite_links
        manager
            .create_table(
                Table::create()
                    .table(InviteLinks::Table)
                    .if_not_exists()
                    .col(big_integer().not_null().name(InviteLinks::Id))
                    .col(big_integer().not_null().name(InviteLinks::UserId))
                    .col(big_integer().not_null().name(InviteLinks::ChannelId))
                    .col(timestamp().not_null().name(InviteLinks::ExpiresAt))
                    .col(boolean().not_null().name(InviteLinks::Used))
                    .primary_key(
                        Index::create()
                            .col(InviteLinks::Id)
                            .col(InviteLinks::UserId)
                            .col(InviteLinks::ChannelId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(InviteLinks::Table, InviteLinks::UserId)
                            .to(Users::Table, Users::TelegramId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(InviteLinks::Table, InviteLinks::ChannelId)
                            .to(Channels::Table, Channels::ChannelId),
                    )
                    .to_owned(),
            )
            .await?;

        // Таблица payment_transactions
        manager
            .create_table(
                Table::create()
                    .table(PaymentTransactions::Table)
                    .if_not_exists()
                    .col(big_integer().not_null().name(PaymentTransactions::Id))
                    .col(
                        big_integer()
                            .not_null()
                            .name(PaymentTransactions::TelegramId),
                    )
                    .col(
                        big_integer()
                            .not_null()
                            .name(PaymentTransactions::ChannelId),
                    )
                    .col(decimal().not_null().name(PaymentTransactions::Price))
                    .col(string().not_null().name(PaymentTransactions::Currency))
                    .col(string().not_null().name(PaymentTransactions::Status))
                    .col(
                        timestamp()
                            .not_null()
                            .default(Expr::cust("now()"))
                            .name(PaymentTransactions::CreatedAt),
                    )
                    .col(timestamp().null().name(PaymentTransactions::CompleatedAt))
                    .col(
                        json()
                            .not_null()
                            .default(Expr::val(json!({})))
                            .name(PaymentTransactions::TransactionData),
                    )
                    .primary_key(
                        Index::create()
                            .col(PaymentTransactions::Id)
                            .col(PaymentTransactions::TelegramId)
                            .col(PaymentTransactions::ChannelId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(PaymentTransactions::Table, PaymentTransactions::TelegramId)
                            .to(Users::Table, Users::TelegramId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(PaymentTransactions::Table, PaymentTransactions::ChannelId)
                            .to(Channels::Table, Channels::ChannelId),
                    )
                    .to_owned(),
            )
            .await?;

        // Таблица channel_memberships
        manager
            .create_table(
                Table::create()
                    .table(ChannelMemberships::Table)
                    .if_not_exists()
                    .col(big_integer().not_null().name(ChannelMemberships::ChannelId))
                    .col(
                        big_integer()
                            .not_null()
                            .name(ChannelMemberships::TelegramId),
                    )
                    .col(
                        timestamp()
                            .not_null()
                            .name(ChannelMemberships::SubscriptionStart),
                    )
                    .col(
                        timestamp()
                            .not_null()
                            .name(ChannelMemberships::SubscriptionEnd),
                    )
                    .col(
                        json()
                            .not_null()
                            .default(Expr::val(json!([])))
                            .name(ChannelMemberships::PaymentHistory),
                    )
                    .col(
                        json()
                            .not_null()
                            .default(Expr::val(json!([])))
                            .name(ChannelMemberships::NotificationsSent),
                    )
                    .col(boolean().not_null().name(ChannelMemberships::Status))
                    .primary_key(
                        Index::create()
                            .col(ChannelMemberships::ChannelId)
                            .col(ChannelMemberships::TelegramId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ChannelMemberships::Table, ChannelMemberships::TelegramId)
                            .to(Users::Table, Users::TelegramId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ChannelMemberships::Table, ChannelMemberships::ChannelId)
                            .to(Channels::Table, Channels::ChannelId),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ChannelMemberships::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PaymentTransactions::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(InviteLinks::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Settings::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Channels::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    TelegramId,
    Username,
    FirstName,
    LastName,
    CreatedAt,
    LastActiveAt,
}

#[derive(DeriveIden)]
enum Channels {
    Table,
    ChannelId,
    LinkedChannelId,
    OwnerTelegramId,
    Title,
    Description,
    MonthlyPrice,
    BotAddedAt,
    CreatedAt,
    Settings,
    IsActive,
    LastCheckDate,
    CryptoAddress,
}

#[derive(DeriveIden)]
enum Settings {
    Table,
    Key,
    Settings,
}

#[derive(DeriveIden)]
enum InviteLinks {
    Table,
    Id,
    UserId,
    ChannelId,
    ExpiresAt,
    Used,
}

#[derive(DeriveIden)]
enum PaymentTransactions {
    Table,
    Id,
    TelegramId,
    ChannelId,
    Price,
    Currency,
    Status,
    CreatedAt,
    CompleatedAt,
    TransactionData,
}

#[derive(DeriveIden)]
enum ChannelMemberships {
    Table,
    ChannelId,
    TelegramId,
    SubscriptionStart,
    SubscriptionEnd,
    PaymentHistory,
    NotificationsSent,
    Status,
}
