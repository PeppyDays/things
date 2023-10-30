use axum::http::StatusCode;
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use domain::identity::commands::Command as IdentityCommand;
use domain::identity::errors::Error as IdentityError;
use domain::identity::models::Role as IdentityRole;
use domain::user::commands::Command as UserCommand;
use domain::user::errors::Error as UserError;

use crate::{container::Container, errors::Error};

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

    register_user(&mut container, request, id).await?;
    register_identity(&mut container, id).await?;

    Ok(Json(Response { id }))
}

async fn register_identity(container: &mut Container, id: Uuid) -> Result<(), Error> {
    let command = IdentityCommand::RegisterIdentity {
        id,
        role: IdentityRole::Member,
    };

    container
        .identity_command_executor
        .execute(command)
        .await
        .map_err(|error| match error {
            IdentityError::AlreadyRegistered { .. } => {
                Error::new(StatusCode::CONFLICT, error.to_string())
            }
            _ => Error::new(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
        })?;

    Ok(())
}

async fn register_user(container: &mut Container, request: Request, id: Uuid) -> Result<(), Error> {
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
        .map_err(|error| match error {
            UserError::AlreadyRegistered { .. } => {
                Error::new(StatusCode::CONFLICT, error.to_string())
            }
            _ => Error::new(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
        })?;

    Ok(())
}
