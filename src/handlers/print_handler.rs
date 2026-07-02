use axum::{
    extract::State,
    Json as AxumJson,
};
use chrono::Utc;
use serde_json::json;
use sqlx::{PgPool, Row};
use tokio::fs;

use crate::models::request::CodeRequest;



pub async fn print_job(
    State(pool): State<PgPool>,
    AxumJson(payload): AxumJson<CodeRequest>,
) -> AxumJson<serde_json::Value> {
    let jobs = match sqlx::query("SELECT * FROM print_jobs WHERE access_code = $1")
        .bind(&payload.access_code)
        .fetch_all(&pool)
        .await
    {
        Ok(j) => j,
        Err(e) => {
            return AxumJson(json!({ "success": false, "message": e.to_string() }));
        }
    };

    if jobs.is_empty() {
        return AxumJson(json!({ "success": false, "message": "Invalid code" }));
    }

    let first = &jobs[0];
    let expires_at: chrono::NaiveDateTime = first.get("expires_at");

    if Utc::now().naive_utc() > expires_at {
        return AxumJson(json!({ "success": false, "message": "Code expired" }));
    }

    let status: String = first.get("status");

    if status == "printed" {
        return AxumJson(json!({ "success": false, "message": "Already printed" }));
    }

    
    // Collect file paths before DB update so we can delete them
    let files_to_delete: Vec<String> = jobs.iter().map(|row| row.get("file_path")).collect();

    sqlx::query("UPDATE print_jobs SET status = 'printed' WHERE access_code = $1")
        .bind(&payload.access_code)
        .execute(&pool)
        .await
        .unwrap();

    // Automatically remove the files from the device
    for file_path in files_to_delete {
        let _ = fs::remove_file(&file_path).await;
    }

    let download_url = format!("http://bharatonlinesafetyservices.com/download/{}", payload.access_code);

    AxumJson(json!({
        "success": true,
        "message": "Job ready for hardware agent",
        "download_url": download_url,
        "copies": first.get::<i32,_>("copies"),
        "color": first.get::<bool,_>("color"),
        "status": "pending"
    }))

    
}

