use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use teloxide::{
    dispatching::dialogue::{GetChatId, InMemStorage},
    prelude::*,
    utils::command::BotCommands,
};
mod krypton_bot;
use krypton_bot::KryptonBot;
use std::env::var;

#[derive(Clone, Default, Serialize, Deserialize)]
enum DialogState {
    #[default]
    Idle,
    InStartFlow(StartState), // Добавлять новые состояния команд здесь
}
#[derive(Clone, Default, Serialize, Deserialize)]
enum StartState {
    #[default]
    Greeting,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting throw dice bot...");
    dotenv().ok(); // Загрузит переменные из .env файла
    let token = var("TELOXIDE_TOKEN").unwrap();
    let db_connection = var("DATABASE_URL").unwrap();
    print!("token {}", token);

    let bot = KryptonBot::new(token, db_connection).await.unwrap();

    Command::repl(bot.bot, answer).await;
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "Show list of commands")]
    Help,
    #[command(description = "Start conversation with bot")]
    Start,
    #[command(description = "Find group to subscribe on")]
    Find,
    #[command(description = "Pay subscription for chanel")]
    Pay,
    #[command(description = "My subscriptions")]
    Subscriptions,
    // delete then
    #[command(description = "handle a username.")]
    Username(String),
    // delete then
    #[command(description = "handle a username and an age.", parse_with = "split")]
    UsernameAndAge { username: String, age: u8 },
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Start => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }
        Command::Username(username) => {
            bot.send_message(msg.chat.id, format!("Your username is @{username}."))
                .await?
        }
        Command::UsernameAndAge { username, age } => {
            bot.send_message(
                msg.chat.id,
                format!("Your username is @{username} and age is {age}."),
            )
            .await?
        }
        _ => {
            bot.send_message(msg.chat.id, format!("Not implemented"))
                .await?
        }
    };

    Ok(())
}

async fn command_handler(
    bot: Bot,
    msg: Message,
    cmd: Command,
    dialogue: Dialogue<InMemStorage<DialogState>>,
) -> ResponseResult<()> {
    match cmd {
        Command::Start => handle_start(bot, msg, dialogue).await,
        // Command::Find => handle_find(bot, msg, dialogue).await,
        // Command::Pay => handle_pay(bot, msg, dialogue).await,
        // Обработчики для других команд
        _ => handle_unknown_command(bot, msg).await,
    }
}

async fn handle_unknown_command(bot: Bot, msg: Message) -> ResponseResult<()> {
    bot.send_message(msg.chat.id, format!("Unknow command, try /start"))
    .await?;

    Ok(())
}
