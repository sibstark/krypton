use commands::Commands;
use dotenv::dotenv;
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait, Set};
use std::env;
use teloxide::{
    RequestError,
    dispatching::DpHandlerDescription,
    dispatching::dialogue::{InMemStorage, InMemStorageError},
    prelude::*,
    types::{ChatMemberKind, ChatMemberStatus},
    utils::command::BotCommands,
};
mod commands;
mod db;
mod state;
use chrono::Utc;
use db::{Channel, ChannelModel, Settings, User, UserModel};
use state::PriceState;
use thiserror::Error;

type PriceDialogue = Dialogue<PriceState, InMemStorage<PriceState>>;

#[derive(Debug, Error)]
pub enum BotError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("Telegram API error: {0}")]
    Teloxide(#[from] RequestError),

    #[error("Dialog API error: {0}")]
    InMemStorage(#[from] InMemStorageError),
}

#[tokio::main]
async fn main() -> Result<(), BotError> {
    pretty_env_logger::init();
    dotenv().ok();
    log::info!("Starting throw dice bot...");
    let token = env::var("BOT_TOKEN").expect("BOT_TOKEN must be set");
    let pg_user = env::var("POSTGRES_USER").expect("POSTGRES_USER must be set");
    let pg_password = env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD must be set");
    let pg_host = env::var("DB_HOST").expect("DB_PORT must be set");
    let pg_port = env::var("DB_PORT").expect("DB_PORT must be set");
    let pg_db = env::var("POSTGRES_DB").expect("POSTGRES_DB must be set");
    let connection_string = format!(
        "postgres://{}:{}@{}:{}/{}",
        pg_user, pg_password, pg_host, pg_port, pg_db
    );
    log::info!("Establishing db connection...");

    let db: DatabaseConnection = Database::connect(connection_string).await?;
    log::info!("Db connection esteblished!");
    let bot = Bot::new(token);

    Dispatcher::builder(bot, handler())
        .enable_ctrlc_handler()
        .dependencies(dptree::deps![db.clone(), InMemStorage::<PriceState>::new()])
        .build()
        .dispatch()
        .await;

    Ok(())
}

fn handler() -> Handler<'static, DependencyMap, Result<(), BotError>, DpHandlerDescription> {
    dptree::entry()
        .branch(
            Update::filter_my_chat_member()
                .filter_map(|upd: ChatMemberUpdated| {
                    match (&upd.old_chat_member.kind, &upd.new_chat_member.kind) {
                        (ChatMemberKind::Left, ChatMemberKind::Administrator { .. }) => Some(upd),
                        _ => None,
                    }
                })
                .endpoint(handle_chat_member_update),
        )
        .branch(
            Update::filter_message()
                .filter_command::<Commands>()
                .branch(dptree::case![Commands::Start].endpoint(handle_start_command))
                .branch(
                    dptree::case![Commands::SetPrice]
                        .enter_dialogue::<Message, InMemStorage<PriceState>, PriceState>()
                        .endpoint(start_price_dialogue),
                )
                .endpoint(not_implemented),
        )
        .branch(Update::filter_message().endpoint(handle_chat_message))
}

async fn handle_chat_message(bot: Bot, update: Message) -> Result<(), BotError> {
    bot.send_message(update.chat.id, "Hello!").await?;
    Ok(())
}

async fn handle_chat_member_update(
    bot: Bot,
    update: ChatMemberUpdated,
    db: DatabaseConnection,
) -> Result<(), BotError> {
    let chat_id = update.chat.id;
    let admins = bot.get_chat_administrators(chat_id).await?;
    let owner = admins
        .iter()
        .find(|admin| admin.status() == ChatMemberStatus::Owner);
    match owner {
        Some(owner) => {
            let telegram_id = owner.user.id.0.try_into().unwrap();
            let date_now = Utc::now().into();
            let username = owner.user.username.clone().unwrap_or_default();
            let first_name = owner.user.first_name.clone();
            let last_name = owner.user.last_name.clone().unwrap_or_default();
            let user_exists = User::find_by_id(telegram_id).one(&db).await?;

            match user_exists {
                Some(exists) => {
                    let mut user: UserModel = exists.into();
                    user.last_active_at = Set(date_now);
                    user.first_name = Set(Some(first_name));
                    user.last_name = Set(Some(last_name));
                    user.save(&db).await?;
                }
                None => {
                    let user = UserModel {
                        telegram_id: Set(telegram_id), // Обязательно `Set()`
                        username: Set(username),
                        first_name: Set(Some(first_name)),
                        last_name: Set(Some(last_name)),
                        created_at: Set(date_now),     // Дата сейчас
                        last_active_at: Set(date_now), // Дата сейчас
                    };
                    user.insert(&db).await?;
                }
            }

            let channel_exists = Channel::find_by_id(chat_id.0).one(&db).await?;
            let channel_title = Some(update.chat.title().unwrap().to_string());
            let channel_description = update.chat.description().map(|s| s.to_string());
            let date_now = Utc::now().into();
            let chat_info = bot.get_chat(chat_id).await?;
            let linked_channel_id = chat_info.linked_chat_id();
            match channel_exists {
                Some(exists) => {
                    let mut channel: ChannelModel = exists.into();
                    channel.title = Set(channel_title);
                    channel.description = Set(channel_description);
                    channel.owner_telegram_id = Set(telegram_id);
                    channel.linked_channel_id = Set(linked_channel_id);
                    channel.update(&db).await?;
                }
                None => {
                    let channel = ChannelModel {
                        channel_id: Set(chat_id.0),
                        linked_channel_id: Set(linked_channel_id),
                        owner_telegram_id: Set(telegram_id),
                        title: Set(channel_title),
                        description: Set(channel_description),
                        monthly_price: Set(None),
                        bot_added_at: Set(date_now),
                        ..Default::default()
                    };
                    channel.insert(&db).await?;
                }
            }
        }
        None => {
            log::info!("В чате {} нет владельца", chat_id);
            bot.send_message(
                update.chat.id,
                format!(
                    "Спасибо за назначение меня администратором! В чате {} нет владельца",
                    chat_id
                ),
            )
            .await?;
        }
    }

    Ok(())
}

/*
async fn terminate_user(bot: Bot, db: DatabaseConnection) -> Result<(), BotError> {
    let main_settings = Settings::find_by_id("main").one(&db).await?;
    match main_settings {
        Some(settings) => {
            let settings: serde_json::Value = serde_json::from_value(settings.settings).unwrap();
            let terminator_id = settings["terminator_id"].as_i64().unwrap();
            // bot.invite(chat_id, user_id)
            Ok(())
        }
        None => Err(BotError::Database(sea_orm::DbErr::Custom(
            "Settings not found".to_string(),
        ))),
    }
}
*/

async fn handle_command(bot: Bot, msg: Message, cmd: Commands) -> Result<(), BotError> {
    match cmd {
        Commands::Start => {
            bot.send_message(msg.chat.id, "Hello! I'm Krypton bot.
If you're the owner, use the /setprice command to adjust the subscription price for your private Telegram channel.
Use the /pay command to pay for a subscription to a private Telegram channel.").await?;
        }
        Commands::Help => {
            bot.send_message(msg.chat.id, Commands::descriptions().to_string())
                .await?;
        }
        Commands::Pay => {
            bot.send_message(msg.chat.id, Commands::descriptions().to_string())
                .await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Not implemented").await?;
        }
    }
    Ok(())
}

async fn handle_start_command(bot: Bot, msg: Message) -> Result<(), BotError> {
    bot.send_message(msg.chat.id, "Hello! I'm Krypton bot.
    If you're the owner, use the /setprice command to adjust the subscription price for your private Telegram channel.
    Use the /pay command to pay for a subscription to a private Telegram channel.").await?;
    Ok(())
}

async fn start_price_dialogue(
    bot: Bot,
    msg: Message,
    price_dialogue: PriceDialogue,
) -> Result<(), BotError> {
    bot.send_message(msg.chat.id, "Setting a new price. Enter the channel name:")
        .await?;
    price_dialogue.update(PriceState::SearchChannel).await?;
    Ok(())
}

async fn not_implemented(bot: Bot, msg: Message) -> Result<(), BotError> {
    bot.send_message(msg.chat.id, "Not implemented").await?;
    Ok(())
}
