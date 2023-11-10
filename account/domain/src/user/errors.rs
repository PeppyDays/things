use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to register user {0} because it was already registered")]
    UserAlreadyRegistered(Uuid),

    #[error("Failed to withdraw user {0} because it was already withdrawn")]
    UserAlreadyWithdrawn(Uuid),

    #[error("Failed to find user {0}")]
    UserNotFound(Uuid),

    #[error("Failed to verify credential of user {0}")]
    InvalidCredential(Uuid),

    #[error("Failed to hashing a password: {0}")]
    HashingPasswordFailed(#[from] argon2::password_hash::Error),

    #[error("Failed to operate on the database: {0}")]
    DatabaseOperationFailed(#[source] anyhow::Error),

    #[error("Failed due to unknown or undefined errors: {0}")]
    Unexpected(#[source] anyhow::Error),
}
