use async_trait::async_trait;
use uuid::Uuid;

use crate::aggregate::EventSourced;
use crate::envelope::Envelope;

#[async_trait]
pub trait Repository<A: EventSourced> {
    async fn save(&mut self, aggregate: &mut A) -> Result<(), String>;
    async fn find_all_events(&self, aggregate_id: &Uuid) -> Result<Vec<Envelope<A>>, String>;
}
