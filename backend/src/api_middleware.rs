use std::env;

use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::api::auth::Claims;

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
