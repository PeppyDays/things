use axum::http::StatusCode;
use uuid::Uuid;

use domain::identity::errors::Error as IdentityError;
use domain::identity::models::entities::User as IdentityUser;

use crate::errors::Error;

pub fn parse_identity_user(id: Uuid, role: String) -> Result<IdentityUser, Error> {
    Ok(IdentityUser::new(
        id,
        role.as_str().try_into().map_err(|error: IdentityError| {
            let message = error.to_string();
            log::warn!("Failed to parse the requested role {}: {}", role, &message);

            Error::new(StatusCode::BAD_REQUEST, &message)
        })?,
    ))
}
