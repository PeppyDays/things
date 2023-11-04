use uuid::Uuid;

use crate::identity::errors::Error;
use crate::identity::models::{AccessToken, Identity, Role, User};
use crate::identity::repositories::Repository;

#[derive(Clone)]
pub enum Command {
    RegisterIdentity { id: Uuid, role: String },
    IssueAccessToken { id: Uuid, role: String },
}

#[derive(Clone)]
pub struct CommandExecutor<R: Repository> {
    repository: R,
}

impl<R: Repository> CommandExecutor<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&mut self, command: Command) -> Result<Option<AccessToken>, Error> {
        match command {
            Command::RegisterIdentity { id, role } => {
                let user = User {
                    id,
                    role: self.convert_to_role(&role)?,
                };
                let identity = self.repository.find_by_user(&user).await?;
                if let Some(_) = identity {
                    return Err(Error::AlreadyRegistered { user });
                }

                let identity = Identity::new(user).await;
                self.repository.save(identity).await?;

                Ok(None)
            }
            Command::IssueAccessToken { id, role } => {
                let user = User {
                    id,
                    role: self.convert_to_role(&role)?,
                };
                let identity = self.repository.find_by_user(&user).await?;

                match identity {
                    Some(mut identity) => {
                        let access_token = identity.issue_access_token().await?;
                        self.repository.save(identity).await?;
                        Ok(Some(access_token))
                    }
                    None => Err(Error::NotFound { user }),
                }
            }
        }
    }

    fn convert_to_role(&self, role: &str) -> Result<Role, Error> {
        match role {
            "Member" => Ok(Role::Member),
            "Administrator" => Ok(Role::Administrator),
            _ => Err(Error::InvalidRole { role: String::from(role) }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::identity::commands::*;
    use crate::identity::models::*;
    use crate::identity::repositories::*;

    #[tokio::test]
    async fn identity_registration_stores_user_information_only() {
        let repository = MemoryRepository::new();
        let mut command_executor = CommandExecutor::new(repository.clone());

        let id = Uuid::new_v4();
        let command = Command::RegisterIdentity {
            id,
            role: String::from("Member"),
        };

        command_executor.execute(command).await.unwrap();

        let identity = repository.find_by_user(&User { id, role: Role::Member }).await.unwrap().unwrap();
        assert_eq!(identity.user, User { id, role: Role::Member });
        assert!(identity.refresh_token.is_none());
    }

    #[tokio::test]
    async fn identity_registration_fails_when_existing_entry_found() {
        let repository = MemoryRepository::new();
        let mut command_executor = CommandExecutor::new(repository.clone());

        let id = Uuid::new_v4();
        let command = Command::RegisterIdentity {
            id,
            role: String::from("Member"),
        };
        command_executor.execute(command.clone()).await.unwrap();

        let result = command_executor.execute(command).await;
        assert_eq!(result, Err(Error::AlreadyRegistered { user: User { id, role: Role::Member } }));
    }

    #[tokio::test]
    async fn issuing_access_token_returns_access_token_and_stores_refresh_token() {
        let repository = MemoryRepository::new();
        let mut command_executor = CommandExecutor::new(repository.clone());

        let id = Uuid::new_v4();
        let command = Command::RegisterIdentity {
            id,
            role: String::from("Member"),
        };
        command_executor.execute(command).await.unwrap();

        let command = Command::IssueAccessToken {
            id,
            role: String::from("Member"),
        };
        let access_token = command_executor.execute(command).await.unwrap();
        let refresh_token = repository.find_by_user(&User { id, role: Role::Member }).await.unwrap().unwrap().refresh_token;

        assert!(access_token.is_some());
        assert!(refresh_token.is_some());
    }

    #[tokio::test]
    async fn issuing_access_token_fails_when_identity_not_found() {
        let repository = MemoryRepository::new();
        let mut command_executor = CommandExecutor::new(repository.clone());

        let id = Uuid::new_v4();
        let command = Command::IssueAccessToken {
            id,
            role: String::from("Member"),
        };
        let result = command_executor.execute(command).await;
        assert_eq!(result, Err(Error::NotFound { user: User { id, role: Role::Member } }));
    }
}
