use axum::{extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use domain::identity::errors::Error as IdentityError;

use crate::extractors::Json;
use crate::handlers::common::parse_identity_user;
use crate::{container::Container, errors::Error};

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    id: Uuid,
    role: String,
    refresh_token: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    access_token: String,
    refresh_token: String,
}

pub async fn handle(
    State(container): State<Container>,
    Json(request): Json<Request>,
) -> Result<Json<Response>, Error> {
    let identity_user = parse_identity_user(request.id, request.role)?;

    let tokens = container
        .identity_service
        .refresh_tokens(identity_user, request.refresh_token.as_str().into())
        .await
        .map_err(|error| match error {
            IdentityError::TokensRefreshFailed(..) => {
                Error::new(StatusCode::UNAUTHORIZED, error.to_string())
            }
            IdentityError::IdentityNotFound(..) | IdentityError::InvalidRole { .. } => {
                Error::new(StatusCode::BAD_REQUEST, error.to_string())
            }
            _ => Error::new(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
        })?;

    Ok(Json(Response {
        access_token: tokens.access_token.into(),
        refresh_token: tokens.refresh_token.into(),
    }))
}
