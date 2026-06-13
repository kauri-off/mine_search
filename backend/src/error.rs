use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

/// A unified application error type.
///
/// All handler functions return `Result<T, AppError>`. Axum converts it into
/// an HTTP response via the `IntoResponse` impl below, while also logging the
/// underlying cause so nothing is silently swallowed.
#[allow(unused)]
#[derive(Debug)]
pub enum AppError {
    /// The request is missing valid authentication.
    Unauthorized,

    /// The caller has exceeded the allowed request rate.
    RateLimited,

    /// The request body or parameters are invalid.
    BadRequest(String),

    /// A database / pool operation failed.
    Database(String),

    /// Something unexpected happened that is not the caller's fault.
    Internal(String),
}

impl AppError {
    /// Convenience constructor that logs the error and wraps it as `Database`.
    pub fn db(context: impl std::fmt::Display, err: impl std::fmt::Display) -> Self {
        tracing::error!("{context}: {err}");
        Self::Database(context.to_string())
    }

    /// Convenience constructor that logs the error and wraps it as `Internal`.
    pub fn internal(context: impl std::fmt::Display, err: impl std::fmt::Display) -> Self {
        tracing::error!("{context}: {err}");
        Self::Internal(context.to_string())
    }

    /// Convenience constructor for bad request errors.
    pub fn bad_request(msg: impl std::fmt::Display) -> Self {
        Self::BadRequest(msg.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Unauthorized => StatusCode::UNAUTHORIZED.into_response(),
            AppError::RateLimited => StatusCode::TOO_MANY_REQUESTS.into_response(),
            // Bad requests are the caller's fault and safe to explain, so surface the
            // reason in a JSON body the UI can display.
            AppError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, Json(json!({ "error": msg }))).into_response()
            }
            // Server-side failures are already logged at construction; keep the
            // response opaque so we never leak internals to the client.
            AppError::Database(_) | AppError::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
