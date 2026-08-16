use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(health));

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();

    println!("Backend running on http://0.0.0.0:{port}");

    axum::serve(listener, app)
        .await
        .unwrap();
}

async fn health() -> &'static str {
    "OK"
}