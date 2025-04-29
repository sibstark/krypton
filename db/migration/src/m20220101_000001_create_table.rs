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
                        ColumnDef::new(Users::TelegramId)
                            .big_integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Users::Username).string().not_null())
                    .col(ColumnDef::new(Users::FirstName).string().null())
                    .col(ColumnDef::new(Users::LastName).string().null())
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("now()")),
                    )
                    .col(
                        ColumnDef::new(Users::LastActiveAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("now()")),
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
                        ColumnDef::new(Channels::ChannelId)
                            .big_integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Channels::LinkedChannelId)
                            .big_integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Channels::OwnerTelegramId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Channels::Title).string().not_null())
                    .col(ColumnDef::new(Channels::Description).string().null())
                    .col(ColumnDef::new(Channels::MonthlyPrice).decimal().null())
                    .col(ColumnDef::new(Channels::BotAddedAt).timestamp_with_time_zone().not_null())
                    .col(
                        ColumnDef::new(Channels::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("now()")),
                    )
                    .col(
                        ColumnDef::new(Channels::Settings)
                            .json()
                            .not_null()
                            .default(Expr::val("{}")),
                    )
                    .col(
                        ColumnDef::new(Channels::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(Channels::LastCheckDate)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("now()")),
                    )
                    .col(ColumnDef::new(Channels::CryptoAddress).string().null())
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
                    .col(
                        ColumnDef::new(Settings::Key)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Settings::Settings)
                            .json()
                            .not_null()
                            .default("{}"),
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
                    .col(
                        ColumnDef::new(InviteLinks::Id)
                            .big_integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(InviteLinks::UserId).big_integer().not_null())
                    .col(
                        ColumnDef::new(InviteLinks::ChannelId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InviteLinks::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(InviteLinks::Used).boolean().not_null())
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
                    .col(
                        ColumnDef::new(PaymentTransactions::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(PaymentTransactions::TelegramId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PaymentTransactions::ChannelId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PaymentTransactions::Price)
                            .decimal()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PaymentTransactions::Currency)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PaymentTransactions::Status)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PaymentTransactions::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("now()")),
                    )
                    .col(
                        ColumnDef::new(PaymentTransactions::CompleatedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(PaymentTransactions::TransactionData)
                            .json()
                            .not_null()
                            .default(Expr::val("{}")),
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
                    .col(
                        ColumnDef::new(ChannelMemberships::ChannelId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ChannelMemberships::TelegramId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ChannelMemberships::SubscriptionStart)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ChannelMemberships::SubscriptionEnd)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ChannelMemberships::PaymentHistory)
                            .json()
                            .not_null()
                            .default(Expr::val("[]")),
                    )
                    .col(
                        ColumnDef::new(ChannelMemberships::NotificationsSent)
                            .json()
                            .not_null()
                            .default(Expr::val("[]")),
                    )
                    .col(
                        ColumnDef::new(ChannelMemberships::Status)
                            .boolean()
                            .not_null(),
                    )
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
