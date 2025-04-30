use std::env;
use dotenv::dotenv;
use redis::Client;
use sea_orm::{ Database, DatabaseConnection };
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use events::event::{pop_payment_event, PaymentEvent};
use serde::Deserialize;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let dragonfly_password = env::var("DRAGONFLY_PASSWORD").expect("DRAGONFLY_PASSWORD must be set");
    let connection_string = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db: DatabaseConnection = Database::connect(connection_string).await?;
    let client = Client::open(format!("redis://:{}@127.0.0.1:6379", dragonfly_password))?;
    let mut manager = client.get_multiplexed_tokio_connection().await?;
    loop {
        if let Some(event) = pop_payment_event(&mut manager).await? {
            println!("\n💰 Получено событие: {:?}", event);

            let status = check_transaction_status(&event).await;
            println!("🔍 Статус транзакции: {}", status);

            // update_transaction_status(&db, event.transaction_id, &status).await?;
            // notify_user(event.telegram_id, &status).await;
        }
    }
}

#[derive(Debug, Deserialize)]
struct TonTransaction {
    in_msg: InMsg,
}

#[derive(Debug, Deserialize)]
struct InMsg {
    value: String,
    source: String,
    payload: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TonApiResponse {
    result: Vec<TonTransaction>,
}

async fn check_transaction_status(event: &PaymentEvent) -> String {
    let address = event.wallet_address.clone();
    let url = format!(
        "https://toncenter.com/api/v2/getTransactions?address={}&limit=15&api_key={}",
        address,
        env::var("TON_API_KEY").unwrap_or_default()
    );

    let res = match reqwest::get(&url).await {
        Ok(resp) => resp,
        Err(_) => return "error".to_string(),
    };

    let data: TonApiResponse = match res.json().await {
        Ok(json) => json,
        Err(_) => return "error".to_string(),
    };

    for tx in data.result {
        if let Some(payload_base64) = tx.in_msg.payload {
            if let Some(decoded_tx_id) = decode_transaction_id_from_payload(&payload_base64) {
                if decoded_tx_id == event.transaction_id {
                    return "success".to_string();
                }
            }
        }
    }

    "pending".to_string()
}

fn decode_transaction_id_from_payload(payload_base64: &str) -> Option<i64> {
    let decoded = STANDARD.decode(payload_base64).ok()?;
    let text = String::from_utf8(decoded).ok()?;
    if let Some(tx_str) = text.strip_prefix("transaction_id=") {
        tx_str.parse::<i64>().ok()
    } else {
        None
    }
}