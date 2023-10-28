use std::fmt::{Display, Formatter};
use uuid::Uuid;

#[derive(Debug)]
pub enum Error {
    AlreadyRegistered { id: Uuid },
    AlreadyWithdrawn { id: Uuid },
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::AlreadyRegistered { id } => write!(f, "User {id} is already registered"),
            Error::AlreadyWithdrawn { id } => write!(f, "User {id} is already withdrawn"),
        }
    }
}

impl std::error::Error for Error {}
