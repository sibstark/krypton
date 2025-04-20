use chrono::{DateTime, Utc};
use sea_orm::DeriveEntityModel;
use sea_orm::entity::prelude::*;
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "subscriptions")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "BigInteger")]
    pub telegram_id: i64,
    #[sea_orm(primary_key, column_type = "BigInteger")]
    pub channel_id: i64,
    #[sea_orm(primary_key, column_type = "BigInteger")]
    pub transaction_id: i64,
    #[sea_orm(column_type = "Text")]
    pub status: String,
    #[sea_orm(column_type = "Timestamp")]
    pub time_from: DateTime<Utc>,
    #[sea_orm(column_type = "Timestamp")]
    pub time_to: DateTime<Utc>,
    #[sea_orm(column_type = "Timestamp", default_value = "now()")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(
        column_type = "Timestamp",
        default_value = "now()",
        on_update = "now()"
    )]
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Transaction,
    Channel,
    User,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::User => Entity::belongs_to(super::user::Entity)
                .from(Column::TelegramId)
                .to(super::user::Column::TelegramId)
                .into(),
            Self::Channel => Entity::belongs_to(super::channel::Entity)
                .from(Column::ChannelId)
                .to(super::channel::Column::ChannelId)
                .into(),
            Self::Transaction => Entity::belongs_to(super::transaction::Entity)
                .from(Column::TransactionId)
                .to(super::transaction::Column::Id)
                .into(),
        }
    }
}

impl Related<super::Channel> for Entity {
    fn to() -> RelationDef {
        Relation::Channel.def()
    }
}

impl Related<super::User> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::Transaction> for Entity {
    fn to() -> RelationDef {
        Relation::Transaction.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
