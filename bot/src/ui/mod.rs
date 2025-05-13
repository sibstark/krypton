pub mod price;
pub mod pay;
pub mod info;

use std::{convert::Infallible, fmt::{Display, Formatter}, sync::Arc};
use thiserror::Error;
use teloxide::{dispatching::dialogue::{ErasedStorage, InMemStorageError, RedisStorageError}, prelude::Dialogue, utils::command::BotCommands, RequestError};

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
    Pay(String)
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
pub enum State {
    #[default]
    Start,
    Price(price::State),
    Pay(pay::State),
    Info(info::State),
}

pub type UserDialogue = Dialogue<State, ErasedStorage<State>>;

#[derive(Debug, Error)]
pub enum BotError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("Telegram API error: {0}")]
    Teloxide(#[from] RequestError),

    #[error("Dialog API error: {0}")]
    InMemStorage(#[from] InMemStorageError),

    #[error("RedisStorage error: {0}")]
    RedisStorage(#[from] RedisStorageError<Infallible>),

    #[error("RedisStorage error: {0}")]
    ErasedStorage(#[from] Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Clone)]
pub struct GateCryptoAddress(pub Arc<String>);

#[derive(Clone)]
pub struct PaymentGateway(pub Arc<String>);

impl Display for GateCryptoAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Display for PaymentGateway {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}