use axum::{
    extract::State,
    headers::{authorization::Bearer, Authorization},
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    TypedHeader,
};

use crate::{container::Container, errors::Error};
use domain::identity::errors::Error as IdentityError;

pub async fn require_authentication<B>(
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    State(container): State<Container>,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, Error> {
    let identity_user = container
        .identity_service
        .verify_access_token(&authorization.token().into())
        .await
        .map_err(|error| match error {
            IdentityError::TokenValidationFailed { .. } => {
                Error::new(StatusCode::UNAUTHORIZED, error.to_string())
            }
            _ => Error::new(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
        })?;

    request.extensions_mut().insert(identity_user);
    Ok(next.run(request).await)
}
