use std::fmt::{Display, Formatter};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::aggregate::*;
use crate::event::*;

#[derive(Default, Clone, Debug)]
pub struct User {
    id: Uuid,
    sequence: u32,
    name: String,
    pending_events: Vec<Envelope<<User as EventSourced>::Event>>,
}

#[async_trait]
impl EventSourced for User {
    type Event = UserEvent;
    type Error = UserError;

    fn get_name() -> String {
        String::from("User")
    }
    fn get_id(&self) -> Uuid {
        self.id
    }
    fn get_sequence(&self) -> u32 {
        self.sequence
    }
    fn set_sequence(&mut self, seq: u32) {
        self.sequence = seq
    }
    fn get_pending_events(&self) -> &Vec<Envelope<Self::Event>> {
        &self.pending_events
    }
    fn get_mut_pending_events(&mut self) -> &mut Vec<Envelope<Self::Event>> {
        &mut self.pending_events
    }
    fn add_pending_event(&mut self, event: Envelope<Self::Event>) {
        self.pending_events.push(event)
    }
    async fn apply(&mut self, event: Self::Event) {
        match event {
            UserEvent::UserRegistered { id } => {
                self.id = id;
            }
            UserEvent::UserModified { name } => {
                self.name = name;
            }
        }
    }
}

#[derive(Debug)]
pub enum UserError {}

impl Display for UserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "UserError")
    }
}

impl std::error::Error for UserError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserEvent {
    UserRegistered { id: Uuid },
    UserModified { name: String },
}

impl DomainEvent for UserEvent {
    fn get_name(&self) -> String {
        match self {
            UserEvent::UserRegistered { .. } => String::from("UserRegistered"),
            UserEvent::UserModified { .. } => String::from("UserModified"),
        }
    }
    fn get_version(&self) -> String {
        match self {
            UserEvent::UserRegistered { .. } => String::from("1.0.0"),
            UserEvent::UserModified { .. } => String::from("1.0.0"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrderEvent {
    OrderPlaced { id: Uuid },
    OrderCompleted,
}

impl DomainEvent for OrderEvent {
    fn get_name(&self) -> String {
        match self {
            OrderEvent::OrderPlaced { .. } => String::from("OrderPlaced"),
            OrderEvent::OrderCompleted => String::from("OrderCompleted"),
        }
    }
    fn get_version(&self) -> String {
        match self {
            OrderEvent::OrderPlaced { .. } => String::from("1.0.0"),
            OrderEvent::OrderCompleted => String::from("1.0.0"),
        }
    }
}
