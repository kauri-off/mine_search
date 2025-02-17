use std::env;

use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};

pub async fn password_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    if let Ok(password) = env::var("BACKEND_PASSWORD") {
        if let Some(x_password) = req.headers().get("x-password") {
            if password.ne(x_password) {
                return Err(StatusCode::UNAUTHORIZED);
            }
        } else {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    return Ok(next.run(req).await);
}
