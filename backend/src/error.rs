use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

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
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Database(_) | AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        status.into_response()
    }
}
