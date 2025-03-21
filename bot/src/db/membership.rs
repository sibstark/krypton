use sea_orm::entity::prelude::*;
use chrono::{ DateTime, Utc };
use sea_orm::DeriveEntityModel;
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "channel_memberships")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "BigInteger")]
    pub channel_id: i64,
    #[sea_orm(primary_key, column_type = "BigInteger" )]
    pub telegram_id: i64,
    #[sea_orm(column_type = "Timestamp")]
    pub subscription_start: DateTime<Utc>,
    #[sea_orm(column_type = "Timestamp")]
    pub subscription_end: DateTime<Utc>,
    #[sea_orm(column_type = "Json", default_value="[]")]
    pub payment_history: Value,
    #[sea_orm(column_type = "Json", default_value="[]")]
    pub notifications_sent: Value,
    #[sea_orm(column_type = "Boolean")]
    pub status: bool
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Channel,
    User
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
            .into()
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
