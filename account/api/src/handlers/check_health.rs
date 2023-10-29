use axum::{http::StatusCode, response::IntoResponse};

pub async fn check_health() -> impl IntoResponse {
    StatusCode::OK
}
