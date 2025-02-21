use std::env;

use axum::{
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    pub iat: usize, // Optional. Issued at (as UTC timestamp)
    pub ip: String,
}

#[derive(Serialize, Deserialize)]
pub struct AuthBody {
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct AuthReturn {
    pub token: String,
}
pub async fn authenticate_user(
    headers: HeaderMap,
    Json(body): Json<AuthBody>,
) -> Result<Json<AuthReturn>, StatusCode> {
    let password = env::var("BACKEND_PASSWORD").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let secret = env::var("BACKEND_SECRET").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if body.password != password {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let now = Utc::now();
    let expire: Duration = Duration::hours(24);
    let exp: usize = (now + expire).timestamp() as usize;
    let iat: usize = now.timestamp() as usize;

    let ip = headers
        .get("X-Real-IP")
        .or_else(|| headers.get("X-Forwarded-For"))
        .and_then(|val| val.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let my_claims = Claims { exp, iat, ip };

    let jwt = encode(
        &Header::default(),
        &my_claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AuthReturn { token: jwt }))
}

pub async fn validate_credentials() {}
