use event_sourcing::event::DomainEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Event {
    UserRegistered { id: Uuid },
    UserModified { name: String, age: u8 },
    UserWithdrawn,
}

impl DomainEvent for Event {
    fn get_name(&self) -> String {
        match self {
            Event::UserRegistered { .. } => String::from("UserRegistered"),
            Event::UserModified { .. } => String::from("UserModified"),
            Event::UserWithdrawn => String::from("UserWithdrawn"),
        }
    }
    fn get_version(&self) -> String {
        match self {
            Event::UserRegistered { .. } => String::from("1.0.0"),
            Event::UserModified { .. } => String::from("1.0.0"),
            Event::UserWithdrawn => String::from("1.0.0"),
        }
    }
}
