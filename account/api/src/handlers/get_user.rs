use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use domain::user::queries::Query;
use serde::Serialize;
use uuid::Uuid;

use domain::user::errors::Error as UserError;

use crate::{container::Container, errors::Error};

#[derive(Serialize)]
pub struct Response {
    id: Uuid,
    name: String,
    email: String,
    language: String,
}

pub async fn handle(
    State(container): State<Container>,
    Path(id): Path<Uuid>,
) -> Result<Json<Response>, Error> {
    let query = Query::GetUser { id };
    container
        .user_query_reader
        .read(query)
        .await
        .map(|u| {
            Json(Response {
                id: u.id,
                name: u.name,
                email: u.email,
                language: u.language,
            })
        })
        .map_err(|error| match error {
            UserError::AlreadyWithdrawn { .. } => {
                Error::new(StatusCode::FORBIDDEN, error.to_string())
            }
            _ => Error::new(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
        })
}
