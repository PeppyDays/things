use async_trait::async_trait;
use sqlx::postgres::PgRow;
use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;

use crate::aggregate::EventSourced;
use crate::envelope::Envelope;
use crate::repository::error::Error;
use crate::repository::interface::Repository;
use crate::repository::serialization::SerializedEnvelope;

const DEFAULT_EVENT_TABLE: &str = "events";

#[derive(Debug, Clone)]
pub struct PostgresRepository {
    pool: Pool<Postgres>,
}

impl PostgresRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<A: EventSourced> Repository<A> for PostgresRepository {
    async fn save(&mut self, aggregate: &mut A) -> Result<(), Error> {
        let query = format!("INSERT INTO {DEFAULT_EVENT_TABLE} (id, aggregate_name, aggregate_id, aggregate_sequence, event_name, event_version, event_payload, metadata) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)");

        let events = aggregate
            .drain_pending_events()
            .into_iter()
            .map(|envelope| {
                SerializedEnvelope::try_from(envelope)
                    .map_err(|error| Error::Serialization(Box::new(error)))
            })
            .collect::<Result<Vec<SerializedEnvelope>, Error>>()?;

        let mut tx = sqlx::Acquire::begin(&self.pool)
            .await
            .map_err(|error| Error::Transaction(Box::new(error)))?;

        for event in events {
            sqlx::query(&query)
                .bind(event.id)
                .bind(event.aggregate_name)
                .bind(event.aggregate_id)
                .bind(event.aggregate_sequence)
                .bind(event.event_name)
                .bind(event.event_version)
                .bind(event.event_payload)
                .bind(event.metadata)
                .execute(&mut *tx)
                .await
                .map_err(|error| Error::Execution(Box::new(error)))?;
        }

        tx.commit()
            .await
            .map_err(|error| Error::Transaction(Box::new(error)))?;

        Ok(())
    }

    async fn find_all_events(&self, aggregate_id: &Uuid) -> Result<Vec<Envelope<A>>, Error> {
        let query = format!("SELECT id, aggregate_name, aggregate_id, aggregate_sequence, event_name, event_version, event_payload, metadata FROM {DEFAULT_EVENT_TABLE} WHERE aggregate_name = $1 AND aggregate_id = $2 ORDER BY aggregate_sequence ASC");

        let envelopes = sqlx::query(&query)
            .bind(A::get_name())
            .bind(aggregate_id)
            .map(|row: PgRow| SerializedEnvelope {
                id: row.get("id"),
                aggregate_name: row.get("aggregate_name"),
                aggregate_id: row.get("aggregate_id"),
                aggregate_sequence: row.get("aggregate_sequence"),
                event_name: row.get("event_name"),
                event_version: row.get("event_version"),
                event_payload: row.get("event_payload"),
                metadata: row.get("metadata"),
            })
            .fetch_all(&self.pool)
            .await
            .map_err(|error| Error::Execution(Box::new(error)))?
            .into_iter()
            .map(|event| {
                Envelope::<A>::try_from(event)
                    .map_err(|error| Error::Deserialization(Box::new(error)))
            })
            .collect::<Result<Vec<Envelope<A>>, Error>>()?;

        match envelopes.is_empty() {
            true => Err(Error::NotFound(aggregate_id.clone())),
            false => Ok(envelopes),
        }
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::aggregate::*;
    use crate::repository::postgresql::*;
    use crate::test::*;

    #[tokio::test]
    #[ignore]
    async fn postgresql_repository_can_save_domain_events() {
        let mut user = User::default();
        let aggregate_id = Uuid::new_v4();
        let events = vec![
            UserEvent::UserRegistered { id: aggregate_id },
            UserEvent::UserModified {
                name: String::from("Arine"),
            },
        ];
        user.update(events[0].clone()).await;
        user.update(events[1].clone()).await;

        let mut repository = PostgresRepository::new(
            sqlx::Pool::connect("postgresql://postgres:welcome@localhost:5432/es")
                .await
                .unwrap(),
        );
        repository.save(&mut user).await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn postgresql_repository_can_save_and_find_all_domain_events() {
        let mut user = User::default();
        let aggregate_id = Uuid::new_v4();
        let events = vec![
            UserEvent::UserRegistered { id: aggregate_id },
            UserEvent::UserModified {
                name: String::from("Arine"),
            },
        ];
        user.update(events[0].clone()).await;
        user.update(events[1].clone()).await;

        let mut repository = PostgresRepository::new(
            sqlx::Pool::connect("postgresql://postgres:welcome@localhost:5432/es")
                .await
                .unwrap(),
        );

        repository.save(&mut user).await.unwrap();
        let loaded_events: Vec<Envelope<User>> =
            repository.find_all_events(&aggregate_id).await.unwrap();

        assert_eq!(loaded_events.len(), 2);
        assert_eq!(loaded_events[0].event, events[0]);
        assert_eq!(loaded_events[1].event, events[1]);
    }
}
