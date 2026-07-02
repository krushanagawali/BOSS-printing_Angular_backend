use sqlx::{PgPool, postgres::PgPoolOptions};

pub async fn connect_db() -> PgPool {

    let database_url =
        std::env::var("DATABASE_URL")
            .expect("DATABASE_URL not found");

    println!("DATABASE_URL => {}", database_url);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect DB");

    println!("Database connected successfully ✅");

    pool
}