use std::{collections::HashMap, env, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    debug_handler
};
use axum_extra::headers::Authorization;
use axum_extra::{TypedHeader, headers::authorization::Credentials};
use chrono::{Utc, Duration};
use db::{Transaction, TransactionModel};
use dotenv;
use events;
use base64::{engine::{general_purpose}, Engine};
use rand::{self, Rng};
use redis::{Client, aio::MultiplexedConnection};
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait };
use serde_json::{Value, json};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use tokio;

#[derive(Debug, Clone)]
struct Wallet(pub String);

impl Credentials for Wallet {
    const SCHEME: &'static str = "Wallet";

    fn decode(value: &axum::http::HeaderValue) -> Option<Self> {
        let str_val = value.to_str().ok()?;
        let prefix = format!("{} ", Self::SCHEME);
        if str_val.starts_with(&prefix) {
            Some(Wallet(str_val[prefix.len()..].to_string()))
        } else {
            None
        }
    }

    fn encode(&self) -> axum::http::HeaderValue {
        let encoded = format!("{} {}", Self::SCHEME, self.0);
        HeaderValue::from_str(&encoded).unwrap()
    }
}
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
    let port = env::var("API_PORT").expect("PORT must be set");
    let connection_string = env::var("API_DATABASE_URL").expect("DATABASE_URL must be set");
    let dragonfly_password =
        env::var("API_DRAGONFLY_PASSWORD").expect("API_DRAGONFLY_PASSWORD must be set");
    let redis_host = env::var("API_HOST_URL").expect("API_HOST_URL must be set");
    let redis_url = format!("redis://:{}@{}:6379", dragonfly_password, redis_host);
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
        .route("/api/auth/challenge", get(get_challenge))
        .route("/api/auth/tonproof", post(verify_proof))
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
    TypedHeader(Authorization(wallet)): TypedHeader<Authorization<Wallet>>,
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

#[derive(serde::Serialize)]
pub struct TonProofChallenge {
    domain: String,
    timestamp: i64,
    payload: String,
}

async fn get_challenge() -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let mut rng = rand::rng();
    let nonce: u64 = rng.random::<u64>();
    let payload = format!("nonce_{}", nonce);

    Ok(Json(DataResponse {
        data: json!(TonProofChallenge {
            domain: "krypton.com".to_string(),
            timestamp: Utc::now().timestamp(),
            payload,
        }),
        ..Default::default()
    }))
}

#[derive(serde::Deserialize)]
struct TonProofRequest {
    address: String,
    proof: ProofPayload,
}

#[derive(serde::Deserialize)]
struct ProofPayload {
    timestamp: i64,
    domain: String,
    payload: String,
    signature: String,
}

#[derive(serde::Serialize)]
pub struct AuthResponse {
    valid: bool,
    address: Option<String>,
}

async fn verify_proof(
    TypedHeader(Authorization(wallet)): TypedHeader<Authorization<Wallet>>,
    Json(req): Json<TonProofRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let now = Utc::now().timestamp();
    if (req.proof.timestamp - now).abs() > Duration::minutes(5).num_seconds() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: json!(AuthResponse { valid: false, address: None }),
            }),
        ));
    }

    // Проверка домена (опционально)
    if req.proof.domain != "yourdomain.com" {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: json!(AuthResponse { valid: false, address: None }),
            }),
        ));
    }

    // Верификация подписи
    let signature_bytes = match general_purpose::STANDARD.decode(&req.proof.signature) {
        Ok(bytes) => bytes,
        Err(_) => return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: json!(AuthResponse { valid: false, address: None }),
            }),
        ))
    };

    let signature_array: [u8; 64] = match signature_bytes.as_slice().try_into() {
        Ok(arr) => arr,
        Err(_) => return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: json!(AuthResponse { valid: false, address: None }),
            }),
        )),
    };

    let sig = ed25519_dalek::Signature::from_bytes(&signature_array);

    let pk_bytes = match general_purpose::STANDARD.decode(&req.address) {
        Ok(bytes) => bytes,
        Err(_) =>         return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: json!(AuthResponse { valid: false, address: None }),
            }),
        ))
    };

    let pk_array: [u8; 32] = match pk_bytes.as_slice().try_into() {
        Ok(arr) => arr,
        Err(_) => return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: json!(AuthResponse { valid: false, address: None }),
            }),
        ))
    };

    let public_key = match VerifyingKey::from_bytes(&pk_array) {
        Ok(pk) => pk,
        Err(_) => return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: json!(AuthResponse { valid: false, address: None }),
            }),
        ))
    };

    if public_key.verify(req.proof.payload.as_bytes(), &sig).is_ok() {
        Ok(Json(
            DataResponse {
                data: json!(AuthResponse { valid: true, address: Some(req.address) }),
                ..Default::default()
            }
        ))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: json!(AuthResponse { valid: false, address: None }),
            }),
        ))
    }
}