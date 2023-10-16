use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use uuid::Uuid;

use crate::{aggregate::EventSourced, event::Envelope};

use super::EventStore;

#[derive(Debug, Clone)]
pub struct MemoryRepository<A>
    where
        A: EventSourced + Send + Sync,
{
    events: Arc<RwLock<HashMap<Uuid, Vec<Envelope<A::Event>>>>>,
}

impl<A> MemoryRepository<A>
    where
        A: EventSourced + Send + Sync,
{
    pub fn new() -> Self {
        Self {
            events: Arc::default(),
        }
    }
}

#[async_trait]
impl<A> EventStore<A> for MemoryRepository<A>
    where
        A: EventSourced + Send + Sync,
{
    async fn save(&mut self, aggregate: &mut A) -> Result<(), String> {
        let mut store = self.events.write().map_err(|_| {
            String::from("Error happened while locking in-memory database for write. Try again.")
        })?;

        store
            .entry(aggregate.get_id().clone())
            .or_insert_with(|| Vec::new())
            .append(&mut aggregate.drain_pending_events());

        Ok(())
    }

    async fn find(&self, id: &Uuid) -> Result<Vec<Envelope<A::Event>>, String> {
        let store = self.events.read().map_err(|_| {
            String::from("Error happened while locking in-memory database for read. Try again.")
        })?;

        match store.get(id) {
            Some(events) => {
                if events.len() <= 0 {
                    return Err(String::from("No events found"));
                }

                Ok(events.clone())
            }
            None => Err(String::from("No entry found")),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::aggregate::*;
    use crate::event::*;
    use crate::repository::memory::*;
    use crate::test::*;

    #[tokio::test]
    async fn repository_saves_aggregate_without_moving_ownership() {
        let mut user = User::default();
        let id = Uuid::new_v4();
        let events = vec![
            UserEvent::UserRegistered { id },
            UserEvent::UserModified {
                name: String::from("Arine"),
            },
        ];
        user.update(events[0].clone()).await;
        user.update(events[1].clone()).await;

        let mut repository = MemoryRepository::<User>::new();
        repository.save(&mut user).await.unwrap();

        assert_eq!(user.get_id(), id);
        assert_eq!(user.get_sequence(), 2);
    }

    #[tokio::test]
    async fn after_repository_saves_aggregate_pending_events_are_empty() {
        let mut user = User::default();
        let id = Uuid::new_v4();
        let events = vec![
            UserEvent::UserRegistered { id },
            UserEvent::UserModified {
                name: String::from("Arine"),
            },
        ];
        user.update(events[0].clone()).await;
        user.update(events[1].clone()).await;

        let mut repository = MemoryRepository::<User>::new();
        repository.save(&mut user).await.unwrap();

        assert_eq!(user.get_pending_events().len(), 0);
    }

    #[tokio::test]
    async fn repository_returns_ok_when_saving_aggregate_with_no_pending_events() {
        let mut user = User::default();
        let mut repository = MemoryRepository::<User>::new();

        assert_eq!(user.get_pending_events().len(), 0);
        let response = repository.save(&mut user).await;

        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn repository_find_returns_events_after_saved() {
        let mut user = User::default();
        let id = Uuid::new_v4();
        let events = vec![
            UserEvent::UserRegistered { id },
            UserEvent::UserModified {
                name: String::from("Arine"),
            },
        ];
        user.update(events[0].clone()).await;
        user.update(events[1].clone()).await;

        let mut repository = MemoryRepository::<User>::new();
        repository.save(&mut user).await.unwrap();

        let envelopes = repository.find(&id).await.unwrap();
        assert_eq!(envelopes.len(), 2);

        let event_1 = envelopes.get(0).unwrap().get_event();
        let event_2 = envelopes.get(1).unwrap().get_event();

        assert_eq!(event_1.get_name(), "UserRegistered");
        assert_eq!(event_2.get_name(), "UserModified");
    }
}
