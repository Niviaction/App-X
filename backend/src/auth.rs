use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::state::AppState;

const SESSION_COOKIE: &str = "session_token";
const SESSION_DAYS: i64 = 7;

#[derive(Deserialize)]
pub struct SignupRequest {
    email: String,
    password: String,
    display_name: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    identifier: String,
    password: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    id: Uuid,
    email: String,
    display_name: Option<String>,
}

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}

fn error(status: StatusCode, message: &str) -> Response {
    (status, Json(ErrorResponse { message: message.to_string() })).into_response()
}

fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}

fn verify_password(password: &str, hash: &str) -> bool {
    let Ok(parsed_hash) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

fn session_cookie(token: Uuid) -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE, token.to_string()))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::None)
        .path("/")
        .max_age(time::Duration::days(SESSION_DAYS))
        .build()
}

async fn create_session(db: &PgPool, user_id: Uuid) -> Result<Uuid, sqlx::Error> {
    let expires_at = Utc::now() + Duration::days(SESSION_DAYS);
    let row = sqlx::query!(
        "INSERT INTO sessions (user_id, expires_at) VALUES ($1, $2) RETURNING token",
        user_id,
        expires_at,
    )
    .fetch_one(db)
    .await?;
    Ok(row.token)
}

pub async fn signup(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<SignupRequest>,
) -> Response {
    let email = body.email.trim().to_lowercase();
    if email.is_empty() || !email.contains('@') {
        return error(StatusCode::BAD_REQUEST, "A valid email is required");
    }
    if body.password.len() < 8 {
        return error(StatusCode::BAD_REQUEST, "Password must be at least 8 characters");
    }

    let password_hash = match hash_password(&body.password) {
        Ok(h) => h,
        Err(_) => return error(StatusCode::INTERNAL_SERVER_ERROR, "Could not process password"),
    };

    let row = sqlx::query!(
        "INSERT INTO users (email, password_hash, display_name) VALUES ($1, $2, $3)
         RETURNING id, email, display_name",
        email,
        password_hash,
        body.display_name,
    )
    .fetch_one(&state.db)
    .await;

    let user = match row {
        Ok(u) => u,
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => {
            return error(StatusCode::CONFLICT, "An account with this email already exists");
        }
        Err(_) => return error(StatusCode::INTERNAL_SERVER_ERROR, "Could not create account"),
    };

    let token = match create_session(&state.db, user.id).await {
        Ok(t) => t,
        Err(_) => return error(StatusCode::INTERNAL_SERVER_ERROR, "Could not start session"),
    };

    let jar = jar.add(session_cookie(token));
    (
        StatusCode::CREATED,
        jar,
        Json(UserResponse {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
        }),
    )
        .into_response()
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<LoginRequest>,
) -> Response {
    let email = body.identifier.trim().to_lowercase();

    let user = sqlx::query!(
        "SELECT id, email, password_hash, display_name FROM users WHERE email = $1",
        email,
    )
    .fetch_optional(&state.db)
    .await;

    let user = match user {
        Ok(Some(u)) => u,
        Ok(None) => return error(StatusCode::UNAUTHORIZED, "Invalid email or password"),
        Err(_) => return error(StatusCode::INTERNAL_SERVER_ERROR, "Could not log in"),
    };

    if !verify_password(&body.password, &user.password_hash) {
        return error(StatusCode::UNAUTHORIZED, "Invalid email or password");
    }

    let token = match create_session(&state.db, user.id).await {
        Ok(t) => t,
        Err(_) => return error(StatusCode::INTERNAL_SERVER_ERROR, "Could not start session"),
    };

    let jar = jar.add(session_cookie(token));
    (
        StatusCode::OK,
        jar,
        Json(UserResponse {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
        }),
    )
        .into_response()
}

pub async fn logout(State(state): State<AppState>, jar: CookieJar) -> Response {
    if let Some(cookie) = jar.get(SESSION_COOKIE)
        && let Ok(token) = Uuid::parse_str(cookie.value())
    {
        let _ = sqlx::query!("DELETE FROM sessions WHERE token = $1", token)
            .execute(&state.db)
            .await;
    }
    let jar = jar.remove(Cookie::from(SESSION_COOKIE));
    (StatusCode::NO_CONTENT, jar).into_response()
}

/// Attached to `request.extensions()` by [`require_auth`] for downstream handlers.
#[derive(Clone)]
pub struct CurrentUser {
    pub id: Uuid,
}

pub async fn require_auth(
    State(state): State<AppState>,
    jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Response {
    let Some(cookie) = jar.get(SESSION_COOKIE) else {
        return error(StatusCode::UNAUTHORIZED, "Not logged in");
    };
    let Ok(token) = Uuid::parse_str(cookie.value()) else {
        return error(StatusCode::UNAUTHORIZED, "Not logged in");
    };

    let row = sqlx::query!(
        "SELECT user_id FROM sessions WHERE token = $1 AND expires_at > now()",
        token,
    )
    .fetch_optional(&state.db)
    .await;

    match row {
        Ok(Some(r)) => {
            req.extensions_mut().insert(CurrentUser { id: r.user_id });
            next.run(req).await
        }
        Ok(None) => error(StatusCode::UNAUTHORIZED, "Session expired or invalid"),
        Err(_) => error(StatusCode::INTERNAL_SERVER_ERROR, "Could not verify session"),
    }
}
