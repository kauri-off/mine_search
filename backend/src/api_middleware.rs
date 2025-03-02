use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use cookie::Cookie;

use crate::jwt_wrapper::jwt_decode;

pub async fn middleware_check(req: Request, next: Next) -> Result<Response, StatusCode> {
    if let Some(cookie_header) = req.headers().get("Cookie") {
        if let Some(cookie) = Cookie::split_parse(
            cookie_header
                .to_str()
                .map_err(|_| StatusCode::UNAUTHORIZED)?,
        )
        .find(|c| c.as_ref().map_or(false, |c| c.name() == "token"))
        {
            if let Ok(c) = cookie {
                let token = c.value();
                let _claims = jwt_decode(token).map_err(|_| StatusCode::UNAUTHORIZED)?;
                return Ok(next.run(req).await);
            }
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}
