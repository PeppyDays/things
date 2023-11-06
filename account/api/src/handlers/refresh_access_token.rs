use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use domain::identity::commands::Command as IdentityCommand;
use domain::identity::errors::Error as IdentityError;

use crate::{container::Container, errors::Error};

#[derive(Deserialize, Clone)]
pub struct Request {
    id: Uuid,
    role: String,
    refresh_token: String,
}

#[derive(Serialize)]
pub struct Response {
    access_token: String,
    refresh_token: String,
}

pub async fn handle(
    State(mut container): State<Container>,
    Json(request): Json<Request>,
) -> Result<Json<Response>, Error> {
    let command = IdentityCommand::RefreshAccessToken {
        id: request.id,
        role: request.role,
        refresh_token: request.refresh_token,
    };

    container
        .identity_command_executor
        .execute(command)
        .await
        .map_err(|error| match error {
            IdentityError::TokenRefreshFailed { .. } => {
                Error::new(StatusCode::UNAUTHORIZED, error.to_string())
            }
            IdentityError::EntityNotFound { .. } | IdentityError::InvalidRole { .. } => {
                Error::new(StatusCode::BAD_REQUEST, error.to_string())
            }
            _ => Error::new(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
        })?
        .map_or(
            Err(Error::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("Tokens are not found"),
            )),
            |tokens| {
                Ok(Json(Response {
                    access_token: tokens.access_token.0,
                    refresh_token: tokens.refresh_token.0,
                }))
            },
        )
}
