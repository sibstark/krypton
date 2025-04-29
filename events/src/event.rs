use redis::AsyncCommands;
use redis::aio::MultiplexedConnection;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentEvent {
    pub transaction_id: i64,
    pub telegram_id: i64,
    pub channel_id: i64,
    pub chat_id: i64,
    pub price: Decimal,
}

pub async fn send_payment_event(
    event: &PaymentEvent,
    con: &MultiplexedConnection,
) -> redis::RedisResult<()> {
    let payload = serde_json::to_string(event).unwrap();
    let _: i64 = con.lpush("pending_payments", payload).await?;
    Ok(())
}

pub async fn pop_payment_event(
    con: &mut MultiplexedConnection,
) -> redis::RedisResult<Option<PaymentEvent>> {
    let result: Option<(String, String)> = con.brpop("pending_payments", 0.0).await?;
    if let Some((_queue, json)) = result {
        match serde_json::from_str(&json) {
            Ok(event) => Ok(Some(event)),
            Err(err) => {
                eprintln!("Не удалось распарсить PaymentEvent: {err:?}, исходная строка: {json}");
                Ok(None)
            }
        }
    } else {
        Ok(None)
    }
}

// Проверка на идемпотентность и установка флага "обработано"
pub async fn process_payment_event(
    con: &mut MultiplexedConnection,
    event: PaymentEvent,
) -> redis::RedisResult<()> {
    let set_key = "processed_transactions";
    let is_new: bool = con.sadd(set_key, event.transaction_id).await?;

    if !is_new {
        println!(
            "Событие с transaction_id={} уже обработано, пропускаем",
            event.transaction_id
        );
        return Ok(()); // Уже обработано, не повторяем
    }

    // Здесь основная логика обработки платежа!
    println!("Обрабатываем событие: {:?}", event);

    // После успешной обработки можешь оставить transaction_id в сете (навсегда или с TTL)
    // Если хочешь, можешь поставить TTL:
    // let _: () = con.expire(set_key, 60*60*24*7).await?; // 7 дней

    Ok(())
}
