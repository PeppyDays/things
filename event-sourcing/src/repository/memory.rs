use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use uuid::Uuid;

use crate::aggregate::EventSourced;
use crate::envelope::Envelope;
use crate::repository::error::Error;
use crate::repository::interface::Repository;
use crate::repository::serialization::SerializedEnvelope;

#[derive(Debug)]
pub struct MemoryRepository {
    rows: Arc<RwLock<HashMap<Uuid, Vec<SerializedEnvelope>>>>,
}

impl Clone for MemoryRepository {
    fn clone(&self) -> Self {
        Self {
            rows: Arc::clone(&self.rows),
        }
    }
}

impl MemoryRepository {
    pub fn new() -> Self {
        Self {
            rows: Arc::default(),
        }
    }
}

#[async_trait]
impl<A> Repository<A> for MemoryRepository
where
    A: EventSourced,
{
    async fn save(&mut self, aggregate: &mut A) -> Result<(), Error> {
        let mut store = self.rows.write().map_err(|_| Error::Unknown)?;

        let mut serialized_events = aggregate
            .drain_pending_events()
            .into_iter()
            .map(|envelope| SerializedEnvelope::try_from(envelope).map_err(|_| Error::Unknown))
            .collect::<Result<Vec<SerializedEnvelope>, Error>>()?;

        store
            .entry(aggregate.get_id().clone())
            .or_insert_with(|| Vec::new())
            .append(&mut serialized_events);

        Ok(())
    }

    async fn find_all_events(&self, aggregate_id: &Uuid) -> Result<Vec<Envelope<A>>, Error> {
        let store = self.rows.read().map_err(|_| Error::Unknown)?;

        match store.get(aggregate_id) {
            Some(events) => match events.is_empty() {
                true => Err(Error::NotFound(aggregate_id.clone())),
                false => Ok(events
                    .clone()
                    .into_iter()
                    .map(|event| Envelope::try_from(event).map_err(|_| Error::Unknown))
                    .collect::<Result<Vec<Envelope<A>>, Error>>()?),
            },
            None => Err(Error::NotFound(aggregate_id.clone())),
        }
    }
}

#[cfg(test)]
mod test {
    use uuid::Uuid;

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

        let mut repository = MemoryRepository::new();
        repository.save(&mut user).await.unwrap();

        assert_eq!(user.get_id(), id);
        assert_eq!(user.get_sequence(), 2);
    }

    #[tokio::test]
    async fn repository_returns_not_found_error_vector_when_events_not_exist() {
        let id = Uuid::default();
        let repository = MemoryRepository::new();

        let result: Result<Vec<Envelope<User>>, Error> = repository.find_all_events(&id).await;
        let error = result.err().unwrap();
        assert_eq!(error.to_string(), format!("No entity found with ID {}", id));
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

        let mut repository = MemoryRepository::new();
        repository.save(&mut user).await.unwrap();

        assert_eq!(user.get_pending_events().len(), 0);
    }

    #[tokio::test]
    async fn repository_returns_ok_when_saving_aggregate_with_no_pending_events() {
        let mut user = User::default();
        let mut repository = MemoryRepository::new();

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

        let mut repository = MemoryRepository::new();
        repository.save(&mut user).await.unwrap();

        let envelopes: Vec<Envelope<User>> = repository.find_all_events(&id).await.unwrap();
        assert_eq!(envelopes.len(), 2);

        let event_1 = &envelopes.get(0).unwrap().event;
        let event_2 = &envelopes.get(1).unwrap().event;

        assert_eq!(event_1.get_name(), "UserRegistered");
        assert_eq!(event_2.get_name(), "UserModified");
    }
}
