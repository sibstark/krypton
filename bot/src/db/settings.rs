use sea_orm::entity::prelude::*;
use sea_orm::DeriveEntityModel;
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "settings")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "Text")]
    pub key: String,
    #[sea_orm(column_type = "Json", default_value = "{}")]
    pub settings: Value
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {

}

impl ActiveModelBehavior for ActiveModel {}
