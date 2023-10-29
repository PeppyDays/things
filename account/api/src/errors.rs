use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

pub struct Error {
    code: StatusCode,
    message: String,
}

impl Error {
    pub fn new(code: StatusCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        (
            self.code,
            Json(Message {
                error: self.message,
            }),
        )
            .into_response()
    }
}

#[derive(Serialize)]
struct Message {
    error: String,
}
