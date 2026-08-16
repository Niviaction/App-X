mod auth;
mod state;

use axum::http::{HeaderValue, Method};
use axum::middleware;
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;

use auth::CurrentUser;
use state::AppState;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("failed to connect to database");

    sqlx::migrate!()
        .run(&db)
        .await
        .expect("failed to run migrations");

    let state = AppState { db };

    let frontend_origin = std::env::var("FRONTEND_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());
    let cors = CorsLayer::new()
        .allow_origin(
            frontend_origin
                .parse::<HeaderValue>()
                .expect("FRONTEND_ORIGIN must be a valid origin"),
        )
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([axum::http::header::CONTENT_TYPE]);

    let protected = Router::new()
        .route("/me", get(me))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth::require_auth));

    let app = Router::new()
        .route("/health", get(health))
        .route("/signup", post(auth::signup))
        .route("/login", post(auth::login))
        .route("/logout", post(auth::logout))
        .merge(protected)
        .layer(cors)
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();

    println!("Backend running on http://0.0.0.0:{port}");

    axum::serve(listener, app).await.unwrap();
}

async fn health() -> &'static str {
    "OK"
}

#[derive(Serialize)]
struct MeResponse {
    id: uuid::Uuid,
}

async fn me(Extension(user): Extension<CurrentUser>) -> Json<MeResponse> {
    Json(MeResponse { id: user.id })
}
