use std::fmt::{Display, Formatter};
use uuid::Uuid;

#[derive(Debug, PartialEq)]
pub enum Error {
    AlreadyRegistered { id: Uuid },
    AlreadyWithdrawn { id: Uuid },
    Database { message: String },
    Unknown,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::AlreadyRegistered { id } => write!(f, "User {id} is already registered"),
            Error::AlreadyWithdrawn { id } => write!(f, "User {id} is already withdrawn"),
            Error::Database { message } => write!(
                f,
                "Error happened during interacting with database: {message}"
            ),
            Error::Unknown => write!(f, "Unknown error"),
        }
    }
}

impl std::error::Error for Error {}
