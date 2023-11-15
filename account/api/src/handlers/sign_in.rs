use axum::{extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use domain::identity::errors::Error as IdentityError;
use domain::identity::models::entities::Tokens as IdentityTokens;
use domain::user::errors::Error as UserError;
use domain::user::queries::Query as UserQuery;

use crate::extractors::Json;
use crate::handlers::common::parse_identity_user;
use crate::{container::Container, errors::Error};

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    id: Uuid,
    role: String,
    password: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    access_token: String,
    refresh_token: String,
}

pub async fn handle(
    State(mut container): State<Container>,
    Json(request): Json<Request>,
) -> Result<Json<Response>, Error> {
    verify_credential(&container, request.clone()).await?;
    let tokens = issue_tokens(&mut container, request).await?;

    Ok(Json(Response {
        access_token: tokens.access_token.into(),
        refresh_token: tokens.refresh_token.into(),
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
        .map_err(|error| {
            let message = error.to_string();
            log::error!("Failed to sign in: {}", &message);

            match error {
                UserError::InvalidCredential(..) | UserError::UserNotFound(..) => Error::new(
                    StatusCode::UNAUTHORIZED,
                    "Failed to sign in due to the invalid credential",
                ),
                _ => Error::new(StatusCode::INTERNAL_SERVER_ERROR, &message),
            }
        })
        .map(|_| ())
}

async fn issue_tokens(
    container: &mut Container,
    request: Request,
) -> Result<IdentityTokens, Error> {
    let identity_user = parse_identity_user(request.id, request.role)?;

    container
        .identity_service
        .issue_tokens(identity_user)
        .await
        .map_err(|error| match error {
            IdentityError::IdentityNotFound(..) => Error::new(
                StatusCode::UNAUTHORIZED,
                "No identity found for the given user",
            ),
            _ => Error::new(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
        })
}
