use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

use axum::{
    Json,
    http::{HeaderMap, HeaderValue, StatusCode, header::SET_COOKIE},
    response::IntoResponse,
};
use chrono::Utc;
use cookie::{Cookie, SameSite, time};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    BACKEND_PASSWORD,
    error::AppError,
    jwt_wrapper::{Claims, jwt_encode},
};

// ---------------------------------------------------------------------------
// Rate limiter
//
// Each IP is allowed MAX_ATTEMPTS login attempts within the WINDOW duration.
// Entries older than WINDOW are pruned on every request so memory stays
// bounded even under sustained attack.
// ---------------------------------------------------------------------------

const MAX_ATTEMPTS: u32 = 5;
const WINDOW: Duration = Duration::from_secs(60);

struct RateLimitEntry {
    attempts: u32,
    window_start: Instant,
}

lazy_static! {
    static ref RATE_LIMIT: Mutex<HashMap<String, RateLimitEntry>> = Mutex::new(HashMap::new());
}

/// Returns `true` if the request from `ip` should be allowed through.
/// Increments the attempt counter; prunes stale entries on every call.
fn check_rate_limit(ip: &str) -> bool {
    let mut map = RATE_LIMIT.lock().expect("rate limit mutex poisoned");
    let now = Instant::now();

    // Prune entries whose window has fully expired.
    map.retain(|_, entry| now.duration_since(entry.window_start) < WINDOW);

    let entry = map.entry(ip.to_owned()).or_insert(RateLimitEntry {
        attempts: 0,
        window_start: now,
    });

    // Reset the window if it has expired for this IP.
    if now.duration_since(entry.window_start) >= WINDOW {
        entry.attempts = 0;
        entry.window_start = now;
    }

    entry.attempts += 1;
    entry.attempts <= MAX_ATTEMPTS
}

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AuthBody {
    pub password: String,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AuthReturn {
    pub token: String,
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

pub async fn authenticate_user(
    headers: HeaderMap,
    Json(body): Json<AuthBody>,
) -> Result<impl IntoResponse, AppError> {
    let ip = headers
        .get("X-Real-IP")
        .or_else(|| headers.get("X-Forwarded-For"))
        .and_then(|val| val.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    if !check_rate_limit(&ip) {
        tracing::warn!("Rate limit exceeded for IP: {ip}");
        return Err(AppError::RateLimited);
    }

    if body.password != *BACKEND_PASSWORD.lock().await {
        tracing::warn!("Failed login attempt from IP: {ip}");
        return Err(AppError::Unauthorized);
    }

    let now = Utc::now();
    let expire = chrono::Duration::hours(24);
    let exp: usize = (now + expire).timestamp() as usize;
    let iat: usize = now.timestamp() as usize;

    let my_claims = Claims { exp, iat, ip };

    let jwt = jwt_encode(my_claims)
        .await
        .map_err(|_| AppError::internal("JWT encode failed", "jwt_encode returned an error"))?;

    let cookie = Cookie::build(("token", &jwt))
        .path("/api")
        .http_only(true)
        .same_site(SameSite::Strict)
        .max_age(time::Duration::days(30));

    let cookie_str = cookie.to_string();
    let cookie_value = HeaderValue::from_str(&cookie_str)
        .map_err(|e| AppError::internal("Invalid cookie header value", e))?;

    let mut response_headers = HeaderMap::new();
    response_headers.insert(SET_COOKIE, cookie_value);

    Ok((
        StatusCode::OK,
        response_headers,
        Json(AuthReturn { token: jwt }),
    ))
}
