use base64::engine::general_purpose::STANDARD;
use base64::Engine; // импортируем трейт
use image::{ImageBuffer, Luma};
use qrcode::QrCode;
use urlencoding;

pub fn generate_qr_code(
    contract_address: String,
    amount: f64,
    transaction_id: i64
) -> ImageBuffer<Luma<u8>, Vec<u8>> {
    let raw_payload = format!("transaction_id={}", transaction_id);

    // Кодируем payload в base64
    let payload_base64 = STANDARD.encode(raw_payload.as_bytes());

    // Комментарий (опционально)
    let text = format!("Fee Split Transfer for Krypton transaction {}", transaction_id);

    // Финальный deeplink
    let ton_link = format!(
        "ton://transfer/{}?amount={}&payload={}&text={}",
        contract_address,
        amount,
        payload_base64,
        urlencoding::encode(&text)
    );

    // Генерация QR-кода
    let code = QrCode::new(ton_link).unwrap();
    code.render::<Luma<u8>>().build()
}
