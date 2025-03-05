use sea_orm::entity::prelude::*;
use sea_orm::DeriveEntityModel;
use chrono::{ DateTime, Utc };


#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "BigInteger")]
    pub telegram_id: i64,
    
    #[sea_orm(column_type = "Text")]
    pub username: String,
    
    #[sea_orm(column_type = "Text", nullable)]
    pub first_name: Option<String>,
    
    #[sea_orm(column_type = "Text", nullable)]
    pub last_name: Option<String>,
    
    #[sea_orm(column_type = "Timestamp", default_value = "now()")]
    pub created_at: DateTime<Utc>,
    
    #[sea_orm(column_type = "Timestamp", default_value = "now()")]
    pub last_active_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
}

impl ActiveModelBehavior for ActiveModel {}