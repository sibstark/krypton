use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use teloxide::{
    dispatching::{
        dialogue::{Dialogue, InMemStorage},
        DpHandlerDescription,
    },
    prelude::*,
    types::{ChatKind, ChatPublic, PublicChatKind},
    utils::command::BotCommands,
    RequestError,
};
mod krypton_bot;
use krypton_bot::KryptonBot;
use std::env::var;
use std::sync::Arc;

#[derive(Clone, Default, Serialize, Deserialize)]
enum DialogState {
    #[default]
    Idle,
    InStartFlow(StartState),
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
    dotenv().ok();
    let token = var("TELOXIDE_TOKEN").unwrap();
    let db_connection = var("DATABASE_URL").unwrap();
    print!("token {}", token);

    let bot = KryptonBot::new(token, db_connection).await.unwrap();
    let storage = InMemStorage::<DialogState>::new();

    Dispatcher::builder(bot.bot, handler())
        .dependencies(dptree::deps![storage])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum PrivateCommand {
    #[command(description = "Show list of commands")]
    Help,
    #[command(description = "Start conversation with bot")]
    Start,
    #[command(description = "Find group to subscribe on")]
    Find,
    #[command(description = "Pay subscription for chanel")]
    Pay,
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum PublicCommand {
    #[command(description = "Show list of commands")]
    Help,
    #[command(description = "My subscriptions")]
    Subscriptions,
    #[command(description = "Activate bot")]
    Activate,
    #[command(description = "Deactivate bot")]
    Deactivate,
}

fn handler() -> Handler<'static, DependencyMap, Result<(), RequestError>, DpHandlerDescription> {
    dptree::entry()
        .branch(
            Update::filter_message().branch(
                dptree::filter(|msg: Message| {
                    msg.chat.is
                    let r = matches!(msg.chat.kind, ChatKind::Private(_));
                    r
                })
                    .branch(
                        dptree::entry()
                            .filter_command::<PrivateCommand>()
                            .endpoint(private_command_handler),
                    )
                    .branch(
                        dptree::entry().endpoint(handle_private_regular_message),
                    ),
            ),
        )
        .branch(
            dptree::filter(|msg: Message| {
                let r = matches!(msg.chat.kind, ChatKind::Public(_));
                r
            })
                .branch(
                    dptree::entry()
                        .filter_command::<PublicCommand>()
                        .endpoint(public_command_handler),
                )
                .branch(
                    dptree::entry().endpoint(handle_public_regular_message),
                ),
        )
}

async fn private_command_handler(
    bot: Bot,
    msg: Message,
    cmd: PrivateCommand,
    dialogue_storage: Arc<InMemStorage<DialogState>>,
) -> ResponseResult<()> {
    let dialogue = Dialogue::new(dialogue_storage, msg.chat.id);

    match cmd {
        PrivateCommand::Start => handle_private_start(bot, msg, dialogue).await,
        _ => handle_private_unknown_command(bot, msg).await,
    }
}

async fn public_command_handler(
    bot: Bot,
    msg: Message,
    cmd: PublicCommand,
    dialogue_storage: Arc<InMemStorage<DialogState>>,
) -> ResponseResult<()> {
    let dialogue = Dialogue::new(dialogue_storage, msg.chat.id);

    match cmd {
        PublicCommand::Help => handle_public_help(bot, msg, dialogue).await,
        _ => handle_public_unknown_command(bot, msg).await,
    }
}

async fn handle_public_help(
    bot: Bot,
    msg: Message,
    dialogue: Dialogue<DialogState, InMemStorage<DialogState>>,
) -> ResponseResult<()> {
    bot.send_message(msg.chat.id, PublicCommand::descriptions().to_string())
        .await?;

    Ok(())
}

async fn handle_private_start(
    bot: Bot,
    msg: Message,
    dialogue: Dialogue<DialogState, InMemStorage<DialogState>>,
) -> ResponseResult<()> {
    bot.send_message(msg.chat.id, PrivateCommand::descriptions().to_string())
        .await?;

    Ok(())
}

async fn handle_private_unknown_command(bot: Bot, msg: Message) -> ResponseResult<()> {
    bot.send_message(msg.chat.id, "Unknown command, try /start")
        .await?;

    Ok(())
}

async fn handle_public_unknown_command(bot: Bot, msg: Message) -> ResponseResult<()> {
    bot.send_message(msg.chat.id, "Unknown command, try /help")
        .await?;

    Ok(())
}

async fn handle_private_regular_message(bot: Bot, msg: Message) -> ResponseResult<()> {
    bot.send_message(
        msg.chat.id,
        "This is not a command. Please use commands like /start or /help",
    )
    .await?;

    Ok(())
}

async fn handle_public_regular_message(bot: Bot, msg: Message) -> ResponseResult<()> {
    bot.send_message(
        msg.chat.id,
        "This is not a command. Please use commands like or /help",
    )
    .await?;

    Ok(())
}
