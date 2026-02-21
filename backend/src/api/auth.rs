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
) -> Result<impl IntoResponse, StatusCode> {
    if body.password != *BACKEND_PASSWORD.lock().await {
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

    let jwt = jwt_encode(my_claims)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let cookie = Cookie::build(("token", &jwt))
        .path("/api")
        .http_only(true)
        .same_site(SameSite::Strict)
        .max_age(time::Duration::days(30));

    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        HeaderValue::from_str(&cookie.to_string()).unwrap(),
    );

    Ok((headers, Json(AuthReturn { token: jwt })))
}
