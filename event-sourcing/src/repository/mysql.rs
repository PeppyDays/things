use async_trait::async_trait;
use sqlx::{MySql, Pool, Row};
use uuid::Uuid;

use crate::aggregate::EventSourced;
use crate::envelope::Envelope;
use crate::repository::interface::Repository;
use crate::repository::serialization::SerializedEnvelope;

const DEFAULT_EVENT_TABLE: &str = "events";

#[derive(Debug)]
pub struct MySqlRepository {
    pool: Pool<MySql>,
}

impl MySqlRepository {
    pub fn new(pool: Pool<MySql>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<A: EventSourced> Repository<A> for MySqlRepository {
    async fn save(&mut self, aggregate: &mut A) -> Result<(), String> {
        let query = format!("INSERT INTO {DEFAULT_EVENT_TABLE} (id, aggregate_name, aggregate_id, aggregate_sequence, event_name, event_version, event_payload, metadata) VALUES (?, ?, ?, ?, ?, ?, ?, ?)");

        let events = aggregate
            .drain_pending_events()
            .into_iter()
            .map(|envelope| SerializedEnvelope::try_from(envelope))
            .collect::<Result<Vec<SerializedEnvelope>, String>>()?;

        let mut tx = sqlx::Acquire::begin(&self.pool).await.map_err(|_| {
            String::from("Error happened while acquiring connection from pool. Try again.")
        })?;

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
                .map_err(|_| {
                    String::from("Error happened while inserting pending events. Try again.")
                })?;
        }

        tx.commit().await.map_err(|_| {
            String::from("Error happened while committing transaction. Try again.")
        })?;
        Ok(())
    }

    async fn find_all_events(&self, aggregate_id: &Uuid) -> Result<Vec<Envelope<A>>, String> {
        let query = format!("SELECT id, aggregate_name, aggregate_id, aggregate_sequence, event_name, event_version, event_payload, metadata FROM {DEFAULT_EVENT_TABLE} WHERE aggregate_name = ? AND aggregate_id = ? ORDER BY aggregate_sequence ASC");

        sqlx::query(&query)
            .bind(A::get_name())
            .bind(aggregate_id)
            .map(|row| {
                SerializedEnvelope {
                    id: row.get("id"),
                    aggregate_name: row.get("aggregate_name"),
                    aggregate_id: row.get("aggregate_id"),
                    aggregate_sequence: row.get("aggregate_sequence"),
                    event_name: row.get("event_name"),
                    event_version: row.get("event_version"),
                    event_payload: row.get("event_payload"),
                    metadata: row.get("metadata"),
                }
            })
            .fetch_all(&self.pool)
            .await
            .map_err(|_| {
                String::from("Error happened while fetching events from database. Try again.")
            })?
            .into_iter()
            .map(|event| Envelope::<A>::try_from(event))
            .collect::<Result<Vec<Envelope<A>>, String>>()
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::aggregate::*;
    use crate::repository::mysql::*;
    use crate::test::*;

    #[tokio::test]
    async fn mysql_repository_can_save_domain_events() {
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

        let mut repository = MySqlRepository::new(
            sqlx::Pool::connect("mysql://root:welcome@localhost:3306/es")
                .await
                .unwrap(),
        );
        repository.save(&mut user).await.unwrap();
    }

    #[tokio::test]
    async fn mysql_repository_can_save_and_find_all_domain_events() {
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

        let mut repository = MySqlRepository::new(
            sqlx::Pool::connect("mysql://root:welcome@localhost:3306/es")
                .await
                .unwrap(),
        );

        repository.save(&mut user).await.unwrap();
        let loaded_events: Vec<Envelope<User>> = repository.find_all_events(&aggregate_id).await.unwrap();

        assert_eq!(loaded_events.len(), 2);
        assert_eq!(loaded_events[0].event, events[0]);
        assert_eq!(loaded_events[1].event, events[1]);
    }
}
