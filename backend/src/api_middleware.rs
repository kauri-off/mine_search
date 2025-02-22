use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use cookie::Cookie;

use crate::jwt_wrapper::jwt_decode;

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
                let _claims = jwt_decode(token).map_err(|_| StatusCode::UNAUTHORIZED)?;

                return Ok(next.run(req).await);
            }
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}
