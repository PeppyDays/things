use uuid::Uuid;

use event_sourcing::aggregate::EventSourced;
use event_sourcing::envelope::Envelope;
use event_sourcing::repository::error::Error as RepositoryError;
use event_sourcing::repository::interface::Repository;

use crate::user::errors::Error;
use crate::user::models::User;

#[derive(Debug)]
pub enum Query {
    GetUser { id: Uuid },
    VerifyCredential { id: Uuid, password: String },
}

#[derive(Clone)]
pub struct QueryReader<R: Repository<User>> {
    repository: R,
}

impl<R: Repository<User>> QueryReader<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    // async fn find_events(&self, id: &Uuid) -> Result<Vec<Envelope<User>>, Error> {
    //     self.repository
    //         .find_all_events(&id)
    //         .await
    //         .map_err(|error| Error::Database {
    //             message: error.to_string(),
    //         })
    // }

    async fn find_events(&self, id: &Uuid) -> Result<Vec<Envelope<User>>, Error> {
        self.repository
            .find_all_events(&id)
            .await
            .map_err(|error| match error {
                RepositoryError::NotFound(id) => Error::EntityNotFound { id },
                _ => Error::Database {
                    message: error.to_string(),
                },
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

                match user.is_withdrawn() {
                    true => Err(Error::AlreadyWithdrawn { id: user.id }),
                    false => Ok(user),
                }
            }
            Query::VerifyCredential { id, password } => {
                let user = self.load_aggregate(&id).await?;

                match user.verify_password(&password) {
                    true => Ok(user),
                    false => Err(Error::InvalidCredential),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use event_sourcing::repository::memory::MemoryRepository;

    use crate::user::{commands::*, queries::*};

    #[tokio::test]
    async fn verifying_with_correct_password_returns_aggregate_instance() {
        let repository = MemoryRepository::new();
        let mut command_executor = CommandExecutor::new(repository.clone());
        let query_reader = QueryReader::new(repository.clone());

        let id = Uuid::new_v4();
        let command = Command::RegisterUser {
            id,
            name: String::from("Arine"),
            password: String::from("welcome"),
            email: String::from("peppydays@gmail.com"),
            language: String::from("en"),
        };
        command_executor.execute(command).await.unwrap();

        let query = Query::VerifyCredential {
            id,
            password: String::from("welcome"),
        };

        assert!(query_reader.read(query).await.is_ok());
    }

    #[tokio::test]
    async fn verifying_with_incorrect_password_returns_error() {
        let repository = MemoryRepository::new();
        let mut command_executor = CommandExecutor::new(repository.clone());
        let query_reader = QueryReader::new(repository.clone());

        let id = Uuid::new_v4();
        let command = Command::RegisterUser {
            id,
            name: String::from("Arine"),
            password: String::from("welcome"),
            email: String::from("peppydays@gmail.com"),
            language: String::from("en"),
        };
        command_executor.execute(command).await.unwrap();

        let query = Query::VerifyCredential {
            id,
            password: String::from("thanks"),
        };

        assert_eq!(
            query_reader.read(query).await,
            Err(Error::InvalidCredential)
        );
    }
}
