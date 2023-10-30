use async_trait::async_trait;
use event_sourcing::aggregate::EventSourced;
use event_sourcing::envelope::Envelope;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use uuid::Uuid;

use crate::user::errors::Error;
use crate::user::events::Event;

#[derive(Default, Serialize, Deserialize, Debug, PartialEq)]
pub struct User {
    pub id: Uuid,
    pub password: String,
    pub name: String,
    pub email: String,
    pub language: String,
    pub role: Role,
    pub status: Status,
    sequence: i64,
    pending_events: Vec<Envelope<Self>>,
}

#[derive(Default, Serialize, Deserialize, Debug, PartialEq)]
pub enum Role {
    #[default]
    Member,
    Administrator,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum Status {
    #[default]
    Registered,
    Active,
    Withdrawn,
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Registered => write!(f, "Registered"),
            Status::Active => write!(f, "Active"),
            Status::Withdrawn => write!(f, "Withdrawn"),
        }
    }
}

#[async_trait]
impl EventSourced for User {
    type Event = Event;
    type Error = Error;

    fn get_name() -> String {
        String::from("User")
    }
    fn get_id(&self) -> Uuid {
        self.id
    }
    fn get_sequence(&self) -> i64 {
        self.sequence
    }
    fn set_sequence(&mut self, seq: i64) {
        self.sequence = seq
    }
    fn get_pending_events(&self) -> &Vec<Envelope<Self>> {
        &self.pending_events
    }
    fn get_mut_pending_events(&mut self) -> &mut Vec<Envelope<Self>> {
        &mut self.pending_events
    }
    fn add_pending_event(&mut self, event: Envelope<Self>) {
        self.pending_events.push(event)
    }
}

impl User {
    pub async fn register(
        &mut self,
        id: Uuid,
        name: String,
        password: String,
        email: String,
        language: String,
    ) -> Result<(), Error> {
        let event = Event::UserRegistered {
            id,
            name,
            password,
            email,
            language,
        };
        self.update(event).await;
        Ok(())
    }

    pub fn is_withdrawn(&self) -> bool {
        self.status == Status::Withdrawn
    }
}
