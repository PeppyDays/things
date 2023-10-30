use event_sourcing::envelope::Envelope;
use event_sourcing::repository::interface::Repository;
use uuid::Uuid;

use crate::identity::errors::Error;
use crate::identity::models::Identity;
use crate::identity::models::Role;

#[derive(Debug)]
pub enum Command {
    RegisterIdentity { id: Uuid, role: Role },
}

#[derive(Clone)]
pub struct CommandExecutor<R: Repository<Identity>> {
    repository: R,
}

impl<R: Repository<Identity>> CommandExecutor<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    async fn find_events(&self, id: &Uuid) -> Result<Vec<Envelope<Identity>>, Error> {
        self.repository
            .find_all_events(&id)
            .await
            .map_err(|error| Error::Database {
                message: error.to_string(),
            })
    }

    // async fn load_aggregate(&self, id: &Uuid) -> Result<User, Error> {
    //     let events = self.find_events(&id).await?;
    //     Ok(User::load(events).await)
    // }

    async fn save_aggregate(&mut self, aggregate: &mut Identity) -> Result<(), Error> {
        self.repository
            .save(aggregate)
            .await
            .map_err(|error| Error::Database {
                message: error.to_string(),
            })
    }

    pub async fn execute(&mut self, command: Command) -> Result<(), Error> {
        match command {
            Command::RegisterIdentity { id, role } => {
                let events = self.find_events(&id).await?;
                if !events.is_empty() {
                    return Err(Error::AlreadyRegistered { id });
                }

                let mut identity = Identity::default();
                identity.register(id, role).await?;
                self.save_aggregate(&mut identity).await?;

                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use event_sourcing::repository::memory::MemoryRepository;

    use crate::identity::commands::*;
    use crate::identity::events::*;

    #[tokio::test]
    async fn register_identity_command_generates_identity_registered_event() {
        let repository = MemoryRepository::new();
        let mut command_executor = CommandExecutor::new(repository.clone());

        let id = Uuid::new_v4();
        let command = Command::RegisterIdentity {
            id,
            role: Role::Member,
        };

        command_executor.execute(command).await.unwrap();

        let envelopes: Vec<Envelope<Identity>> = repository.find_all_events(&id).await.unwrap();
        let envelope = envelopes.get(0).unwrap();
        assert_eq!(envelopes.len(), 1);
        assert_eq!(
            envelope.event,
            Event::IdentityRegistered {
                id,
                role: Role::Member,
            }
        )
    }
}
