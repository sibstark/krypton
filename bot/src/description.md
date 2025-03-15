```rust
Dispatcher::builder(
        bot,
        Update::filter_message()
            .enter_dialogue::<Message, InMemStorage<State>, State>()
            .branch(dptree::case![State::Start].endpoint(start))
            .branch(dptree::case![State::ReceiveFullName].endpoint(receive_full_name))
            .branch(dptree::case![State::ReceiveAge { full_name }].endpoint(receive_age))
            .branch(
                dptree::case![State::ReceiveLocation { full_name, age }].endpoint(receive_location),
            ),
    )
```
использовать конструкцию выше 
но только именно для команды ::Pay, ::SetPrice

если такая команда то мы сбрасывает предыдущий стейт и идем по шагам
1. введите имя канала с @ в начале
2. вводим цену (только SetPrice)
3. Pay - чекаем есть ли уже подписка для такого канала
4. Чекаем название канала, если такого нет то пишем пользователю 

Пример как может быть диалог разделен на 2 разных ветки
https://claude.ai/chat/39f77f78-eeb5-4fdb-81cd-ca14955af69c
```rust
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};

// Define states for the first dialogue (Pay)
#[derive(Clone, Default)]
pub enum PayState {
    #[default]
    Start,
    ReceiveAmount,
    ReceiveConfirmation {
        amount: u32,
    },
}

// Define states for the second dialogue (SetPrice)
#[derive(Clone, Default)]
pub enum PriceState {
    #[default]
    Start,
    ReceivePrice,
    ReceiveDescription {
        price: u32,
    },
}

// Define dialogue types
type PayDialogue = Dialogue<PayState, InMemStorage<PayState>>;
type PriceDialogue = Dialogue<PriceState, InMemStorage<PriceState>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting multi-dialogue bot...");
    let bot = Bot::from_env();

    // Command handler
    let command_handler = Update::filter_message()
        .filter_command::<Command>()
        .branch(
            dptree::case![Command::Pay].endpoint(start_pay_dialogue)
        )
        .branch(
            dptree::case![Command::SetPrice].endpoint(start_price_dialogue)
        );

    // Pay dialogue
    let pay_dialogue = Update::filter_message()
        .enter_dialogue::<Message, InMemStorage<PayState>, PayState>()
        .branch(dptree::case![PayState::Start].endpoint(pay_start))
        .branch(dptree::case![PayState::ReceiveAmount].endpoint(pay_receive_amount))
        .branch(dptree::case![PayState::ReceiveConfirmation { amount }].endpoint(pay_receive_confirmation));

    // Price dialogue
    let price_dialogue = Update::filter_message()
        .enter_dialogue::<Message, InMemStorage<PriceState>, PriceState>()
        .branch(dptree::case![PriceState::Start].endpoint(price_start))
        .branch(dptree::case![PriceState::ReceivePrice].endpoint(price_receive_price))
        .branch(dptree::case![PriceState::ReceiveDescription { price }].endpoint(price_receive_description));

    Dispatcher::builder(bot, dptree::entry().branch(command_handler).branch(pay_dialogue).branch(price_dialogue))
        .dependencies(dptree::deps![
            InMemStorage::<PayState>::new(),
            InMemStorage::<PriceState>::new()
        ])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    Pay,
    SetPrice,
}

// Command handlers to start each dialogue
async fn start_pay_dialogue(bot: Bot, msg: Message, pay_dialogue: PayDialogue) -> HandlerResult {
    pay_dialogue.update(PayState::Start).await?;
    pay_start(bot, pay_dialogue, msg).await
}

async fn start_price_dialogue(bot: Bot, msg: Message, price_dialogue: PriceDialogue) -> HandlerResult {
    price_dialogue.update(PriceState::Start).await?;
    price_start(bot, price_dialogue, msg).await
}

// Pay dialogue handlers
async fn pay_start(bot: Bot, dialogue: PayDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Starting payment process. Enter amount to pay:").await?;
    dialogue.update(PayState::ReceiveAmount).await?;
    Ok(())
}

async fn pay_receive_amount(bot: Bot, dialogue: PayDialogue, msg: Message) -> HandlerResult {
    match msg.text().map(|text| text.parse::<u32>()) {
        Some(Ok(amount)) => {
            bot.send_message(
                msg.chat.id, 
                format!("You want to pay {}. Confirm? (yes/no)", amount)
            ).await?;
            dialogue.update(PayState::ReceiveConfirmation { amount }).await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Please send a valid number.").await?;
        }
    }
    Ok(())
}

async fn pay_receive_confirmation(
    bot: Bot,
    dialogue: PayDialogue,
    amount: u32,
    msg: Message,
) -> HandlerResult {
    match msg.text() {
        Some(text) if text.to_lowercase() == "yes" => {
            bot.send_message(
                msg.chat.id, 
                format!("Payment of {} processed successfully!", amount)
            ).await?;
            dialogue.exit().await?;
        }
        Some(text) if text.to_lowercase() == "no" => {
            bot.send_message(msg.chat.id, "Payment canceled.").await?;
            dialogue.exit().await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Please answer with 'yes' or 'no'.").await?;
        }
    }
    Ok(())
}

// Price dialogue handlers
async fn price_start(bot: Bot, dialogue: PriceDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Setting a new price. Enter the price:").await?;
    dialogue.update(PriceState::ReceivePrice).await?;
    Ok(())
}

async fn price_receive_price(bot: Bot, dialogue: PriceDialogue, msg: Message) -> HandlerResult {
    match msg.text().map(|text| text.parse::<u32>()) {
        Some(Ok(price)) => {
            bot.send_message(
                msg.chat.id, 
                format!("Price set to {}. Enter a description for this price:", price)
            ).await?;
            dialogue.update(PriceState::ReceiveDescription { price }).await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Please send a valid number.").await?;
        }
    }
    Ok(())
}

async fn price_receive_description(
    bot: Bot,
    dialogue: PriceDialogue,
    price: u32,
    msg: Message,
) -> HandlerResult {
    match msg.text() {
        Some(description) => {
            bot.send_message(
                msg.chat.id, 
                format!("Price set successfully!\nPrice: {}\nDescription: {}", price, description)
            ).await?;
            dialogue.exit().await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Please send a text description.").await?;
        }
    }
    Ok(())
}

```