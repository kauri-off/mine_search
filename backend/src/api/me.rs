use axum::http::StatusCode;

pub async fn me() -> StatusCode {
    StatusCode::OK
}
