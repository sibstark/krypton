use teloxide::{ dispatching::DpHandlerDescription, prelude::*, types::ChatMemberKind, RequestError };
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    dotenv().ok();
    log::info!("Starting throw dice bot...");
    let token = env::var("BOT_TOKEN").expect("BOT_TOKEN must be set");
    let bot = Bot::new(token);

    Dispatcher::builder(bot, handler())
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}

fn handler() -> Handler<'static, DependencyMap, Result<(), RequestError>, DpHandlerDescription> {
    dptree::entry().branch(Update::filter_my_chat_member().branch(dptree::entry().endpoint(handle_chat_member_update)))
}


async fn handle_chat_member_update(bot: Bot, update: ChatMemberUpdated) -> Result<(), teloxide::RequestError> {
    if let (ChatMemberKind::Left, ChatMemberKind::Administrator { .. }) = 
        (update.old_chat_member.kind, update.new_chat_member.kind) {
        
        log::info!("Бот стал администратором в чате {:?}", update.chat.id);
        
        bot.send_message(
            update.chat.id, 
            "Спасибо за назначение меня администратором!"
        ).await?;
    }
    
    Ok(())
}