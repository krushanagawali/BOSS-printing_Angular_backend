mod routes;
mod handlers;
mod models;

use axum::{
    Router,
    routing::{get_service, post}, 
    http::{StatusCode, Method, header::CONTENT_TYPE},
    extract::{State, Json},       
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

// --- NEW: Reset Password Request ---
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
            // Capture the EXACT database error
            let db_error = format!("DB Error: {}", e);
            
            // Print it to the server console just in case
            println!("🚨 REGISTER FAILED: {}", db_error);
            
            // Send the exact error to the Angular frontend
            (StatusCode::BAD_REQUEST, Json(AuthResponse { success: false, message: db_error }))
        }
    }}

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

// --- NEW: Reset Password Handler ---
pub async fn reset_password(
    State(pool): State<PgPool>,
    Json(payload): Json<ResetPasswordRequest>,
) -> (StatusCode, Json<AuthResponse>) {
    
    // 1. Hash the new password securely
    let hashed_password = hash(&payload.password, DEFAULT_COST).unwrap();

    // 2. Determine which table we are updating
    let table_name = if payload.role == "shopkeeper" { "shopkeepers" } else { "users" };

    // 3. Update the database where the mobile number matches
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

    // Ensure upload directory exists on startup
    fs::create_dir_all("uploads").await.unwrap();

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS]) 
        .allow_headers([CONTENT_TYPE]);

    let app = Router::new()
        // Your existing modular routes
        .merge(routes::create_routes())
        
        // --- AUTH ROUTES ---
        .route("/api/register", post(register_user))
        .route("/api/login", post(login_user))
        .route("/api/profile", post(get_profile))
        .route("/api/reset-password", post(reset_password)) // <-- NEW ROUTE MOUNTED HERE
        // -------------------
        
        .nest_service("/uploads", get_service(ServeDir::new("uploads")).handle_error(|_err| async move {
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to serve file")
        }))
        .layer(cors)
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("Server running on http://0.0.0.0:3000 (accessible via https://bharatonlinesafetyservices.com)");

    axum::serve(listener, app).await.unwrap();
}