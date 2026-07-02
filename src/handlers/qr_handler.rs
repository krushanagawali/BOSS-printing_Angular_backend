use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use serde_json::json;

#[derive(Deserialize)]
pub struct QrHandshake {
    pub access_code: String,
    pub timestamp: i64,
}

pub async fn verify_qr(
    State(pool): State<PgPool>,
    Json(payload): Json<QrHandshake>,
) -> Json<serde_json::Value> {
    let now = chrono::Utc::now().timestamp();
    
    // 1. Verify if timestamp is within 30 seconds (Anti-replay)
    if (now - payload.timestamp).abs() > 30 {
        return Json(json!({"success": false, "message": "QR Expired: Please refresh."}));
    }

    // 2. Validate existence in DB
    let exists = sqlx::query("SELECT 1 FROM print_jobs WHERE access_code = $1")
        .bind(&payload.access_code)
        .fetch_optional(&pool)
        .await
        .unwrap_or(None);

    if exists.is_some() {
        Json(json!({"success": true, "message": "Handshake verified. Printing..."}))
    } else {
        Json(json!({"success": false, "message": "Invalid QR"}))
    }
}