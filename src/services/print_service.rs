// use sqlx::PgPool;
// use uuid::Uuid;
// use chrono::NaiveDateTime;

// pub async fn save_print_job(
//     pool: &PgPool,
//     file_path: String,
//     access_code: String,
//     expires_at: NaiveDateTime,
// ) {

//     sqlx::query(
//         r#"
//         INSERT INTO print_jobs
//         (id, file_path, access_code, expires_at, copies, color, status)
//         VALUES ($1, $2, $3, $4, $5, $6, $7)
//         "#
//     )
//     .bind(Uuid::new_v4())
//     .bind(file_path)
//     .bind(access_code)
//     .bind(expires_at)
//     .bind(2)
//     .bind(true)
//     .bind("pending")
//     .execute(pool)
//     .await
//     .expect("Failed to save print job");

//     println!("DB insert successful!");
// }