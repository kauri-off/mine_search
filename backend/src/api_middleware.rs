use std::env;

use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use cookie::Cookie;
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::api::auth::Claims;

pub async fn middleware_check(req: Request, next: Next) -> Result<Response, StatusCode> {
    if let Some(cookie_header) = req.headers().get("Cookie") {
        for cookie in Cookie::split_parse(
            cookie_header
                .to_str()
                .map_err(|_| StatusCode::UNAUTHORIZED)?,
        ) {
            let cookie = match cookie {
                Ok(c) => c,
                Err(_) => continue,
            };

            if cookie.name() == "token" {
                let token = cookie.value();
                let secret =
                    env::var("BACKEND_SECRET").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                let _claims = decode::<Claims>(
                    token,
                    &DecodingKey::from_secret(secret.as_ref()),
                    &Validation::default(),
                )
                .map_err(|_| StatusCode::UNAUTHORIZED)?;

                return Ok(next.run(req).await);
            }
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}
