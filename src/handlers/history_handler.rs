use axum::{
    extract::State,
    Json,
};
use serde::Serialize;
use sqlx::{PgPool, Row};
use chrono::NaiveDateTime;

#[derive(Serialize)]
pub struct PrintHistory {
    pub access_code: String,
    pub copies: i32,
    pub color: bool,
    pub status: String,
    pub created_at: String,
    pub expires_at: String,
}

pub async fn get_history(
    State(pool): State<PgPool>,
) -> Json<Vec<PrintHistory>> {

    // Safely mapping data dynamically instead of relying on exact compile-time macros
    let rows = sqlx::query(
        r#"
        SELECT
            access_code,
            copies,
            color,
            status,
            created_at,
            expires_at
        FROM print_jobs
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let history = rows
        .into_iter()
        .map(|row| {
            let created_at: NaiveDateTime = row.get("created_at");
            let expires_at: NaiveDateTime = row.get("expires_at");

            PrintHistory {
                access_code: row.get("access_code"),
                copies: row.get("copies"),
                color: row.get("color"),
                status: row.get("status"),
                created_at: created_at.to_string(),
                expires_at: expires_at.to_string(),
            }
        })
        .collect::<Vec<_>>();

    Json(history)
}