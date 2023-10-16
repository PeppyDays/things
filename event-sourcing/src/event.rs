use std::{collections::HashMap, fmt::Debug, sync::Arc};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub trait DomainEvent: Debug + Sync + Send {
    fn get_name(&self) -> String;
    fn get_version(&self) -> String;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Envelope<T: DomainEvent + Clone> {
    id: Uuid,
    aggregate_name: String,
    aggregate_id: Uuid,
    aggregate_sequence: u32,
    event_name: String,
    event_version: String,
    event_payload: Arc<T>,
    metadata: HashMap<String, String>,
}

impl<T: DomainEvent + Clone> Envelope<T> {
    pub fn new(
        aggregate_name: String,
        aggregate_id: Uuid,
        aggregate_sequence: u32,
        event: Arc<T>,
        metadata: HashMap<String, String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            aggregate_name,
            aggregate_id,
            aggregate_sequence,
            event_name: event.get_name(),
            event_version: event.get_version(),
            event_payload: event,
            metadata,
        }
    }

    pub fn get_event(&self) -> &T {
        &*self.event_payload
    }

    pub fn get_aggregate_sequence(&self) -> u32 {
        self.aggregate_sequence
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::event::*;
    use crate::test::*;

    #[test]
    fn event_can_be_enveloped() {
        let event = OrderEvent::OrderPlaced { id: Uuid::new_v4() };

        let envelope = Envelope::new(
            String::from("Order"),
            Uuid::new_v4(),
            1,
            Arc::new(event.clone()),
            HashMap::new(),
        );

        assert_eq!(envelope.get_event(), &event);
    }

    #[test]
    fn event_can_be_serialized() {
        let event = OrderEvent::OrderPlaced {
            id: Uuid::from_str("8f9e3f50-f662-461a-9048-48d55ceb829d").unwrap(),
        };
        let expected = "{\"OrderPlaced\":{\"id\":\"8f9e3f50-f662-461a-9048-48d55ceb829d\"}}";

        let serialized = serde_json::to_string(&event).unwrap();

        assert_eq!(serialized, expected);
    }

    #[test]
    fn event_can_be_serialized_and_deserialized() {
        let event = OrderEvent::OrderPlaced {
            id: Uuid::from_str("8f9e3f50-f662-461a-9048-48d55ceb829d").unwrap(),
        };
        let serialized = serde_json::to_string(&event).unwrap();

        let deserialized: OrderEvent = serde_json::from_str(&serialized).unwrap();

        assert_eq!(event, deserialized);
    }

    #[test]
    fn envelope_can_be_serialized_and_deserialized() {
        let event = OrderEvent::OrderCompleted;
        let envelope = Envelope::new(
            String::from("Order"),
            Uuid::new_v4(),
            1,
            Arc::new(event.clone()),
            HashMap::new(),
        );

        let serialized = serde_json::to_string(&envelope).unwrap();
        let deserialized = serde_json::from_str(&serialized).unwrap();

        assert_eq!(envelope, deserialized);
    }
}
