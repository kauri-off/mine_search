use axum::{extract::Request, middleware::Next, response::Response};
use cookie::Cookie;

use crate::{error::AppError, jwt_wrapper::jwt_decode};

pub async fn middleware_check(req: Request, next: Next) -> Result<Response, AppError> {
    let authorized = 'auth: {
        let Some(cookie_header) = req.headers().get("Cookie") else {
            break 'auth false;
        };

        let Ok(raw) = cookie_header.to_str() else {
            break 'auth false;
        };

        let token = Cookie::split_parse(raw)
            .filter_map(|c| c.ok())
            .find(|c| c.name() == "token")
            .map(|c| c.value().to_owned());

        let Some(token) = token else {
            break 'auth false;
        };

        jwt_decode(&token).await.is_ok()
    };

    if authorized {
        Ok(next.run(req).await)
    } else {
        Err(AppError::Unauthorized)
    }
}
