use std::env;

use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response, Json};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    pub exp: usize, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    pub iat: usize, // Optional. Issued at (as UTC timestamp)
    pub ip: String,
}

pub async fn middleware_check(req: Request, next: Next) -> Result<Response, StatusCode> {
    if let Some(jwt) = req.headers().get("Authorization") {
        let secret = env::var("BACKEND_SECRET").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let _claims = decode::<Claims>(
            jwt.to_str().unwrap(),
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        )
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

        return Ok(next.run(req).await);
    }

    return Err(StatusCode::UNAUTHORIZED);
}

#[derive(Serialize, Deserialize)]
pub struct AuthBody {
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct AuthReturn {
    pub token: String,
}

pub async fn authenticate_user(Json(body): Json<AuthBody>) -> Result<Json<AuthReturn>, StatusCode> {
    let password = env::var("BACKEND_PASSWORD").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let secret = env::var("BACKEND_SECRET").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if body.password != password {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let now = Utc::now();
    let expire: chrono::TimeDelta = Duration::hours(24);
    let exp: usize = (now + expire).timestamp() as usize;
    let iat: usize = now.timestamp() as usize;

    let my_claims = Claims {
        exp,
        iat,
        ip: "temp".to_string(),
    };

    let jwt = encode(
        &Header::default(),
        &my_claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AuthReturn { token: jwt }))
}

pub async fn validate_credentials() {}
