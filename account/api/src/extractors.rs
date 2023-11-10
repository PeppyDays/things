use axum::{
    extract::{rejection::JsonRejection, FromRequest},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Serialize;

use crate::errors::Error;

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(Error))]
pub struct Json<T>(pub T);

impl From<JsonRejection> for Error {
    fn from(rejection: JsonRejection) -> Self {
        Error::new(StatusCode::UNPROCESSABLE_ENTITY, rejection.body_text())
    }
}

impl<T: Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> axum::response::Response {
        let Self(value) = self;
        axum::Json(value).into_response()
    }
}
