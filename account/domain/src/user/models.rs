use async_trait::async_trait;
use event_sourcing::aggregate::EventSourced;
use event_sourcing::envelope::Envelope;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::user::errors::Error;
use crate::user::events::Event;

#[derive(Default, Serialize, Deserialize, Debug, PartialEq)]
pub struct User {
    id: Uuid,
    sequence: i64,
    name: String,
    age: u8,
    status: Status,
    pending_events: Vec<Envelope<Self>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum Status {
    #[default]
    Registered,
    Active,
    Withdrawn,
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

    async fn apply(&mut self, event: Self::Event) {
        match event {
            Event::UserRegistered { id } => {
                self.id = id;
                self.status = Status::Active;
            }
            Event::UserModified { name, age } => {
                self.name = name;
                self.age = age;
            }
            Event::UserWithdrawn => {
                self.status = Status::Withdrawn;
            }
        }
    }
}

impl User {
    pub async fn register(&mut self, id: Uuid) -> Result<(), Error> {
        let event = Event::UserRegistered { id };
        self.update(event).await;
        Ok(())
    }

    pub async fn modify(&mut self, name: String, age: u8) -> Result<(), Error> {
        let event = Event::UserModified { name, age };
        self.update(event).await;
        Ok(())
    }

    pub async fn withdraw(&mut self) -> Result<(), Error> {
        if self.status == Status::Withdrawn {
            return Err(Error::AlreadyRegistered { id: self.id });
        }

        let event = Event::UserWithdrawn;
        self.update(event).await;
        Ok(())
    }
}
