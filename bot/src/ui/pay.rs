use super::{BotError, Commands, PaymentGateway, State as GlobalState, UserDialogue};
use chrono::Utc;
use db::{Channel, TransactionModel};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use teloxide::{
    Bot,
    dispatching::{UpdateHandler, dialogue::ErasedStorage},
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
};
// Define states for the second dialogue (Pay)
#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
pub enum State {
    #[default]
    Start,
    SelectChannel {
        channel_id: Option<i64>,
    },
    Pay {
        channel_id: i64,
        channel_name: String,
    },
    PaymentStatus,
}

async fn start_pay_dialogue(
    bot: Bot,
    msg: Message,
    dialogue: UserDialogue,
) -> Result<(), BotError> {
    match msg.text() {
        Some(messate) => {
            if let Some(payload) = messate.strip_prefix("/start ") {
                if let Some(channel_id_str) = payload.strip_prefix("pay_channel_") {
                    if let Ok(channel_id) = channel_id_str.parse::<i64>() {
                        dialogue
                            .update(GlobalState::Pay(State::SelectChannel {
                                channel_id: channel_id.into(),
                            }))
                            .await?;
                        return Ok(());
                    }
                }
            }
        }
        None => {}
    }
    bot.send_message(
        msg.chat.id,
        "Hello! Enter channel which you want to pay subscription:",
    )
    .await?;
    dialogue
        .update(GlobalState::Pay(State::SelectChannel { channel_id: None }))
        .await?;
    Ok(())
}

async fn handle_pay_channel_selection(
    bot: Bot,
    msg: Message,
    channel_id: Option<i64>,
    db: DatabaseConnection,
) -> Result<(), BotError> {
    let mut channels: Vec<db::channel::Model> = vec![];

    if let Some(channel_id) = channel_id {
        channels = Channel::find_by_id(channel_id).all(&db).await?;
    } else {
        if let Some(name) = msg.text() {
            channels = Channel::find()
                .filter(db::channel::Column::Title.starts_with(name))
                .all(&db)
                .await?;
        } else {
            bot.send_message(msg.chat.id, "Please, enter valid channel name")
                .await?;
            return Ok(());
        }
    }

    if channels.len() == 0 {
        bot.send_message(msg.chat.id, "Channels not found").await?;
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
    Ok(())
}

async fn handle_pay_button(
    bot: Bot,
    q: CallbackQuery,
    db: DatabaseConnection,
    dialogue: UserDialogue,
    payment_gateway: PaymentGateway,
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
                    if channel.crypto_address.is_none() {
                        bot.send_message(chat_id, "Unfortunately, current channel doesn't have a valid crypto address valid")
                            .await?;
                        return Ok(());
                    }
                    bot.delete_message(chat_id, message_id).await?;
                    let monthly_price = channel.monthly_price.unwrap();
                    let wallet_address = channel.crypto_address.unwrap();
                    let date_now = Utc::now().into();
                    let transaction = TransactionModel {
                        telegram_id: Set(telegram_id),
                        channel_id: Set(channel_id),
                        price: Set(monthly_price),
                        status: Set("active".to_string()),
                        created_at: Set(date_now),
                        wallet_address: Set(wallet_address.clone()),
                        message_id: Set(message_id.0.into()),
                        chat_id: Set(chat_id.0),
                        currency: Set("USDT".to_string()),
                        ..Default::default()
                    };
                    let transaction_id = transaction.insert(&db).await?;
                    /*
                    let transaction_id = insert_result.id;
                    let event = PaymentEvent {
                        transaction_id,
                        telegram_id,
                        channel_id,
                        chat_id: chat_id.0,
                        price: monthly_price,
                        wallet_address: wallet_address.clone(),
                    };
                    send_payment_event(&event, &mut redis_manager).await?;
                    */
                    let link = format!("{}/{}", payment_gateway, transaction_id.id);
                    let message = format!(
                        "Please follow this link {} to proceed the action. Thank you!",
                        link
                    );
                    bot.send_message(chat_id, message).await?;
                    dialogue
                        .update(GlobalState::Pay(State::Pay {
                            channel_id,
                            channel_name: channel.title,
                        }))
                        .await?;
                } else {
                    bot.send_message(chat_id, "Channel not found").await?;
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
                .enter_dialogue::<Message, ErasedStorage<GlobalState>, GlobalState>()
                .filter_command::<Commands>()
                .branch(dptree::case![Commands::Pay(payload)].endpoint(start_pay_dialogue)),
        )
        .branch(
            Update::filter_message()
                .enter_dialogue::<Message, ErasedStorage<GlobalState>, GlobalState>()
                .branch(
                    dptree::case![GlobalState::Pay(x)].branch(
                        dptree::case![State::SelectChannel { channel_id }]
                            .endpoint(handle_pay_channel_selection),
                    ),
                ),
        )
        .branch(
            Update::filter_callback_query()
                .enter_dialogue::<CallbackQuery, ErasedStorage<GlobalState>, GlobalState>()
                .branch(dptree::case![GlobalState::Pay(x)].branch(
                    dptree::case![State::SelectChannel { channel_id }].endpoint(handle_pay_button),
                )),
        )
}
