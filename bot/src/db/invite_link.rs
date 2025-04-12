use chrono::{DateTime, Utc};
use sea_orm::DeriveEntityModel;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "invite_links")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "BigInteger")]
    pub id: i64,
    #[sea_orm(column_type = "BigInteger")]
    pub user_id: i64,
    #[sea_orm(column_type = "BigInteger")]
    pub channel_id: i64,
    #[sea_orm(column_type = "Timestamp")]
    pub expires_at: DateTime<Utc>,
    #[sea_orm(column_type = "Boolean")]
    pub used: bool
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
                .from(Column::UserId)
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
