use db::Channel;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use teloxide::{
    Bot,
    dispatching::{UpdateHandler, dialogue::ErasedStorage},
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
};

use super::{BotError, Commands, State as GlobalState, UserDialogue};

// Define states for the second dialogue (ShowInfoState)
#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
pub enum State {
    #[default]
    Start,
    SelectChannel,
}

pub(crate) async fn start_info_dialogue(
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
        .update(GlobalState::Info(State::SelectChannel))
        .await?;
    Ok(())
}

pub(crate) async fn handle_info_channel_selection(
    bot: Bot,
    q: CallbackQuery,
    db: DatabaseConnection,
    dialogue: UserDialogue,
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
                    match crypto_address {
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

pub(crate) fn schema() -> UpdateHandler<BotError> {
    dptree::entry()
        .branch(
            Update::filter_message()
                .filter_command::<Commands>()
                .branch(
                    dptree::case![Commands::Info]
                        .enter_dialogue::<Message, ErasedStorage<GlobalState>, GlobalState>()
                        .endpoint(start_info_dialogue),
                ),
        )
        .branch(
            Update::filter_callback_query()
                .enter_dialogue::<CallbackQuery, ErasedStorage<GlobalState>, GlobalState>()
                .filter_async(|dialogue: UserDialogue| async move {
                    matches!(
                        dialogue.get().await.ok().flatten(),
                        Some(GlobalState::Info(State::SelectChannel))
                    )
                })
                .endpoint(handle_info_channel_selection),
        )
}
