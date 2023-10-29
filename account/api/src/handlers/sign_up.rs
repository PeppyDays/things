use axum::http::StatusCode;
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{container::Container, errors::Error};
use domain::user::commands::Command as UserCommand;
use domain::user::errors::Error as UserError;

#[derive(Deserialize)]
pub struct Request {
    name: String,
    password: String,
    email: String,
    language: String,
}

#[derive(Serialize)]
pub struct Response {
    id: Uuid,
}

pub async fn sign_up_with_credential(
    State(mut container): State<Container>,
    Json(request): Json<Request>,
) -> Result<Json<Response>, Error> {
    let id = Uuid::new_v4();
    let command = UserCommand::RegisterUser {
        id,
        name: request.name,
        password: request.password,
        email: request.email,
        language: request.language,
    };

    container
        .user_command_executor
        .execute(command)
        .await
        .map(|_| Json(Response { id }))
        .map_err(|error| match error {
            UserError::AlreadyRegistered { .. } => {
                Error::new(StatusCode::CONFLICT, error.to_string())
            }
            _ => Error::new(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
        })
}
