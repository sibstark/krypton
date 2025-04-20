use commands::Commands;
use dotenv::dotenv;
use image::DynamicImage;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter, Set,
    prelude::Decimal,
};
use std::env;
use std::io::Cursor;
use std::sync::Arc;
use teloxide::types::InputFile;
use teloxide::{
    RequestError,
    dispatching::{
        DpHandlerDescription,
        dialogue::{InMemStorage, InMemStorageError},
    },
    prelude::*,
    types::{ChatMemberKind, ChatMemberStatus, InlineKeyboardButton, InlineKeyboardMarkup},
    utils::command::BotCommands,
};
mod commands;
mod qr;
mod state;
mod ton;
use chrono::Utc;
use db::{Channel, ChannelModel, User, UserModel};
use qr::generate_qr_code;
use state::{PayState, PriceState, ShowInfoState};
use thiserror::Error;
use ton::address_validator::is_valid_address;

type PriceDialogue = Dialogue<PriceState, InMemStorage<PriceState>>;
type PayDialog = Dialogue<PayState, InMemStorage<PayState>>;
type InfoDialog = Dialogue<ShowInfoState, InMemStorage<ShowInfoState>>;

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
    let gate_crypto_address =
        Arc::new(env::var("GATE_CRYPTO_ADDRESS").expect("GATE_CRYPTO_ADDRESS must be set"));
    let token = env::var("BOT_TOKEN").expect("BOT_TOKEN must be set");
    let connection_string = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    log::info!("Establishing db connection...");

    let db: DatabaseConnection = Database::connect(connection_string).await?;
    log::info!("Db connection esteblished!");
    let bot = Bot::new(token);

    Dispatcher::builder(bot, handler())
        .enable_ctrlc_handler()
        .dependencies(dptree::deps![
            db.clone(),
            gate_crypto_address.clone(),
            InMemStorage::<PriceState>::new(),
            InMemStorage::<ShowInfoState>::new(),
            InMemStorage::<PayState>::new()
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
        .branch(
            Update::filter_message()
                .filter_command::<Commands>()
                .branch(dptree::case![Commands::Start].endpoint(handle_start_command))
                .branch(dptree::case![Commands::Help].endpoint(handle_help_command))
                .branch(
                    dptree::case![Commands::SetPrice]
                        .enter_dialogue::<Message, InMemStorage<PriceState>, PriceState>()
                        .endpoint(start_price_dialogue),
                )
                .branch(
                    dptree::case![Commands::Info]
                        .enter_dialogue::<Message, InMemStorage<ShowInfoState>, ShowInfoState>()
                        .endpoint(start_info_dialogue),
                )
                .branch(
                    dptree::case![Commands::Pay]
                        .enter_dialogue::<Message, InMemStorage<PayState>, PayState>()
                        .endpoint(start_pay_dialogue),
                )
                .endpoint(not_implemented),
        )
        .branch(
            Update::filter_callback_query()
                .enter_dialogue::<CallbackQuery, InMemStorage<PriceState>, PriceState>()
                .branch(
                    dptree::case![PriceState::SelectChannel].endpoint(handle_channel_selection),
                ),
        )
        .branch(
            Update::filter_callback_query()
                .enter_dialogue::<CallbackQuery, InMemStorage<ShowInfoState>, ShowInfoState>()
                .branch(
                    dptree::case![ShowInfoState::SelectChannel]
                        .endpoint(handle_info_channel_selection),
                ),
        )
        .branch(
            Update::filter_message()
                .enter_dialogue::<Message, InMemStorage<PriceState>, PriceState>()
                .branch(
                    dptree::case![PriceState::EnterPrice {
                        channel_id,
                        channel_name
                    }]
                    .endpoint(handle_price_input),
                ),
        )
        .branch(
            Update::filter_message()
                .enter_dialogue::<Message, InMemStorage<PriceState>, PriceState>()
                .branch(
                    dptree::case![PriceState::EnterCryptoAddress { channel_id }]
                        .endpoint(handle_crypto_address_input),
                ),
        )
        .branch(
            Update::filter_message()
                .enter_dialogue::<Message, InMemStorage<PayState>, PayState>()
                .branch(
                    dptree::case![PayState::SelectChannel].endpoint(handle_pay_channel_selection),
                ),
        )
        .branch(
            Update::filter_callback_query()
                .enter_dialogue::<CallbackQuery, InMemStorage<PayState>, PayState>()
                .branch(dptree::case![PayState::SelectChannel].endpoint(handle_pay_button)),
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
            let channel_title = update.chat.title().unwrap().to_string();
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

async fn start_price_dialogue(
    bot: Bot,
    msg: Message,
    db: DatabaseConnection,
    dialogue: PriceDialogue,
) -> Result<(), BotError> {
    let owner_id = msg.from.unwrap().id;
    let channels: Vec<db::channel::Model> = Channel::find()
        .filter(db::channel::Column::OwnerTelegramId.eq(owner_id.0))
        .all(&db)
        .await?;
    let channels_count = channels.len();
    if channels_count == 0 {
        bot.send_message(msg.chat.id, "You have no channel ownhership.")
            .await?;
        dialogue.exit().await?;
        return Ok(());
    }
    let buttons: Vec<Vec<InlineKeyboardButton>> = channels
        .iter()
        .map(|c| {
            let channel_name = c.title.clone();
            let callback_data = format!("channel_{}", c.channel_id);
            vec![InlineKeyboardButton::callback(channel_name, callback_data)]
        })
        .collect();
    let keyboard = InlineKeyboardMarkup::new(buttons);
    bot.send_message(msg.chat.id, "Select a channel: ")
        .reply_markup(keyboard)
        .await?;
    dialogue.update(PriceState::SelectChannel).await?;
    Ok(())
}

async fn handle_channel_selection(
    bot: Bot,
    q: CallbackQuery,
    dialogue: PriceDialogue,
    db: DatabaseConnection,
) -> Result<(), BotError> {
    // First, answer the callback query to stop the loading animation
    bot.answer_callback_query(q.id).await?;
    if let Some(data) = q.data {
        if let Some(channel_id_str) = data.strip_prefix("channel_") {
            if let Ok(channel_id) = channel_id_str.parse::<i64>() {
                // Find the channel in the database
                if let Some(channel) = Channel::find_by_id(channel_id).one(&db).await? {
                    let channel_name = channel.title.clone();
                    let message = q.message.unwrap();
                    let chat = message.chat().clone();
                    let chat_id = chat.id;
                    let message_id = message.id();
                    bot.edit_message_text(
                        chat_id,
                        message_id,
                        format!("Setting price for channel: {}\n\nPlease enter the monthly subscription price in USD:", channel_name)
                    ).reply_markup(InlineKeyboardMarkup::default()) // Remove the keyboard
                    .await?;

                    // Update the dialogue state to EnterPrice
                    dialogue
                        .update(PriceState::EnterPrice {
                            channel_id,
                            channel_name,
                        })
                        .await?;

                    return Ok(());
                }
            }
        }
    }

    // If we got here, something went wrong
    if let Some(message) = q.message {
        bot.send_message(
            message.chat().id,
            "Error processing your selection. Please try again.",
        )
        .await?;
    }

    dialogue.exit().await?;
    Ok(())
}

async fn not_implemented(bot: Bot, msg: Message) -> Result<(), BotError> {
    bot.send_message(msg.chat.id, "Not implemented").await?;
    Ok(())
}

async fn handle_price_input(
    bot: Bot,
    msg: Message,
    dialogue: PriceDialogue,
    (channel_id, channel_name): (i64, String),
    db: DatabaseConnection,
) -> Result<(), BotError> {
    // Try to parse the price from the message
    if let Some(text) = msg.text() {
        if let Ok(price) = text.parse::<f64>() {
            if price <= 0.0 {
                bot.send_message(
                    msg.chat.id,
                    "Price must be greater than 0. Please enter a valid price:",
                )
                .await?;
                return Ok(());
            }

            // Update the channel price in the database
            let channel = Channel::find_by_id(channel_id).one(&db).await?;
            if let Some(channel) = channel {
                let date_now = Utc::now().into();
                let mut channel_model: ChannelModel = channel.into();
                let monthly_price = Decimal::from_f64_retain(price).unwrap().round_dp(2);
                channel_model.monthly_price = Set(Some(monthly_price));
                channel_model.last_check_date = Set(date_now);
                channel_model.update(&db).await?;
                bot.send_message(
                    msg.chat.id,
                    format!(
                        "✅ Price for channel \"{}\" has been set to USD {:.2} per month. Enter crypto address for payment:",
                        channel_name, price
                    ),
                )
                .await?;
                dialogue
                    .update(PriceState::EnterCryptoAddress { channel_id })
                    .await?;
                return Ok(());
            } else {
                bot.send_message(msg.chat.id, "Channel not found in the database.")
                    .await?;
            }
        } else {
            bot.send_message(msg.chat.id, "Please enter a valid number (e.g., 9.99):")
                .await?;
            return Ok(());
        }
    }
    Ok(())
}

async fn handle_crypto_address_input(
    bot: Bot,
    msg: Message,
    dialogue: PriceDialogue,
    channel_id: i64,
    db: DatabaseConnection,
) -> Result<(), BotError> {
    // Try to parse the price from the message
    if let Some(text) = msg.text() {
        let crypto_address = text.to_string();
        if !is_valid_address(&crypto_address) {
            bot.send_message(msg.chat.id, "Please enter a valid crypto address:")
                .await?;
            return Ok(());
        }
        let channel: Option<db::channel::Model> = Channel::find_by_id(channel_id).one(&db).await?;
        if let Some(channel) = channel {
            let title = channel.title.clone();
            let monthly_price = channel.monthly_price.clone().unwrap();
            let date_now = Utc::now().into();
            let mut channel_model: ChannelModel = channel.into();
            channel_model.last_check_date = Set(date_now);
            channel_model.crypto_address = Set(Some(crypto_address.clone()));
            channel_model.update(&db).await?;
            bot.send_message(
            msg.chat.id,
            format!(
                "✅ Channel \"{}\" has USD {:.2} per month price and {} crypto address for payment.",
                title, monthly_price, crypto_address
            )).await?;
            dialogue.exit().await?;
            return Ok(());
        }
    } else {
        bot.send_message(msg.chat.id, "Please enter a crypto address:")
            .await?;
    }
    Ok(())
}

async fn start_pay_dialogue(bot: Bot, msg: Message, dialogue: PayDialog) -> Result<(), BotError> {
    bot.send_message(
        msg.chat.id,
        "Hello! Enter channel which you want to pay subscription:",
    )
    .await?;
    dialogue.update(PayState::SelectChannel).await?;
    Ok(())
}

async fn start_info_dialogue(
    bot: Bot,
    msg: Message,
    db: DatabaseConnection,
    dialogue: InfoDialog,
) -> Result<(), BotError> {
    let owner_id = msg.from.unwrap().id;
    let channels: Vec<db::channel::Model> = Channel::find()
        .filter(db::channel::Column::OwnerTelegramId.eq(owner_id.0))
        .all(&db)
        .await?;
    let channels_count = channels.len();
    if channels_count == 0 {
        bot.send_message(msg.chat.id, "You have no channel ownhership.")
            .await?;
        dialogue.exit().await?;
        return Ok(());
    }
    let buttons: Vec<Vec<InlineKeyboardButton>> = channels
        .iter()
        .map(|c| {
            let channel_name = c.title.clone();
            let callback_data = format!("channel_{}", c.channel_id);
            vec![InlineKeyboardButton::callback(channel_name, callback_data)]
        })
        .collect();
    let keyboard = InlineKeyboardMarkup::new(buttons);
    bot.send_message(msg.chat.id, "Select a channel: ")
        .reply_markup(keyboard)
        .await?;
    dialogue.update(ShowInfoState::SelectChannel).await?;
    Ok(())
}

async fn handle_info_channel_selection(
    bot: Bot,
    q: CallbackQuery,
    db: DatabaseConnection,
    dialogue: InfoDialog,
) -> Result<(), BotError> {
    bot.answer_callback_query(q.id).await?;
    let message = q.message.unwrap();
    let chat_id = message.chat().id;

    if let Some(data) = q.data {
        if let Some(channel_id_str) = data.strip_prefix("channel_") {
            if let Ok(channel_id) = channel_id_str.parse::<i64>() {
                if let Some(channel) = Channel::find_by_id(channel_id).one(&db).await? {
                    let monthly_price = channel.monthly_price.unwrap();
                    let title = channel.title.clone();
                    let crypto_address = channel.crypto_address.clone();
                    let message_id = message.id();
                    bot.delete_message(chat_id, message_id).await?;
                    match  crypto_address {
                        Some(address) => {
                            bot.send_message(
                                chat_id,
                                format!(
                                    "✅ Channel \"{}\" has USD {:.2} per month price and {} crypto address for payment.",
                                    title, monthly_price, address
                                ),
                            ).await?;
                        }
                        None => {
                            bot.send_message(
                                chat_id,
                                format!(
                                    "✅ Channel \"{}\" has USD {:.2} per month price and no crypto address for payment.",
                                    title, monthly_price
                                ),
                            ).await?;
                        }
                    }
                    dialogue.exit().await?;
                } else {
                    bot.send_message(chat_id, "Channels not found").await?;
                }
            }
        }
    }
    Ok(())
}

async fn handle_pay_channel_selection(
    bot: Bot,
    msg: Message,
    db: DatabaseConnection,
) -> Result<(), BotError> {
    if let Some(name) = msg.text() {
        let channels: Vec<db::channel::Model> = Channel::find()
            .filter(db::channel::Column::Title.starts_with(name))
            .all(&db)
            .await?;
        if channels.len() == 0 {
            bot.send_message(msg.chat.id, "Channels not found").await?;
        } else {
            let buttons: Vec<Vec<InlineKeyboardButton>> = channels
                .iter()
                .map(|c| {
                    let channel_name = c.title.clone();
                    let callback_data = format!("channel_{}", c.channel_id);
                    vec![InlineKeyboardButton::callback(channel_name, callback_data)]
                })
                .collect();
            let keyboard = InlineKeyboardMarkup::new(buttons);
            bot.send_message(msg.chat.id, "Select a channel: ")
                .reply_markup(keyboard)
                .await?;
        }
    } else {
        bot.send_message(msg.chat.id, "Please, enter valid channel name")
            .await?;
    }
    Ok(())
}

async fn handle_pay_button(
    bot: Bot,
    q: CallbackQuery,
    dialogue: PayDialog,
    db: DatabaseConnection,
    gate_crypto_address: Arc<String>,
) -> Result<(), BotError> {
    bot.answer_callback_query(q.id).await?;
    let message = q.message.unwrap();
    let chat_id = message.chat().id;
    let message_id = message.id();
    let telegram_id = q.from.id.0.try_into().unwrap();
    if let Some(data) = q.data {
        if let Some(channel_id_str) = data.strip_prefix("channel_") {
            if let Ok(channel_id) = channel_id_str.parse::<i64>() {
                if let Some(channel) = Channel::find_by_id(channel_id).one(&db).await? {
                    bot.delete_message(chat_id, message_id).await?;
                    let price = channel.monthly_price.unwrap().try_into().unwrap();
                    let qr_code: image::ImageBuffer<image::Luma<u8>, Vec<u8>> = generate_qr_code(
                        gate_crypto_address.to_string(),
                        price,
                        telegram_id,
                        channel_id,
                    );
                    // Преобразуем QR в PNG
                    let mut png_bytes: Vec<u8> = Vec::new();
                    let dyn_image = DynamicImage::ImageLuma8(qr_code);
                    let _ = dyn_image
                        .write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)
                        .unwrap();

                    bot.send_photo(
                        chat_id,
                        InputFile::memory(png_bytes).file_name("payment_qr.png"),
                    )
                    .caption("Please complete the payment using the QR code below. Access will be granted automatically after the payment. Thank you!")
                    .await?;
                } else {
                    bot.send_message(chat_id, "Channel not found").await?;
                }
            }
        }
    }
    Ok(())
}
