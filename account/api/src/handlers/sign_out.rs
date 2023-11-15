use axum::{extract::State, http::StatusCode, Extension};

use domain::identity::errors::Error as IdentityError;
use domain::identity::models::entities::User as IdentityUser;

use crate::{container::Container, errors::Error};

pub async fn handle(
    Extension(identity_user): Extension<IdentityUser>,
    State(container): State<Container>,
) -> Result<StatusCode, Error> {
    container
        .identity_service
        .invalidate_tokens(identity_user)
        .await
        .map_err(|error| {
            let message = error.to_string();
            log::error!("Failed to sign out: {}", &message);

            match error {
                IdentityError::IdentityNotFound(..) => Error::new(StatusCode::NOT_FOUND, &message),
                IdentityError::InvalidRole { .. } => Error::new(StatusCode::BAD_REQUEST, &message),
                _ => Error::new(StatusCode::INTERNAL_SERVER_ERROR, &message),
            }
        })?;

    Ok(StatusCode::OK)
}
