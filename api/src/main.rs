use std::env;

use axum::{Router, routing::get};
use db;
use dotenv;
use events;
use redis::Client;
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait, Set};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let port = env::var("PORT").expect("PORT must be set");
    let connection_string = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let dragonfly_password =
        env::var("DRAGONFLY_PASSWORD").expect("DRAGONFLY_PASSWORD must be set");

    let redis_url = format!("redis://:{}@127.0.0.1:6379", dragonfly_password);
    let redis_client = Client::open(redis_url)?;
    let redis_connection = redis_client.get_multiplexed_async_connection().await?;
    let db = Database::connect(connection_string).await?;
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    axum::serve(listener, app).await?;
    Ok(())
}
