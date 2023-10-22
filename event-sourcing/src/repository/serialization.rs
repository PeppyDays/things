use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::aggregate::EventSourced;
use crate::envelope::Envelope;
use crate::event::DomainEvent;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SerializedEnvelope {
    pub id: Uuid,
    pub aggregate_name: String,
    pub aggregate_id: Uuid,
    pub aggregate_sequence: u64,
    pub event_name: String,
    pub event_version: String,
    pub event_payload: Value,
    pub metadata: Value,
}

impl<A> TryFrom<Envelope<A>> for SerializedEnvelope
    where
        A: EventSourced,
{
    type Error = String;

    fn try_from(envelope: Envelope<A>) -> Result<Self, Self::Error> {
        Ok(Self {
            id: envelope.id,
            aggregate_name: A::get_name(),
            aggregate_id: envelope.aggregate_id,
            aggregate_sequence: envelope.aggregate_sequence,
            event_name: envelope.event.get_name(),
            event_version: envelope.event.get_version(),
            event_payload: serde_json::to_value(&envelope.event).map_err(|e| e.to_string())?,
            metadata: serde_json::to_value(&envelope.metadata).map_err(|e| e.to_string())?,
        })
    }
}

impl<A> TryFrom<SerializedEnvelope> for Envelope<A>
    where
        A: EventSourced,
{
    type Error = String;

    fn try_from(event: SerializedEnvelope) -> Result<Self, Self::Error> {
        Ok(Self {
            id: event.id,
            aggregate_id: event.aggregate_id,
            aggregate_sequence: event.aggregate_sequence,
            event: serde_json::from_value(event.event_payload).map_err(|e| e.to_string())?,
            metadata: serde_json::from_value(event.metadata).map_err(|e| e.to_string())?,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::repository::serialization::*;
    use crate::test::*;

    #[test]
    fn envelope_can_be_serialized() {
        let event = UserEvent::UserRegistered { id: Uuid::new_v4() };
        let envelope = Envelope::<User>::new(
            Uuid::new_v4(),
            1,
            event.clone(),
            HashMap::new(),
        );

        let serialized = SerializedEnvelope::try_from(envelope).unwrap();

        assert_eq!(serialized.aggregate_sequence, 1);
        assert_eq!(serialized.event_name, "UserRegistered");
    }

    #[test]
    fn envelope_can_serialized_and_deserialized() {
        let event_id = Uuid::new_v4();
        let aggregate_id = Uuid::new_v4();

        let event = UserEvent::UserRegistered { id: event_id };
        let envelope = Envelope::<User>::new(
            aggregate_id,
            1,
            event.clone(),
            HashMap::new(),
        );

        let serialized = SerializedEnvelope::try_from(envelope).unwrap();
        let deserialized = Envelope::<User>::try_from(serialized).unwrap();

        assert_eq!(deserialized.aggregate_id, aggregate_id);
        assert_eq!(deserialized.aggregate_sequence, 1);
        assert_eq!(deserialized.event, UserEvent::UserRegistered { id: event_id });
    }
}
