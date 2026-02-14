use axum::{
    Json,
    http::{HeaderMap, HeaderValue, StatusCode, header::SET_COOKIE},
    response::IntoResponse,
};
use cookie::{Cookie, SameSite, time};
use serde::{Deserialize, Serialize};

use crate::jwt_wrapper::jwt_decode;

#[derive(Serialize, Deserialize)]
pub struct Token {
    pub token: String,
}

pub async fn set_cookie(Json(token): Json<Token>) -> Result<impl IntoResponse, StatusCode> {
    jwt_decode(&token.token)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let cookie = Cookie::build(("token", token.token))
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

    Ok(headers)
}
