use axum::{
    extract::{Path, Query, State}, // <-- Added Query here
    http::{header, StatusCode},
    response::IntoResponse,
};
use sqlx::{PgPool, Row};
use std::collections::HashMap; // <-- Added HashMap here
use tokio::fs;

pub async fn download_file(
    Path(access_code): Path<String>,
    Query(params): Query<HashMap<String, String>>, // <-- Now handles ?file=...
    State(pool): State<PgPool>,
) -> impl IntoResponse {

    // If the print agent asks for a specific file, find it. Otherwise, return the first one.
    let job = if let Some(file_path) = params.get("file") {
        sqlx::query("SELECT file_path, status FROM print_jobs WHERE access_code = $1 AND file_path = $2 LIMIT 1")
            .bind(&access_code)
            .bind(file_path)
            .fetch_optional(&pool).await
    } else {
        sqlx::query("SELECT file_path, status FROM print_jobs WHERE access_code = $1 LIMIT 1")
            .bind(&access_code)
            .fetch_optional(&pool).await
    };

    let job = match job {
        Ok(Some(j)) => j,
        _ => return (StatusCode::NOT_FOUND, "File not found").into_response(),
    };

    let status: String = job.get("status");
    if status == "printed" {
        return (StatusCode::GONE, "File has already been printed and removed from the device.").into_response();
    }

    let file_path: String = job.get("file_path");

    let file = match fs::read(&file_path).await {
        Ok(f) => f,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Cannot read file").into_response(),
    };

    (
        [
            (header::CONTENT_TYPE, "application/pdf"),
            (header::CONTENT_DISPOSITION, "inline; filename=print.pdf"),
        ],
        file
    ).into_response()
}