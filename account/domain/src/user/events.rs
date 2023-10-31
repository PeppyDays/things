use crate::user::models::{Status, User};
use async_trait::async_trait;
use event_sourcing::event::DomainEvent;
use event_sourcing::event::EventApplier;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::user::models::Role;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Event {
    UserRegistered {
        id: Uuid,
        name: String,
        password: String,
        email: String,
        language: String,
    },
}

impl DomainEvent for Event {
    fn get_name(&self) -> String {
        match self {
            Event::UserRegistered { .. } => String::from("UserRegistered"),
        }
    }
    fn get_version(&self) -> String {
        match self {
            Event::UserRegistered { .. } => String::from("1.0.0"),
        }
    }
}

#[async_trait]
impl EventApplier<User> for User {
    async fn apply(&mut self, event: Event) {
        match event {
            Event::UserRegistered {
                id,
                name,
                password,
                email,
                language,
            } => {
                self.id = id;
                self.name = name;
                self.password = password;
                self.email = email;
                self.language = language;
                self.role = Role::Member;
                self.status = Status::Active;
            }
        }
    }
}
