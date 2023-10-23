use std::fmt::Debug;

use serde::de::DeserializeOwned;
use serde::Serialize;

pub trait DomainEvent:
    Debug + Serialize + DeserializeOwned + Clone + PartialEq + Sync + Send
{
    fn get_name(&self) -> String;
    fn get_version(&self) -> String;
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::str::FromStr;

    use uuid::Uuid;

    use crate::envelope::Envelope;
    use crate::test::*;

    #[test]
    fn event_can_be_enveloped() {
        let event = UserEvent::UserRegistered { id: Uuid::new_v4() };

        let envelope = Envelope::<User>::new(Uuid::new_v4(), 1, event.clone(), HashMap::new());

        assert_eq!(envelope.event, event);
    }

    #[test]
    fn event_can_be_serialized_as_string() {
        let event = UserEvent::UserRegistered {
            id: Uuid::from_str("8f9e3f50-f662-461a-9048-48d55ceb829d").unwrap(),
        };
        let expected = "{\"UserRegistered\":{\"id\":\"8f9e3f50-f662-461a-9048-48d55ceb829d\"}}";

        let serialized = serde_json::to_string(&event).unwrap();

        assert_eq!(serialized, expected);
    }

    #[test]
    fn event_can_be_serialized_and_deserialized_as_value() {
        let event = UserEvent::UserRegistered {
            id: Uuid::from_str("8f9e3f50-f662-461a-9048-48d55ceb829d").unwrap(),
        };

        let serialized = serde_json::to_value(&event).unwrap();
        let deserialized: UserEvent = serde_json::from_value(serialized).unwrap();

        assert_eq!(event, deserialized);
    }
}
