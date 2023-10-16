use async_trait::async_trait;
use uuid::Uuid;

use crate::{aggregate::EventSourced, event::Envelope};

mod memory;

#[async_trait]
trait EventStore<A: EventSourced> {
    async fn save(&mut self, aggregate: &mut A) -> Result<(), String>;
    async fn find(&self, id: &Uuid) -> Result<Vec<Envelope<A::Event>>, String>;
}
