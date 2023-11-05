use axum::{
    extract::State,
    headers::{authorization::Bearer, Authorization},
    http::StatusCode,
    TypedHeader,
};

use domain::identity::errors::Error as IdentityError;
use domain::identity::queries::Query as IdentityQuery;
use domain::identity::{commands::Command as IdentityCommand, models::User as IdentityUser};

use crate::{container::Container, errors::Error};

pub async fn sign_out(
    State(mut container): State<Container>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
) -> Result<StatusCode, Error> {
    let user = extract_user_from_authorization_header(authorization, &container).await?;
    let command = IdentityCommand::InvalidateRefreshToken {
        id: user.id,
        role: user.role,
    };

    container
        .identity_command_executor
        .execute(command)
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

async fn extract_user_from_authorization_header(
    authorization: Authorization<Bearer>,
    container: &Container,
) -> Result<IdentityUser, Error> {
    let query = IdentityQuery::GetUserFromAccessToken {
        access_token: String::from(authorization.token()),
    };

    container
        .identity_query_reader
        .read(query)
        .await
        .map_err(|error| match error {
            IdentityError::TokenValidationFailed { .. } => {
                Error::new(StatusCode::UNAUTHORIZED, error.to_string())
            }
            _ => Error::new(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
        })
}
