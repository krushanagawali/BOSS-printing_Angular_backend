mod routes;
mod handlers;
mod models;

use axum::{
    Router,
    routing::{get_service, post}, 
    http::{StatusCode, Method, header::CONTENT_TYPE},
    extract::{State, Json,DefaultBodyLimit},       
};
use tower_http::services::ServeDir;
use dotenvy::dotenv;
use sqlx::{PgPool, Row};          
use std::env;
use tower_http::cors::{CorsLayer, Any};
use tokio::fs;
use serde::{Deserialize, Serialize};
use bcrypt::{hash, verify, DEFAULT_COST};

// ==========================================
// AUTHENTICATION MODELS
// ==========================================
#[derive(Deserialize)]
pub struct RegisterRequest {
    pub role: String,
    pub user_id: String,
    pub password: String,
    pub mobile_number: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub role: String,
    pub user_id: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct ProfileRequest {
    pub role: String,
    pub user_id: String,
}

#[derive(Serialize)]
pub struct ProfileResponse {
    pub success: bool,
    pub mobile_number: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Deserialize)]
pub struct ResetPasswordRequest {
    pub role: String,
    pub mobile_number: String,
    pub password: String,
}

// ==========================================
// AUTHENTICATION HANDLERS
// ==========================================
pub async fn register_user(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterRequest>,
) -> (StatusCode, Json<AuthResponse>) {
    let hashed_password = hash(&payload.password, DEFAULT_COST).unwrap();

    let table_name = if payload.role == "shopkeeper" { "shopkeepers" } else { "users" };
    let id_column = if payload.role == "shopkeeper" { "shopkeeper_id" } else { "user_id" };

    let query = format!(
        "INSERT INTO {} ({}, password_hash, mobile_number) VALUES ($1, $2, $3)",
        table_name, id_column
    );

    let result = sqlx::query(&query)
        .bind(&payload.user_id)
        .bind(&hashed_password)
        .bind(&payload.mobile_number)
        .execute(&pool)
        .await;

    match result {
        Ok(_) => (
            StatusCode::OK,
            Json(AuthResponse { success: true, message: "Account created!".to_string() })
        ),
        Err(e) => {
            let msg = if e.to_string().contains("unique constraint") {
                "ID or Mobile Number already exists.".to_string()
            } else {
                "Failed to create account.".to_string()
            };
            (StatusCode::BAD_REQUEST, Json(AuthResponse { success: false, message: msg }))
        }
    }
}

pub async fn login_user(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginRequest>,
) -> (StatusCode, Json<AuthResponse>) {
    let table_name = if payload.role == "shopkeeper" { "shopkeepers" } else { "users" };
    let id_column = if payload.role == "shopkeeper" { "shopkeeper_id" } else { "user_id" };

    let query = format!("SELECT password_hash FROM {} WHERE {} = $1", table_name, id_column);
    
    let result = sqlx::query(&query)
        .bind(&payload.user_id)
        .fetch_optional(&pool)
        .await;

    match result {
        Ok(Some(row)) => {
            let stored_hash: String = row.get("password_hash");
            if verify(&payload.password, &stored_hash).unwrap_or(false) {
                (StatusCode::OK, Json(AuthResponse { success: true, message: "Login successful".to_string() }))
            } else {
                (StatusCode::UNAUTHORIZED, Json(AuthResponse { success: false, message: "Incorrect password".to_string() }))
            }
        },
        Ok(None) => (StatusCode::NOT_FOUND, Json(AuthResponse { success: false, message: "User not found".to_string() })),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(AuthResponse { success: false, message: "Database error".to_string() }))
    }
}

pub async fn get_profile(
    State(pool): State<PgPool>,
    Json(payload): Json<ProfileRequest>,
) -> (StatusCode, Json<ProfileResponse>) {
    let table_name = if payload.role == "shopkeeper" { "shopkeepers" } else { "users" };
    let id_column = if payload.role == "shopkeeper" { "shopkeeper_id" } else { "user_id" };

    let query = format!("SELECT mobile_number FROM {} WHERE {} = $1", table_name, id_column);
    
    let result = sqlx::query(&query)
        .bind(&payload.user_id)
        .fetch_optional(&pool)
        .await;

    match result {
        Ok(Some(row)) => {
            let mobile: String = row.get("mobile_number");
            (
                StatusCode::OK, 
                Json(ProfileResponse { success: true, mobile_number: mobile, message: "Success".to_string() })
            )
        },
        _ => (
            StatusCode::NOT_FOUND, 
            Json(ProfileResponse { success: false, mobile_number: "".to_string(), message: "User not found".to_string() })
        )
    }
}

// THIS IS THE MISSING FUNCTION THAT WAS CAUSING THE ERROR
pub async fn reset_password(
    State(pool): State<PgPool>,
    Json(payload): Json<ResetPasswordRequest>,
) -> (StatusCode, Json<AuthResponse>) {
    
    let hashed_password = hash(&payload.password, DEFAULT_COST).unwrap();
    let table_name = if payload.role == "shopkeeper" { "shopkeepers" } else { "users" };

    let query = format!(
        "UPDATE {} SET password_hash = $1 WHERE mobile_number = $2",
        table_name
    );

    let result = sqlx::query(&query)
        .bind(&hashed_password)
        .bind(&payload.mobile_number)
        .execute(&pool)
        .await;

    match result {
        Ok(res) if res.rows_affected() > 0 => (
            StatusCode::OK,
            Json(AuthResponse { success: true, message: "Password updated successfully!".to_string() })
        ),
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(AuthResponse { success: false, message: "Mobile number not found.".to_string() })
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuthResponse { success: false, message: "Database error.".to_string() })
        )
    }
}


// --- ULTIMATE GHOST BYPASS HANDLER ---
pub async fn force_download(axum::extract::Path(file_id): axum::extract::Path<String>) -> axum::response::Response {
    // We add the .pdf extension BACK ON secretly inside the server!
    let absolute_path = format!("/var/www/boss-backend/backend/uploads/{}.pdf", file_id);
    
    match tokio::fs::read(&absolute_path).await {
        Ok(bytes) => {
            axum::response::Response::builder()
                .status(axum::http::StatusCode::OK)
                .header(axum::http::header::CONTENT_TYPE, "application/pdf")
                .body(axum::body::Body::from(bytes))
                .unwrap()
        }
        Err(e) => {
            axum::response::Response::builder()
                .status(axum::http::StatusCode::NOT_FOUND)
                .body(axum::body::Body::from(format!("LINUX ERROR: {}", e)))
                .unwrap()
        }
    }
}

// ==========================================
// MAIN APP ROUTER
// ==========================================
#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL missing");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("DB connection failed");

    println!("Database connected successfully");

    fs::create_dir_all("uploads").await.unwrap();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS]) 
        .allow_headers([CONTENT_TYPE],Any);

    let app = Router::new()
        .merge(routes::create_routes())
        .route("/api/register", post(register_user))
        .route("/api/login", post(login_user))
        .route("/api/profile", post(get_profile))
        .route("/api/reset-password", post(reset_password)) 
        
.route("/api/force-download/:file_id", axum::routing::get(force_download))
        // --- BULLETPROOF FIX: We tell it EXACTLY where the files live on the Linux hard drive ---
        .nest_service("/uploads", get_service(ServeDir::new("/var/www/boss-backend/backend/uploads")).handle_error(|_err| async move {
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to serve file")
        }))
        .nest_service("/api/uploads", get_service(ServeDir::new("/var/www/boss-backend/backend/uploads")).handle_error(|_err| async move {
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to serve file")
        }))
        // ----------------------------------------------------------------------------------------

        .layer(DefaultBodyLimit::max(50 * 1024 * 1024))
        .layer(cors)
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("Server running on http://bharatonlinesafetyservices.com (accessible via https://bharatonlinesafetyservices.com)");

    axum::serve(listener, app).await.unwrap();
}
