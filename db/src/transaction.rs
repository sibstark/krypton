use chrono::{DateTime, Utc};
use sea_orm::DeriveEntityModel;
use sea_orm::entity::prelude::*;
use serde_json::Value;

// status = active, failed, completed
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "payment_transactions")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "BigInteger")]
    pub id: i64,
    #[sea_orm(column_type = "BigInteger")]
    pub telegram_id: i64,
    #[sea_orm(column_type = "BigInteger")]
    pub channel_id: i64,
    #[sea_orm(column_type = "BigInteger")]
    pub chat_id: i64,
    #[sea_orm(column_type = "Decimal(None)")]
    pub price: Decimal,
    #[sea_orm(column_type = "Text")]
    pub currency: String,
    #[sea_orm(column_type = "Text")]
    pub status: String,
    #[sea_orm(column_type = "Timestamp", default_value = "now()")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "Timestamp")]
    pub completed_at: Option<DateTime<Utc>>,
    #[sea_orm(column_type = "Json", default_value = "{}")]
    pub transaction_data: Value,
    #[sea_orm(column_type = "Text")]
    pub wallet_address: String,
    #[sea_orm(column_type = "BigInteger")]
    pub message_id: i64
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
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

impl ActiveModelBehavior for ActiveModel {}
