use teloxide::{ dispatching::DpHandlerDescription, prelude::*, types::{ChatMemberKind, ChatMemberStatus}, RequestError };
use dotenv::dotenv;
use std::env;
use sea_orm::{ ActiveModelTrait, Database, DatabaseConnection, EntityTrait, Set };
mod db;
use db::{ User, UserModel };
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), sea_orm::DbErr>{
    pretty_env_logger::init();
    dotenv().ok();
    log::info!("Starting throw dice bot...");
    let token = env::var("BOT_TOKEN").expect("BOT_TOKEN must be set");
    let pg_user = env::var("POSTGRES_USER").expect("POSTGRES_USER must be set");
    let pg_password = env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD must be set");
    let pg_host = env::var("DB_HOST").expect("DB_PORT must be set");
    let pg_port = env::var("DB_PORT").expect("DB_PORT must be set");
    let pg_db = env::var("POSTGRES_DB").expect("POSTGRES_DB must be set");
    let connection_string = format!("postgres://{}:{}@{}:{}/{}", pg_user, pg_password, pg_host, pg_port, pg_db);
    log::info!("Establishing db connection...");

    let db: DatabaseConnection = Database::connect(connection_string).await?;
    log::info!("Db connection esteblished!");
    let bot = Bot::new(token);

    Dispatcher::builder(bot, handler())
    .enable_ctrlc_handler()
    .dependencies(dptree::deps![db.clone()])
    .build()
    .dispatch()
    .await;

    Ok(())
}

fn handler() -> Handler<'static, DependencyMap, Result<(), RequestError>, DpHandlerDescription> {
    dptree::entry().branch(Update::filter_my_chat_member().branch(dptree::entry().endpoint(handle_chat_member_update)))
}


async fn handle_chat_member_update(bot: Bot, update: ChatMemberUpdated, db: DatabaseConnection) -> Result<(), teloxide::RequestError> {
    if let (ChatMemberKind::Left, ChatMemberKind::Administrator { .. }) = 
        (update.old_chat_member.kind, update.new_chat_member.kind) {
        let chat_id = update.chat.id;
        let admins = bot.get_chat_administrators(chat_id).await?;
        let owner = admins.iter().find(|admin| admin.status() == ChatMemberStatus::Owner);
        match owner {
            Some(owner) => {
                let telegram_id = owner.user.id.0.try_into().unwrap();
                let date_now = Utc::now().into();
                let username = owner.user.username.clone().unwrap_or_default();
                let first_name = owner.user.first_name.clone();
                let last_name = owner.user.last_name.clone().unwrap_or_default();
                let user_exists = User::find_by_id(telegram_id)
                    .one(&db)
                    .await.unwrap();

                match user_exists {
                    Some(exists) => {
                        let mut user: UserModel = exists.into();
                        user.last_active_at = Set(date_now);
                        user.first_name = Set(Some(first_name));
                        user.last_name = Set(Some(last_name));
                        user.save(&db).await.ok();
                    },
                    None => {
                        let user = UserModel {
                            telegram_id: Set(telegram_id),  // Обязательно `Set()`
                            username: Set(username),
                            first_name: Set(Some(first_name)),
                            last_name: Set(Some(last_name)),
                            created_at: Set(date_now),      // Дата сейчас
                            last_active_at: Set(date_now),  // Дата сейчас
                        };
                        match user.insert(&db).await {
                            Ok(_) => {
                                log::info!("Пользователь {} добавлен в базу данных", telegram_id);
                            },
                            Err(err) => {
                                log::error!("Ошибка при добавлении пользователя {} в базу данных: {:?}", telegram_id, err);
                            }
                        };
                    }
                }

            },
            None => {
                log::info!("В чате {} нет владельца", chat_id);
                bot.send_message(
                    update.chat.id, 
                    format!("Спасибо за назначение меня администратором! В чате {} нет владельца", chat_id)
                ).await?;
            }
        }
    }
    
    Ok(())
}