use crate::user::errors::Error;
use crate::user::models::User;
use event_sourcing::aggregate::EventSourced;
use event_sourcing::envelope::Envelope;
use event_sourcing::repository::interface::Repository;
use uuid::Uuid;

#[derive(Debug)]
pub enum Query {
    GetUser { id: Uuid },
}

#[derive(Clone)]
pub struct QueryReader<R: Repository<User>> {
    repository: R,
}

impl<R: Repository<User>> QueryReader<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    async fn find_events(&self, id: &Uuid) -> Result<Vec<Envelope<User>>, Error> {
        self.repository
            .find_all_events(&id)
            .await
            .map_err(|error| Error::Database {
                message: error.to_string(),
            })
    }

    async fn load_aggregate(&self, id: &Uuid) -> Result<User, Error> {
        let events = self.find_events(&id).await?;
        Ok(User::load(events).await)
    }

    pub async fn read(&self, query: Query) -> Result<User, Error> {
        match query {
            Query::GetUser { id } => {
                let user = self.load_aggregate(&id).await?;

                if user.is_withdrawn() {
                    return Err(Error::AlreadyWithdrawn { id: user.id });
                }
                Ok(user)
            }
        }
    }
}
