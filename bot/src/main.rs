// mod section;
mod qr;
mod ton;
mod ui;
//

use chrono::Utc;
use db::{Channel, ChannelModel, User, UserModel};
use dotenv::dotenv;
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait, Set};
use std::sync::Arc;
use std::{env, vec};
use teloxide::dispatching::dialogue::Storage;
use teloxide::dispatching::dialogue::serializer::{Bincode, Json};
use teloxide::types::ChatKind;
use teloxide::{
    dispatching::{
        DpHandlerDescription,
        dialogue::{ErasedStorage, RedisStorage},
    },
    prelude::*,
    types::{ChatMemberKind, ChatMemberStatus},
    utils::command::BotCommands,
};
use ui::{BotError, Commands, GateCryptoAddress, PaymentGateway, State};

type DialogueStorage = std::sync::Arc<ErasedStorage<State>>;

#[tokio::main]
async fn main() -> Result<(), BotError> {
    pretty_env_logger::init();
    dotenv().ok();
    log::info!("Starting throw dice bot...");
    let gate_crypto_address = GateCryptoAddress(Arc::new(
        env::var("GATE_CRYPTO_ADDRESS").expect("GATE_CRYPTO_ADDRESS must be set"),
    ));
    let payment_gateway = PaymentGateway(Arc::new(
        env::var("PAYMENT_GATEWAY").expect("PAYMENT_GATEWAY must be set"),
    ));
    let token = env::var("BOT_TOKEN").expect("BOT_TOKEN must be set");
    let connection_string = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let dragonfly_password =
        env::var("DRAGONFLY_PASSWORD").expect("DRAGONFLY_PASSWORD must be set");
    log::info!("Establishing db connection...");

    let db: DatabaseConnection = Database::connect(connection_string).await?;
    let redis_url = format!("redis://:{}@127.0.0.1:6379", dragonfly_password);
    // let client = Client::open(format!("redis://:{}@127.0.0.1:6379", dragonfly_password))?;
    // let manager = client.get_multiplexed_tokio_connection().await?;
    log::info!("Db connection esteblished!");
    let bot = Bot::new(token);

    let dialogue: DialogueStorage = RedisStorage::open(&redis_url.clone(), Json)
        .await
        .unwrap()
        .erase();

    Dispatcher::builder(bot, handler())
        .enable_ctrlc_handler()
        .dependencies(dptree::deps![
            db.clone(),
            gate_crypto_address.clone(),
            payment_gateway.clone(),
            dialogue
        ])
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
        .branch(ui::info::schema())
        //.branch(ui::pay::schema())
        //.branch(ui::price::schema())
        .branch(
            Update::filter_message()
                .filter_command::<Commands>()
                .branch(dptree::case![Commands::Start].endpoint(handle_start_command))
                .branch(dptree::case![Commands::Help].endpoint(handle_help_command))
                .endpoint(not_implemented),
        ).branch(Update::filter_message().endpoint(handle_chat_message))

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
            let channel_title = update.chat.title().unwrap().to_string();
            let channel_description = match update.chat.kind {
                ChatKind::Private(_) => None,
                ChatKind::Public(chat) => chat.title,
            };
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

async fn handle_start_command(bot: Bot, msg: Message) -> Result<(), BotError> {
    bot.send_message(msg.chat.id, "Hello! I'm Krypton bot.
    If you're the owner, use the /setprice command to adjust the subscription price for your private Telegram channel.
    Use the /pay command to pay for a subscription to a private Telegram channel.").await?;
    Ok(())
}

async fn handle_help_command(bot: Bot, msg: Message) -> Result<(), BotError> {
    bot.send_message(msg.chat.id, Commands::descriptions().to_string())
        .await?;
    Ok(())
}

async fn not_implemented(bot: Bot, msg: Message) -> Result<(), BotError> {
    bot.send_message(msg.chat.id, "Not implemented").await?;
    Ok(())
}
