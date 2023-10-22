use std::collections::HashMap;

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use uuid::Uuid;

use crate::envelope::Envelope;
use crate::event::DomainEvent;

#[async_trait]
pub trait EventSourced: Default + Serialize + DeserializeOwned + Send + Sync {
    type Event: DomainEvent;
    type Error: std::error::Error;

    fn get_name() -> String;

    fn get_id(&self) -> Uuid;

    fn get_sequence(&self) -> i64;
    fn set_sequence(&mut self, seq: i64);
    fn increase_sequence(&mut self) {
        self.set_sequence(self.get_sequence() + 1);
    }

    fn get_pending_events(&self) -> &Vec<Envelope<Self>>;
    fn get_mut_pending_events(&mut self) -> &mut Vec<Envelope<Self>>;
    fn add_pending_event(&mut self, event: Envelope<Self>);
    fn drain_pending_events(&mut self) -> Vec<Envelope<Self>> {
        self.get_mut_pending_events().drain(..).collect()
    }

    async fn apply(&mut self, event: Self::Event);

    async fn update(&mut self, event: Self::Event) {
        self.increase_sequence();
        self.apply(event.clone()).await;
        let pending_event = Envelope::new(
            self.get_id(),
            self.get_sequence(),
            event,
            HashMap::new(),
        );
        self.add_pending_event(pending_event);
    }

    async fn load(enveloped_events: Vec<Envelope<Self>>) -> Self {
        // async is not permitted inside anonymous block for now, so fold cannot be used
        let mut aggregate = Self::default();
        for enveloped_event in enveloped_events {
            aggregate.apply(enveloped_event.event).await;
            aggregate.set_sequence(enveloped_event.aggregate_sequence);
        }
        aggregate
    }
}

#[cfg(test)]
mod tests {
    use crate::aggregate::*;
    use crate::test::*;

    #[tokio::test]
    async fn aggregate_supports_default_instantiation() {
        let user = User::default();

        assert_eq!(user.get_id(), Uuid::default());
    }

    #[tokio::test]
    async fn aggregate_stores_domain_events_when_modified() {
        let mut user = User::default();
        let event = UserEvent::UserRegistered { id: Uuid::new_v4() };

        user.update(event).await;

        assert_eq!(user.get_pending_events().len(), 1);
        assert_eq!(
            user.get_pending_events()
                .first()
                .unwrap()
                .event
                .get_name(),
            "UserRegistered"
        );
    }

    #[tokio::test]
    async fn aggregate_can_drain_pending_events() {
        let mut user = User::default();
        let events = vec![
            UserEvent::UserRegistered { id: Uuid::new_v4() },
            UserEvent::UserModified {
                name: String::from("Arine"),
            },
        ];
        user.update(events[0].clone()).await;
        user.update(events[1].clone()).await;

        assert_eq!(user.get_pending_events().len(), 2);

        let drained_events = user.drain_pending_events();

        assert_eq!(user.get_pending_events().len(), 0);
        assert_eq!(drained_events.len(), 2);
    }

    #[tokio::test]
    async fn aggregate_can_be_loaded_from_events() {
        let id = Uuid::new_v4();
        let events = vec![
            Envelope::<User>::new(
                id,
                1,
                UserEvent::UserRegistered { id },
                HashMap::new(),
            ),
            Envelope::<User>::new(
                id,
                2,
                UserEvent::UserModified {
                    name: String::from("Arine"),
                },
                HashMap::new(),
            ),
        ];

        let user = User::load(events).await;

        assert_eq!(user.get_id(), id);
        assert_eq!(user.get_sequence(), 2);
    }
}
