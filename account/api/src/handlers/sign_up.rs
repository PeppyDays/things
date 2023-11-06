use axum::http::StatusCode;
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use domain::identity::errors::Error as IdentityError;
use domain::user::commands::Command as UserCommand;
use domain::user::errors::Error as UserError;

use crate::handlers::parse_identity_user;
use crate::{container::Container, errors::Error};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    name: String,
    password: String,
    email: String,
    language: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    id: Uuid,
}

pub async fn handle(
    State(mut container): State<Container>,
    Json(request): Json<Request>,
) -> Result<Json<Response>, Error> {
    let id = Uuid::new_v4();

    register_user(&mut container, request, id).await?;
    register_identity(&mut container, id).await?;

    Ok(Json(Response { id }))
}

async fn register_identity(container: &mut Container, id: Uuid) -> Result<(), Error> {
    let identity_user = parse_identity_user(id, String::from("Member"))?;

    container
        .identity_service
        .register_identity(identity_user)
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
