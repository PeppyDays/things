use async_trait::async_trait;
use event_sourcing::{aggregate::EventSourced, envelope::Envelope};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::identity::errors::Error;
use crate::identity::events::Event;

#[derive(Default, Serialize, Deserialize, Debug, PartialEq)]
pub struct Identity {
    pub id: Uuid,
    pub role: Role,
    pub refresh_token: Option<RefreshToken>,
    sequence: i64,
    pending_events: Vec<Envelope<Self>>,
}

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum Role {
    #[default]
    Member,
    Administrator,
}

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct RefreshToken {
    pub token: Uuid,
    pub expiry: u128,
}

#[async_trait]
impl EventSourced for Identity {
    type Event = Event;
    type Error = Error;

    fn get_name() -> String {
        String::from("Identity")
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

impl Identity {
    pub async fn register(&mut self, id: Uuid, role: Role) -> Result<(), Error> {
        let event = Event::IdentityRegistered { id, role };
        self.update(event).await;
        Ok(())
    }
}
