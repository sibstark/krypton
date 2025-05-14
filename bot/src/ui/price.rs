use super::{BotError, Commands, State as GlobalState, UserDialogue};
use crate::ton::address_validator::is_valid_address;
use chrono::Utc;
use db::{Channel, ChannelModel};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    prelude::Decimal,
};
use teloxide::{
    Bot,
    dispatching::{UpdateHandler, dialogue::ErasedStorage},
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
};

// Define states for the second dialogue (SetPrice)
#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
pub enum State {
    #[default]
    Start,
    SelectChannel,
    EnterPrice {
        channel_id: i64,
        channel_name: String,
    },
    EnterCryptoAddress {
        channel_id: i64,
    },
}

pub(crate) async fn start_price_dialogue(
    bot: Bot,
    msg: Message,
    db: DatabaseConnection,
    dialogue: UserDialogue,
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
    dialogue
        .update(GlobalState::Price(State::SelectChannel))
        .await?;
    Ok(())
}

async fn handle_channel_selection(
    bot: Bot,
    q: CallbackQuery,
    dialogue: UserDialogue,
    db: DatabaseConnection,
) -> Result<(), BotError> {
    // First, answer the callback query to stop the loading animation
    bot.answer_callback_query(q.id).await?;
    if let Some(data) = q.data {
        if let Some(channel_id_str) = data.strip_prefix("channel_") {
            if let Ok(channel_id) = channel_id_str.parse::<i64>() {
                // Find the channel in the database
                if let Some(channel) = Channel::find_by_id(channel_id).one(&db).await? {
                    let owner_telegram_id = channel.owner_telegram_id;
                    let user_id: i64 = q.from.id.0.try_into().unwrap();
                    if owner_telegram_id != user_id {
                        bot.send_message(q.from.id, "You are not the owner of this channel.")
                            .await?;
                        return Ok(());
                    }
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
                        .update(GlobalState::Price(State::EnterPrice {
                            channel_id,
                            channel_name,
                        }))
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

pub(crate) async fn handle_price_input(
    bot: Bot,
    msg: Message,
    dialogue: UserDialogue,
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
                    .update(GlobalState::Price(State::EnterCryptoAddress { channel_id }))
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
    dialogue: UserDialogue,
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

pub(crate) fn schema() -> UpdateHandler<BotError> {
    dptree::entry()
        .branch(
            Update::filter_message()
                .enter_dialogue::<Message, ErasedStorage<GlobalState>, GlobalState>()
                .filter_command::<Commands>()
                .branch(dptree::case![Commands::SetPrice].endpoint(start_price_dialogue)),
        )
        .branch(
            Update::filter_message()
                .enter_dialogue::<Message, ErasedStorage<GlobalState>, GlobalState>()
                .branch(
                    dptree::case![GlobalState::Price(x)]
                        .branch(
                            dptree::case![State::EnterPrice {
                                channel_id,
                                channel_name
                            }]
                            .endpoint(handle_price_input),
                        )
                        .branch(
                            dptree::case![State::EnterCryptoAddress { channel_id }]
                                .endpoint(handle_crypto_address_input),
                        ),
                ),
        )
        .branch(
            Update::filter_callback_query()
                .enter_dialogue::<CallbackQuery, ErasedStorage<GlobalState>, GlobalState>()
                .branch(dptree::case![GlobalState::Price(x)].branch(
                    dptree::case![State::SelectChannel].endpoint(handle_channel_selection),
                )),
        )
}
