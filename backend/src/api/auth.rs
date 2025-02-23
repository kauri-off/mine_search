use std::env;

use axum::{
    http::{header::SET_COOKIE, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{Duration, Utc};
use cookie::{time, Cookie, SameSite};
use serde::{Deserialize, Serialize};

use crate::jwt_wrapper::{jwt_encode, Claims};

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
) -> Result<impl IntoResponse, StatusCode> {
    let password = env::var("BACKEND_PASSWORD").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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

    let jwt = jwt_encode(my_claims).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let cookie = Cookie::build(("token", &jwt))
        .path("/api")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .max_age(time::Duration::days(30));

    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        HeaderValue::from_str(&cookie.to_string()).unwrap(),
    );

    Ok((headers, Json(AuthReturn { token: jwt })))
}

pub async fn validate_credentials() {}
