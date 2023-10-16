use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use uuid::Uuid;

use crate::event::{DomainEvent, Envelope};

#[async_trait]
pub trait EventSourced: Default + Sync + Send + Clone {
    type Event: DomainEvent + Clone;
    type Error: std::error::Error;

    fn get_name() -> String;

    fn get_id(&self) -> Uuid;

    fn get_sequence(&self) -> u32;
    fn set_sequence(&mut self, seq: u32);
    fn increase_sequence(&mut self) {
        self.set_sequence(self.get_sequence() + 1);
    }

    fn get_pending_events(&self) -> &Vec<Envelope<Self::Event>>;
    fn get_mut_pending_events(&mut self) -> &mut Vec<Envelope<Self::Event>>;
    fn add_pending_event(&mut self, event: Envelope<Self::Event>);
    fn drain_pending_events(&mut self) -> Vec<Envelope<Self::Event>> {
        self.get_mut_pending_events().drain(..).collect()
    }

    async fn apply(&mut self, event: Self::Event);

    async fn update(&mut self, event: Self::Event) {
        self.increase_sequence();
        self.apply(event.clone()).await;
        let pending_event = Envelope::new(
            Self::get_name(),
            self.get_id(),
            self.get_sequence(),
            Arc::new(event),
            HashMap::new(),
        );
        self.add_pending_event(pending_event);
    }

    async fn load(events: &Vec<Envelope<Self::Event>>) -> Self {
        // async is not permitted inside anonymous block for now ..
        // events.iter().fold(Self::default(), |mut aggregate, event| {
        //     aggregate.apply(event.get_event().clone()).await;
        //     aggregate.set_sequence(event.aggregate_sequence);
        //     aggregate
        // })
        let mut aggregate = Self::default();
        for event in events {
            aggregate.apply(event.get_event().clone()).await;
            aggregate.set_sequence(event.get_aggregate_sequence());
        }
        aggregate
    }
}

#[cfg(test)]
mod tests {
    use crate::aggregate::*;
    use crate::event::*;
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
                .get_event()
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
            Envelope::new(
                User::get_name(),
                id,
                1,
                Arc::new(UserEvent::UserRegistered { id }),
                HashMap::new(),
            ),
            Envelope::new(
                User::get_name(),
                id,
                2,
                Arc::new(UserEvent::UserModified {
                    name: String::from("Arine"),
                }),
                HashMap::new(),
            ),
        ];

        let user = User::load(&events).await;

        assert_eq!(user.get_id(), id);
        assert_eq!(user.get_sequence(), 2);
    }
}
