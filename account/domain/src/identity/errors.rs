use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to register user {0} because it was already registered")]
    IdentityAlreadyRegistered(Uuid),

    #[error("Failed to find user {0}'s identity")]
    IdentityNotFound(Uuid),

    #[error("Failed to transform {0} to the user's role")]
    InvalidRole(String),

    #[error("Failed to create tokens")]
    TokensCreationFailed(#[source] jsonwebtoken::errors::Error),

    #[error("Failed to find persisted refresh token of user {0}")]
    RefreshTokenNotFound(Uuid),

    #[error("Failed to match persisted refresh token to the given refresh token of user {0}")]
    RefreshTokenMismatched(Uuid),

    #[error("Failed to refresh tokens")]
    TokensRefreshFailed(#[source] jsonwebtoken::errors::Error),

    #[error("Failed to decode tokens")]
    TokensValidationFailed(#[source] jsonwebtoken::errors::Error),

    #[error("Failed to operate on the database: {0}")]
    DatabaseOperationFailed(#[source] anyhow::Error),

    #[error("Failed due to unknown or undefined errors: {0}")]
    Unexpected(#[source] anyhow::Error),
}
