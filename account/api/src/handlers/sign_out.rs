use axum::{
    extract::State,
    headers::{authorization::Bearer, Authorization},
    http::StatusCode,
    TypedHeader,
};

use domain::identity::errors::Error as IdentityError;
use domain::identity::models::User as IdentityUser;

use crate::{container::Container, errors::Error};

pub async fn handle(
    State(container): State<Container>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
) -> Result<StatusCode, Error> {
    let identity_user =
        extract_identity_user_from_authorization_header(authorization, &container).await?;

    container
        .identity_service
        .invalidate_tokens(identity_user)
        .await
        .map_err(|error| match error {
            IdentityError::EntityNotFound { .. } => {
                Error::new(StatusCode::NOT_FOUND, error.to_string())
            }
            IdentityError::InvalidRole { .. } => {
                Error::new(StatusCode::BAD_REQUEST, error.to_string())
            }
            _ => Error::new(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
        })?;

    Ok(StatusCode::OK)
}

async fn extract_identity_user_from_authorization_header(
    authorization: Authorization<Bearer>,
    container: &Container,
) -> Result<IdentityUser, Error> {
    container
        .identity_service
        .verify_access_token(&authorization.token().to_string().into())
        .await
        .map_err(|error| match error {
            IdentityError::TokenValidationFailed { .. } => {
                Error::new(StatusCode::UNAUTHORIZED, error.to_string())
            }
            _ => Error::new(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
        })
}
