
use sqlx::{
    Pool, Postgres,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use std::env;
use std::str::FromStr;
use uuid::Uuid;

pub async fn get_test_pool() -> (Pool<Postgres>, String) {
    // Load .env variables
    // We assume .env is in the root of the project
    dotenvy::dotenv().ok();

    // Get the database URL from environment
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Generate a unique name for the test database
    let db_name = format!("test_{}", Uuid::new_v4().simple());

    // Connect to the default maintenance database (usually 'postgres') using the credentials from DATABASE_URL
    let options = PgConnectOptions::from_str(&database_url).expect("Invalid DATABASE_URL");

    // Switch to 'postgres' database to create the new test DB
    // We can't create a database while connected to the same database we want to create/drop sometimes,
    // but more importantly we need a connection to execute CREATE DATABASE.
    // Usually connecting to 'postgres' is safe.
    let maintenance_options = options.clone().database("postgres");

    let pool = PgPoolOptions::new()
        .connect_with(maintenance_options)
        .await
        .expect("Failed to connect to maintenance database");

    // Create the new test database
    sqlx::query(&format!("CREATE DATABASE \"{}\"", db_name))
        .execute(&pool)
        .await
        .expect("Failed to create test database");

    pool.close().await;

    // Connect to the newly created test database
    let test_db_options = options.clone().database(&db_name);
    let new_pool = PgPoolOptions::new()
        .connect_with(test_db_options)
        .await
        .expect("Failed to connect to test database");

    // Run migrations on the new database
    // Note: The path is relative to the directory containing Cargo.toml
    sqlx::migrate!("./migrations")
        .run(&new_pool)
        .await
        .expect("Failed to run migrations");

    (new_pool, db_name)
}
