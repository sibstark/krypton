use std::{collections::HashMap, env, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use db::{Transaction, TransactionModel};
use dotenv;
use events;
use redis::{Client, aio::MultiplexedConnection};
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait, IntoActiveModel, Set};
use serde_json::{Value, json};
use tokio;

#[derive(serde::Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(serde::Serialize)]
struct DataResponse {
    status: String,
    data: serde_json::Value,
}

impl Default for DataResponse {
    fn default() -> Self {
        Self {
            status: "ok".to_string(),
            data: serde_json::Value::Null,
        }
    }
}
#[derive(Clone)]
struct AppState {
    redis: Arc<tokio::sync::Mutex<MultiplexedConnection>>,
    db: DatabaseConnection,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let port = env::var("PORT").expect("PORT must be set");
    let connection_string = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let dragonfly_password =
        env::var("DRAGONFLY_PASSWORD").expect("DRAGONFLY_PASSWORD must be set");

    let redis_url = format!("redis://:{}@dragonfly:6379", dragonfly_password);
    let redis_client = Client::open(redis_url)?;
    let redis_connection = redis_client.get_multiplexed_async_connection().await?;
    let db = Database::connect(connection_string).await?;

    let state = AppState {
        redis: Arc::new(tokio::sync::Mutex::new(redis_connection)),
        db,
    };

    let app = Router::new()
        .route("/api/payment/{id}", get(get_transaction))
        .route("/api/payment/{id}/start", post(start_payment))
        .with_state(state.clone());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    axum::serve(listener, app).await?;
    Ok(())
}

async fn get_transaction(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let transaction = Transaction::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "DB error".to_string(),
                }),
            )
        })?;

    // Если есть — unwrap или else
    let Some(tx) = transaction else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Transaction not found".to_string(),
            }),
        ));
    };

    /*
    let price = tx.price;
    let telegram_id = tx.telegram_id;
    let channel_id = tx.channel_id;
    let gate_crypto_address = env::var("GATE_CRYPTO_ADDRESS").unwrap_or_default();
    let qr_code: image::ImageBuffer<image::Luma<u8>, Vec<u8>> = generate_qr_code(
        gate_crypto_address.to_string(),
        price,
        telegram_id,
        channel_id,
    );
    // Преобразуем QR в PNG
    let mut png_bytes: Vec<u8> = Vec::new();
    let dyn_image = DynamicImage::ImageLuma8(qr_code);
    let _ = dyn_image
        .write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)
        .unwrap();
    let img = InputFile::memory(png_bytes).file_name("payment_qr.png");
    */
    Ok(Json(DataResponse {
        data: json!({
            "id": tx.id,
            "price": tx.price,
            "created_at": tx.created_at,
            "status": tx.status,
            "currency": tx.currency
        }),
        ..Default::default()
    }))
}

async fn start_payment(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let transaction = Transaction::find_by_id(id)
        .one(&state.db)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "DB error".to_string(),
                }),
            )
        })?;

    // Если есть — unwrap или else
    let Some(tx) = transaction else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Transaction not found".to_string(),
            }),
        ));
    };

    /*
    let price = tx.price;
    let telegram_id = tx.telegram_id;
    let channel_id = tx.channel_id;
    let gate_crypto_address = env::var("GATE_CRYPTO_ADDRESS").unwrap_or_default();
    let qr_code: image::ImageBuffer<image::Luma<u8>, Vec<u8>> = generate_qr_code(
        gate_crypto_address.to_string(),
        price,
        telegram_id,
        channel_id,
    );
    // Преобразуем QR в PNG
    let mut png_bytes: Vec<u8> = Vec::new();
    let dyn_image = DynamicImage::ImageLuma8(qr_code);
    let _ = dyn_image
        .write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)
        .unwrap();
    let img = InputFile::memory(png_bytes).file_name("payment_qr.png");
    */
    Ok(Json(DataResponse {
        data: json!({
            "id": tx.id,
            "price": tx.price,
            "created_at": tx.created_at,
            "status": tx.status,
            "currency": tx.currency
        }),
        ..Default::default()
    }))
}
