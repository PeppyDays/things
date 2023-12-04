use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

use crate::{container::Container, errors::Error};
use domain::identity::errors::Error as IdentityError;

pub async fn require_authentication(
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    State(container): State<Container>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, Error> {
    let identity_user = container
        .identity_service
        .verify_access_token(&authorization.token().into())
        .await
        .map_err(|error| match error {
            IdentityError::TokensValidationFailed(..) => {
                Error::new(StatusCode::UNAUTHORIZED, error.to_string())
            }
            _ => Error::new(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
        })?;

    request.extensions_mut().insert(identity_user);
    Ok(next.run(request).await)
}
