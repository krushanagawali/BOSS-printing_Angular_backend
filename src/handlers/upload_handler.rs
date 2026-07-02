use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    Json as AxumJson,
};
use chrono::{Duration, Utc};
use nanoid::nanoid;
use serde_json::json;
use sqlx::PgPool;
use tokio::fs;
use uuid::Uuid;

pub async fn upload_file(
    State(pool): State<PgPool>,
    mut multipart: Multipart,
) -> Result<AxumJson<serde_json::Value>, (StatusCode, AxumJson<serde_json::Value>)> {
    let mut file_paths = vec![];
    let mut copies = 1;
    let mut color = true;
    let mut paper_size = "A4".to_string();
    let mut orientation = "Portrait".to_string();
    let mut print_sides = "Single Side".to_string();
    let mut page_selection = "All Pages".to_string();
    let mut custom_page_range = "".to_string();
    let mut print_quality = "Normal".to_string();
    let mut scaling = "Fit to Page".to_string();
    let mut expiry_minutes = 5_i64;
    
    // Capture the JSON array of specific file settings
    let mut master_settings_str: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (StatusCode::BAD_REQUEST, AxumJson(json!({"success": false, "message": e.to_string()})))
    })? {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "files" => {
                // 1. EXTRACT THE EXTENSION FIRST (Before the field is consumed)
                let original_name = field.file_name().unwrap_or("file.pdf").to_string();
                let ext = std::path::Path::new(&original_name)
                    .extension()
                    .and_then(std::ffi::OsStr::to_str)
                    .unwrap_or("pdf")
                    .to_string();

                let filename = format!("{}.{}", Uuid::new_v4(), ext);
                let path = format!("uploads/{}", filename);

                // 2. NOW READ THE BYTES (This takes ownership and moves the field)
                let data = field.bytes().await.map_err(|e| {
                    (StatusCode::BAD_REQUEST, AxumJson(json!({"success": false, "message": e.to_string()})))
                })?;

                // 3. SAVE THE FILE
                fs::write(&path, &data).await.map_err(|e| {
                    (StatusCode::INTERNAL_SERVER_ERROR, AxumJson(json!({"success": false, "message": e.to_string()})))
                })?;

                file_paths.push(path);
            }
            "master_settings" => master_settings_str = Some(field.text().await.unwrap_or_default()),
            "copies" => copies = field.text().await.unwrap_or("1".into()).parse().unwrap_or(1),
            "color" => color = field.text().await.unwrap_or("true".into()) == "true",
            "paper_size" => paper_size = field.text().await.unwrap_or_default(),
            "orientation" => orientation = field.text().await.unwrap_or_default(),
            "print_sides" => print_sides = field.text().await.unwrap_or_default(),
            "page_selection" => page_selection = field.text().await.unwrap_or_default(),
            "custom_page_range" => custom_page_range = field.text().await.unwrap_or_default(),
            "print_quality" => print_quality = field.text().await.unwrap_or_default(),
            "scaling" => scaling = field.text().await.unwrap_or_default(),
            "expiry_minutes" => expiry_minutes = field.text().await.unwrap_or("5".into()).parse().unwrap_or(5),
            _ => {}
        }
    }

    if file_paths.is_empty() {
        return Err((StatusCode::BAD_REQUEST, AxumJson(json!({"success": false, "message": "No files uploaded"}))));
    }

    // SMART MAPPING: Replace dummy filenames with actual saved paths
    if let Some(ref mut ms) = master_settings_str {
        if let Ok(mut ms_array) = serde_json::from_str::<Vec<serde_json::Value>>(ms) {
            for (i, item) in ms_array.iter_mut().enumerate() {
                if let Some(path) = file_paths.get(i) {
                    item["filename"] = json!(path);
                }
            }
            *ms = serde_json::to_string(&ms_array).unwrap_or_default();
        }
    }

    let access_code = nanoid!(6, &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']);
    let expires_at = Utc::now().naive_utc() + Duration::minutes(expiry_minutes);

    for path in &file_paths {
        sqlx::query(
            r#"
            INSERT INTO print_jobs (
                access_code, file_path, copies, color, paper_size, 
                orientation, print_sides, page_selection, custom_page_range, 
                print_quality, scaling, status, expires_at, master_settings
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,'pending',$12,$13)
            "#
        )
        .bind(&access_code)
        .bind(path)
        .bind(copies)
        .bind(color)
        .bind(&paper_size)
        .bind(&orientation)
        .bind(&print_sides)
        .bind(&page_selection)
        .bind(&custom_page_range)
        .bind(&print_quality)
        .bind(&scaling)
        .bind(expires_at)
        .bind(&master_settings_str) // <--- SAVED TO DB!
        .execute(&pool)
        .await
        .map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, AxumJson(json!({"success": false, "message": e.to_string()})))
        })?;
    }

    Ok(AxumJson(json!({
        "success": true,
        "access_code": access_code,
        "files": file_paths.len()
    })))
}