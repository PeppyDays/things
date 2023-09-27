use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};

pub fn create_router() -> Router {
    Router::new().route("/", get(check_health))
}

async fn check_health() -> impl IntoResponse {
    StatusCode::OK
}
