
use sqlx::PgPool;
use teloxide::prelude::*;
use std::error::Error;

pub struct KryptonBot {
    pub bot: Bot,
    // db: PgPool
}

impl KryptonBot {
    pub async fn new(token: String, connectin: String) -> Result<Self, Box<dyn Error>> {
        let bot = Bot::new(token);
        // let db = PgPool::connect(&connectin).await?;
        Ok(Self {
            bot
        })
    }
}