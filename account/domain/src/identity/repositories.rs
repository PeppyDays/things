use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;

use crate::identity::errors::Error;
use crate::identity::models::Identity;
use crate::identity::models::User;

#[async_trait]
pub trait Repository {
    async fn save(&self, identity: Identity) -> Result<(), Error>;
    async fn find_by_user(&self, user: &User) -> Result<Option<Identity>, Error>;
}

pub struct MemoryRepository {
    rows: Arc<RwLock<HashMap<String, Identity>>>,
}

impl Clone for MemoryRepository {
    fn clone(&self) -> Self {
        Self {
            rows: Arc::clone(&self.rows),
        }
    }
}

impl MemoryRepository {
    pub fn new() -> Self {
        Self {
            rows: Arc::default(),
        }
    }

    fn get_key(&self, user: &User) -> String {
        format!("{:?}-{:?}", user.id, user.role)
    }
}

#[async_trait]
impl Repository for MemoryRepository {
    async fn save(&self, identity: Identity) -> Result<(), Error> {
        let mut store = self.rows.write().map_err(|_| Error::Unknown)?;

        if let Some(existing_identity) = store.get_mut(&self.get_key(&identity.user)) {
            *existing_identity = identity;
        } else {
            store.insert(self.get_key(&identity.user), identity);
        }

        Ok(())
    }

    async fn find_by_user(&self, user: &User) -> Result<Option<Identity>, Error> {
        let store = self.rows.read().map_err(|_| Error::Unknown)?;
        match store.get(&self.get_key(user)) {
            Some(identity) => Ok(Some(identity.clone())),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::identity::models::*;
    use crate::identity::repositories::*;

    #[tokio::test]
    async fn repository_saves_identity_whether_it_was_saved_or_not() {
        let repository = MemoryRepository::new();
        let mut identity = Identity {
            user: User {
                id: Uuid::new_v4(),
                role: Role::Member,
            },
            refresh_token: None,
        };

        let result = repository.save(identity.clone()).await;
        assert!(result.is_ok());
        assert_eq!(repository.rows.read().unwrap().len(), 1);

        identity.issue_access_and_refresh_tokens().await.unwrap();
        let result = repository.save(identity.clone()).await;
        assert!(result.is_ok());
        assert_eq!(repository.rows.read().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn repository_finds_some_identity_by_user_when_it_exists() {
        let repository = MemoryRepository::new();
        let user = User {
            id: Uuid::new_v4(),
            role: Role::Member,
        };
        let identity = Identity {
            user: user.clone(),
            refresh_token: None,
        };
        repository.save(identity.clone()).await.unwrap();

        let result = repository.find_by_user(&user).await.unwrap().unwrap();

        assert_eq!(result, identity);
    }

    #[tokio::test]
    async fn repository_finds_none_identity_by_user_when_it_does_not_exist() {
        let repository = MemoryRepository::new();
        let user = User {
            id: Uuid::new_v4(),
            role: Role::Member,
        };

        let result = repository.find_by_user(&user).await.unwrap();

        assert!(result.is_none());
    }
}
