use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use domain::identity::errors::Error as IdentityError;
use domain::identity::{commands::Command as IdentityCommand, models::Tokens};
use domain::user::errors::Error as UserError;
use domain::user::queries::Query as UserQuery;

use crate::{container::Container, errors::Error};

#[derive(Deserialize, Clone)]
pub struct Request {
    id: Uuid,
    role: String,
    password: String,
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
    verify_credential(&container, request.clone()).await?;
    let tokens = issue_access_and_refresh_tokens(&mut container, request).await?;

    Ok(Json(Response {
        access_token: tokens.access_token.0,
        refresh_token: tokens.refresh_token.0,
    }))
}

async fn verify_credential(container: &Container, request: Request) -> Result<(), Error> {
    let query = UserQuery::VerifyCredential {
        id: request.id,
        password: request.password,
    };

    container
        .user_query_reader
        .read(query)
        .await
        .map_err(|error| match error {
            UserError::InvalidCredential | UserError::EntityNotFound { .. } => Error::new(
                StatusCode::UNAUTHORIZED,
                "Failed to sign in due to the invalid credential",
            ),
            _ => Error::new(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
        })
        .map(|_| ())
}

async fn issue_access_and_refresh_tokens(
    container: &mut Container,
    request: Request,
) -> Result<Tokens, Error> {
    let command = IdentityCommand::IssueAccessToken {
        id: request.id,
        role: request.role,
    };

    container
        .identity_command_executor
        .execute(command)
        .await
        .map_err(|error| match error {
            IdentityError::EntityNotFound { .. } => Error::new(
                StatusCode::UNAUTHORIZED,
                "No identity found for the given user",
            ),
            _ => Error::new(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
        })?
        .ok_or_else(|| Error::new(StatusCode::UNAUTHORIZED, "Failed to issue access token"))
}
