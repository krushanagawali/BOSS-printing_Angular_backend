use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;

// Import your module handlers
use crate::handlers::{
    upload_handler::upload_file,
    code_handler::get_job,
    print_handler::print_job,
    download_handler::download_file,
    history_handler::get_history,
};

pub fn create_routes() -> Router<PgPool> {
    Router::new()
        .route("/api/upload", post(upload_file))
        .route("/api/job", post(get_job))
        .route("/print", post(print_job))
        .route("/download/:access_code", get(download_file))
        .route("/api/history", get(get_history))
}