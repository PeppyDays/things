use std::fmt::{Display, Formatter};

use crate::identity::models::User;

#[derive(Debug, PartialEq)]
pub enum Error {
    AlreadyRegistered { user: User },
    EntityNotFound { user: User },
    InvalidRole { role: String },
    TokenCreationFailed { message: String },
    TokenRefreshFailed { message: String },
    TokenValidationFailed { message: String },
    Database { message: String },
    Unknown,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::AlreadyRegistered { user } => {
                write!(f, "User {}'s identity is already registered", user.id)
            }
            Error::EntityNotFound { user } => write!(f, "User {}'s identity is not found", user.id),
            Error::InvalidRole { role } => write!(f, "Role {} is not defined", role),
            Error::TokenCreationFailed { message } => {
                write!(
                    f,
                    "Error happened during processing authentication token: {}",
                    message
                )
            }
            Error::TokenRefreshFailed { message } => {
                write!(f, "Token cannot be refreshed: {}", message)
            }
            Error::TokenValidationFailed { message } => {
                write!(f, "Token cannot be validated: {}", message)
            }
            Error::Database { message } => write!(
                f,
                "Error happened during interacting with database: {}",
                message
            ),
            Error::Unknown => write!(f, "Unknown error"),
        }
    }
}

impl std::error::Error for Error {}
