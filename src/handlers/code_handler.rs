use axum::{
    extract::State,  
    Json as AxumJson,
};
use chrono::Utc;
use serde_json::json;
use sqlx::{PgPool, Row};

use crate::models::request::CodeRequest;



pub async fn get_job(
    State(pool): State<PgPool>,
    AxumJson(payload): AxumJson<CodeRequest>,
) -> AxumJson<serde_json::Value> {
    let jobs = match sqlx::query("SELECT * FROM print_jobs WHERE access_code = $1")
        .bind(&payload.access_code)
        .fetch_all(&pool)
        .await
    {
        Ok(j) => j,
        Err(e) => return AxumJson(json!({ "success": false, "message": e.to_string() })),
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

    let files: Vec<String> = jobs.iter().map(|j| j.get::<String, _>("file_path")).collect();
    
    // FETCH THE MASTER SETTINGS
    let master_settings: Option<String> = first.try_get("master_settings").unwrap_or(None);

    AxumJson(json!({
        "success": true,
        "files": files,
        "master_settings": master_settings, // <--- Sent to frontend!
        "copies": first.get::<i32,_>("copies"),
        "color": first.get::<bool,_>("color"),
        "paper_size": first.get::<String,_>("paper_size"),
        "orientation": first.get::<String,_>("orientation"),
        "print_sides": first.get::<String,_>("print_sides"),
        "page_selection": first.get::<String,_>("page_selection"),
        "custom_page_range": first.get::<String,_>("custom_page_range"),
        "print_quality": first.get::<String,_>("print_quality"),
        "scaling": first.get::<String,_>("scaling")
    }))
}