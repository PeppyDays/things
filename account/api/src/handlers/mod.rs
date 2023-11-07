use axum::http::StatusCode;
use uuid::Uuid;

use domain::identity::errors::Error as IdentityError;
use domain::identity::models::entities::User as IdentityUser;

use crate::errors::Error;

pub mod check_health;
pub mod get_user;
pub mod refresh_access_token;
pub mod sign_in;
pub mod sign_out;
pub mod sign_up;

fn parse_identity_user(id: Uuid, role: String) -> Result<IdentityUser, Error> {
    Ok(IdentityUser::new(
        id,
        role.as_str().try_into().map_err(|error: IdentityError| {
            Error::new(StatusCode::BAD_REQUEST, error.to_string())
        })?,
    ))
}
