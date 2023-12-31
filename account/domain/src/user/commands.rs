use uuid::Uuid;

// use event_sourcing::aggregate::EventSourced;
use event_sourcing::envelope::Envelope;
use event_sourcing::repository::error::Error as RepositoryError;
use event_sourcing::repository::interface::Repository;

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
            .find_all_events(id)
            .await
            .map_err(|error| match error {
                RepositoryError::NotFound(id) => Error::UserNotFound(id),
                _ => Error::DatabaseOperationFailed(error.into()),
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
            .map_err(|error| Error::DatabaseOperationFailed(error.into()))
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
                let resulted_events = self.find_events(&id).await;

                match resulted_events {
                    Ok(_) => Err(Error::UserAlreadyRegistered(id)),
                    Err(error) => match error {
                        Error::UserNotFound(id) => {
                            let mut user = User::default();
                            user.register(id, name, password, email, language).await?;
                            self.save_aggregate(&mut user).await?;
                            Ok(())
                        }
                        _ => Err(Error::DatabaseOperationFailed(error.into())),
                    },
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use event_sourcing::aggregate::EventSourced;
    use event_sourcing::repository::memory::MemoryRepository;

    use crate::user::commands::*;

    #[tokio::test]
    async fn new_user_registration_succeeds() {
        let repository = MemoryRepository::default();
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
        assert_eq!(envelopes.len(), 1);

        let user = User::load(envelopes).await;

        assert_eq!(user.id, id);
        assert_eq!(user.name, "Arine");
        assert_eq!(user.email, "peppydays@gmail.com");
        assert_eq!(user.language, "en");
        assert!(user.verify_password("welcome"));
    }

    #[tokio::test]
    async fn user_registration_fails_if_already_registered() {
        let repository = MemoryRepository::default();
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

        assert!(matches!(error, Error::UserAlreadyRegistered(..)));
    }
}
