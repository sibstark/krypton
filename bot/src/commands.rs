use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(rename_rule="lowercase", description = "These commands are supported:")]
pub enum Commands {
    #[command(description = "Start your dealing with the Krypton bot")]
    Start,
    #[command(description = "Get the list of available commands")]
    Help,
    #[command(description="Set price for owned Telegram channel")]
    SetPrice,
    #[command(description="Show info about owned Telegram channel")]
    Info,
    #[command(description="Pay for channel subscription")]
    Pay
}