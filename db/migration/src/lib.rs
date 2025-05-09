pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20250417_112923_add_wallet_address_to_payment;
mod m20250417_121459_add_message_id;
mod m20250418_122825_add_subscriptions_table;
mod m20250429_130451_create_chat_id_field;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20250417_112923_add_wallet_address_to_payment::Migration),
            Box::new(m20250417_121459_add_message_id::Migration),
            Box::new(m20250418_122825_add_subscriptions_table::Migration),
            Box::new(m20250429_130451_create_chat_id_field::Migration),
        ]
    }
}
