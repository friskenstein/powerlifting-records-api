use std::env;
use axum::{routing::get, Router};

mod db;
mod build;
mod api;

#[tokio::main]
async fn main() {
    let csv_dir = env::var("CSV_DIR").expect("CSV_DIR is not set in .env");
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a valid number");

    build::build_records(&csv_dir);

    let app = Router::new()
        .route("/records", get(api::get_records))
        .route("/errors", get(api::get_errors));

    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
