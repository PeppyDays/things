// use event_sourcing::aggregate::EventSourced;
use event_sourcing::envelope::Envelope;
use event_sourcing::repository::interface::Repository;
use uuid::Uuid;

use crate::user::errors::Error;
use crate::user::models::User;

#[derive(Debug)]
pub enum Command {
    RegisterUser {
        id: Uuid,
        name: String,
        password: String,
        email: String,
        language: String,
    },
}

#[derive(Clone)]
pub struct CommandExecutor<R: Repository<User>> {
    repository: R,
}

impl<R: Repository<User>> CommandExecutor<R> {
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

    // async fn load_aggregate(&self, id: &Uuid) -> Result<User, Error> {
    //     let events = self.find_events(&id).await?;
    //     Ok(User::load(events).await)
    // }

    async fn save_aggregate(&mut self, aggregate: &mut User) -> Result<(), Error> {
        self.repository
            .save(aggregate)
            .await
            .map_err(|error| Error::Database {
                message: error.to_string(),
            })
    }

    pub async fn execute(&mut self, command: Command) -> Result<(), Error> {
        match command {
            Command::RegisterUser {
                id,
                name,
                password,
                email,
                language,
            } => {
                let events = self.find_events(&id).await?;
                if !events.is_empty() {
                    return Err(Error::AlreadyRegistered { id });
                }

                let mut user = User::default();
                user.register(id, name, password, email, language).await?;
                self.save_aggregate(&mut user).await?;

                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use event_sourcing::repository::memory::MemoryRepository;

    use crate::user::commands::*;
    use crate::user::events::*;

    #[tokio::test]
    async fn register_user_command_generates_user_generated_event() {
        let repository = MemoryRepository::new();
        let mut command_executor = CommandExecutor::new(repository.clone());

        let id = Uuid::new_v4();
        let command = Command::RegisterUser {
            id,
            name: String::from("Arine"),
            password: String::from("welcome"),
            email: String::from("peppydays@gmail.com"),
            language: String::from("en"),
        };

        command_executor.execute(command).await.unwrap();

        let envelopes: Vec<Envelope<User>> = repository.find_all_events(&id).await.unwrap();
        let envelope = envelopes.get(0).unwrap();
        assert_eq!(envelopes.len(), 1);
        assert_eq!(
            envelope.event,
            Event::UserRegistered {
                id,
                name: String::from("Arine"),
                password: String::from("welcome"),
                email: String::from("peppydays@gmail.com"),
                language: String::from("en")
            }
        )
    }

    #[tokio::test]
    async fn user_registration_fails_if_already_registered() {
        let repository = MemoryRepository::new();
        let mut command_executor = CommandExecutor::new(repository);

        let id = Uuid::new_v4();
        let command = Command::RegisterUser {
            id,
            name: String::from("Arine"),
            password: String::from("welcome"),
            email: String::from("peppydays@gmail.com"),
            language: String::from("en"),
        };
        command_executor.execute(command).await.unwrap();

        let command = Command::RegisterUser {
            id,
            name: String::from("Ailee"),
            password: String::from("welcome"),
            email: String::from("ailee.koh@healingpaper.com"),
            language: String::from("en"),
        };
        let error = command_executor.execute(command).await.err().unwrap();

        assert_eq!(error, Error::AlreadyRegistered { id })
    }
}
