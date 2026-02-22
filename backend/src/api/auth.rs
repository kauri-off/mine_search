use axum::{
    Json,
    http::{HeaderMap, HeaderValue, StatusCode, header::SET_COOKIE},
    response::IntoResponse,
};
use chrono::{Duration, Utc};
use cookie::{Cookie, SameSite, time};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    BACKEND_PASSWORD,
    error::AppError,
    jwt_wrapper::{Claims, jwt_encode},
};

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

pub async fn authenticate_user(
    headers: HeaderMap,
    Json(body): Json<AuthBody>,
) -> Result<impl IntoResponse, AppError> {
    if body.password != *BACKEND_PASSWORD.lock().await {
        return Err(AppError::Unauthorized);
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
