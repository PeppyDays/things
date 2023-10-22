use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::aggregate::EventSourced;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Envelope<A: EventSourced> {
    pub id: Uuid,
    pub aggregate_id: Uuid,
    pub aggregate_sequence: u64,
    pub event: A::Event,
    pub metadata: HashMap<String, String>,
}

impl<A: EventSourced> Envelope<A> {
    pub fn new(
        aggregate_id: Uuid,
        aggregate_sequence: u64,
        event: A::Event,
        metadata: HashMap<String, String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            aggregate_id,
            aggregate_sequence,
            event,
            metadata,
        }
    }
}
