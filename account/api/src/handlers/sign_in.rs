use axum::{extract::State, http::StatusCode, Json};
use serde::Deserialize;
use uuid::Uuid;

use domain::user::errors::Error as UserError;
use domain::user::queries::Query as UserQuery;

use crate::{container::Container, errors::Error};

#[derive(Deserialize)]
pub struct Request {
    id: Uuid,
    password: String,
}

pub async fn sign_in_with_credential(
    State(container): State<Container>,
    Json(request): Json<Request>,
) -> Result<(), Error> {
    verify_credential(container, request).await?;
    Ok(())
}

async fn verify_credential(container: Container, request: Request) -> Result<(), Error> {
    let query = UserQuery::VerifyCredential {
        id: request.id,
        password: request.password,
    };

    container
        .user_query_reader
        .read(query)
        .await
        .map_err(|error| match error {
            UserError::InvalidCredential => Error::new(StatusCode::UNAUTHORIZED, error.to_string()),
            _ => Error::new(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
        })
        .map(|_| ())
}
