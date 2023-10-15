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

    fn get_pending_events(&self) -> &[Envelope<Self::Event>];
    fn add_pending_event(&mut self, event: Envelope<Self::Event>);

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

    async fn load(events: &[Envelope<Self::Event>]) -> Self {
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
    use std::fmt::{Display, Formatter};

    use serde::{Deserialize, Serialize};

    use crate::aggregate::*;
    use crate::event::*;

    #[tokio::test]
    async fn aggregate_supports_default_instantiation() {
        let user = User::default();

        assert_eq!(user.id, Uuid::nil());
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

        assert_eq!(user.id, id);
        assert_eq!(user.sequence, 2);
        assert_eq!(user.name, String::from("Arine"));
    }

    #[derive(Default, Clone, Debug)]
    struct User {
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
        fn get_pending_events(&self) -> &[Envelope<Self::Event>] {
            &self.pending_events
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
    enum UserError {}

    impl Display for UserError {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "UserError")
        }
    }

    impl std::error::Error for UserError {}

    #[derive(Debug, Clone, Serialize, Deserialize)]
    enum UserEvent {
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
}
