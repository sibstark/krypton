use sea_orm::entity::prelude::*;
use chrono::{ DateTime, Utc };
use sea_orm::DeriveEntityModel;
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "channels")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "BigInteger")]
    pub channel_id: i64,
    #[sea_orm(column_type = "BigInteger")]
    pub linked_channel_id: Option<i64>,
    // refers to user.telegram_id
    #[sea_orm(column_type = "BigInteger")]
    pub owner_telegram_id:i64,
    #[sea_orm(column_type = "Text")]
    pub title: String,
    #[sea_orm(column_type = "Text")]
    pub description: Option<String>,
    #[sea_orm(column_type = "Decimal(None)")]
    pub monthly_price: Option<Decimal>,
    #[sea_orm(column_type = "Timestamp")]
    pub bot_added_at: DateTime<Utc>,
    #[sea_orm(column_type = "Timestamp", default_value = "now()")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "Json", default_value = "{}")]
    pub settings: Value,
    #[sea_orm(column_type = "Boolean", default_value = "true")]
    pub is_active: bool,
    #[sea_orm(column_type = "Timestamp", default_value = "now()")]
    pub last_check_date: DateTime<Utc>,
    #[sea_orm(column_type = "Text")]
    pub crypto_address: Option<String>
}


#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    User
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::User => Entity::belongs_to(super::user::Entity)
                .from(Column::OwnerTelegramId)
                .to(super::user::Column::TelegramId)
                .into(),
        }
    }
}

impl Related<super::User> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}