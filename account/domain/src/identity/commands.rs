use uuid::Uuid;

use crate::identity::errors::Error;
use crate::identity::models::RefreshToken;
use crate::identity::models::{Identity, Role, Tokens, User};
use crate::identity::repositories::Repository;

#[derive(Clone)]
pub enum Command {
    RegisterIdentity {
        id: Uuid,
        role: String,
    },
    IssueAccessToken {
        id: Uuid,
        role: String,
    },
    RefreshAccessToken {
        id: Uuid,
        role: String,
        refresh_token: String,
    },
    InvalidateRefreshToken {
        id: Uuid,
        role: Role,
    },
}

#[derive(Clone)]
pub struct CommandExecutor<R: Repository> {
    repository: R,
}

impl<R: Repository> CommandExecutor<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&mut self, command: Command) -> Result<Option<Tokens>, Error> {
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
                        let tokens = identity.issue_access_and_refresh_tokens().await?;
                        self.repository.save(identity).await?;
                        Ok(Some(tokens))
                    }
                    None => Err(Error::EntityNotFound { user }),
                }
            }
            Command::RefreshAccessToken {
                id,
                role,
                refresh_token,
            } => {
                let user = User {
                    id,
                    role: self.convert_to_role(&role)?,
                };
                let identity = self.repository.find_by_user(&user).await?;

                match identity {
                    Some(mut identity) => {
                        identity
                            .validate_refresh_token(&RefreshToken(refresh_token))
                            .await?;
                        let tokens = identity.issue_access_and_refresh_tokens().await?;
                        self.repository.save(identity).await?;
                        Ok(Some(tokens))
                    }
                    None => Err(Error::EntityNotFound { user }),
                }
            }
            Command::InvalidateRefreshToken { id, role } => {
                let user = User { id, role };
                let identity = self.repository.find_by_user(&user).await?;

                match identity {
                    Some(mut identity) => {
                        identity.clear_refresh_token()?;
                        self.repository.save(identity).await?;
                        Ok(None)
                    }
                    None => Err(Error::EntityNotFound { user }),
                }
            }
        }
    }

    fn convert_to_role(&self, role: &str) -> Result<Role, Error> {
        match role {
            "Member" => Ok(Role::Member),
            "Administrator" => Ok(Role::Administrator),
            _ => Err(Error::InvalidRole {
                role: String::from(role),
            }),
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

        let identity = repository
            .find_by_user(&User {
                id,
                role: Role::Member,
            })
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            identity.user,
            User {
                id,
                role: Role::Member
            }
        );
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
        assert_eq!(
            result,
            Err(Error::AlreadyRegistered {
                user: User {
                    id,
                    role: Role::Member
                }
            })
        );
    }

    #[tokio::test]
    async fn issuing_access_and_refresh_tokens_return_tokens_and_stores_refresh_token() {
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
        let tokens = command_executor.execute(command).await.unwrap();

        assert!(tokens.is_some());
    }

    #[tokio::test]
    async fn issuing_access_and_refresh_tokens_fail_when_identity_not_found() {
        let repository = MemoryRepository::new();
        let mut command_executor = CommandExecutor::new(repository.clone());

        let id = Uuid::new_v4();
        let command = Command::IssueAccessToken {
            id,
            role: String::from("Member"),
        };
        let result = command_executor.execute(command).await;
        assert_eq!(
            result,
            Err(Error::EntityNotFound {
                user: User {
                    id,
                    role: Role::Member
                }
            })
        );
    }

    #[tokio::test]
    async fn refresh_access_token_succeeds_when_requested_and_persisted_tokens_are_same() {
        let repository = MemoryRepository::new();
        let mut command_executor = CommandExecutor::new(repository.clone());

        let id = Uuid::new_v4();
        let user = User {
            id,
            role: Role::Member,
        };
        let mut identity = Identity {
            user: user.clone(),
            refresh_token: None,
        };
        identity.issue_access_and_refresh_tokens().await.unwrap();
        repository.save(identity.clone()).await.unwrap();

        let refresh_token = identity.refresh_token.unwrap().0.clone();
        let command = Command::RefreshAccessToken {
            id,
            role: String::from("Member"),
            refresh_token,
        };
        let tokens = command_executor.execute(command).await.unwrap();

        assert!(tokens.is_some());
    }

    #[tokio::test]
    async fn refresh_access_token_fails_when_requested_and_persisted_tokens_are_mismatched() {
        let repository = MemoryRepository::new();
        let mut command_executor = CommandExecutor::new(repository.clone());

        let id = Uuid::new_v4();
        let user = User {
            id,
            role: Role::Member,
        };
        let identity = Identity {
            user: user.clone(),
            refresh_token: Some(RefreshToken(String::from("000.111.222"))),
        };
        repository.save(identity).await.unwrap();

        let command = Command::RefreshAccessToken {
            id,
            role: String::from("Member"),
            refresh_token: String::from("000.000.000"),
        };
        let error = command_executor.execute(command).await.unwrap_err();

        assert!(matches!(error, Error::TokenRefreshFailed { .. }));
    }

    #[tokio::test]
    async fn refresh_access_token_fails_when_no_entity_found() {
        let repository = MemoryRepository::new();
        let mut command_executor = CommandExecutor::new(repository.clone());

        let id = Uuid::new_v4();
        let command = Command::RefreshAccessToken {
            id,
            role: String::from("Member"),
            refresh_token: String::from("000.000.000"),
        };
        let error = command_executor.execute(command).await.unwrap_err();

        assert!(matches!(error, Error::EntityNotFound { .. }));
    }

    #[tokio::test]
    async fn refresh_token_invalidation_removes_persisted_refresh_token() {
        let repository = MemoryRepository::new();
        let mut command_executor = CommandExecutor::new(repository.clone());

        let id = Uuid::new_v4();
        let command = Command::RegisterIdentity {
            id,
            role: String::from("Member"),
        };
        command_executor.execute(command).await.unwrap();

        let command = Command::InvalidateRefreshToken {
            id,
            role: Role::Member,
        };
        command_executor.execute(command).await.unwrap();

        let identity = repository
            .find_by_user(&User {
                id,
                role: Role::Member,
            })
            .await
            .unwrap()
            .unwrap();
        assert_eq!(identity.refresh_token, None);
    }
}
