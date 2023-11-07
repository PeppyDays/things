use crate::identity::errors::Error;
use crate::identity::models::entities::{AccessToken, Identity, RefreshToken, Tokens, User};
use crate::identity::repositories::Repository;

#[derive(Clone)]
pub struct Service<R: Repository> {
    repository: R,
}

impl<R: Repository> Service<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R: Repository> Service<R> {
    pub async fn register_identity(&self, user: User) -> Result<(), Error> {
        let identity = self.repository.find_by_user(&user).await?;
        if let Some(_) = identity {
            return Err(Error::AlreadyRegistered { user });
        }

        let identity = Identity::new(user, None);
        self.repository.save(identity).await?;

        Ok(())
    }

    pub async fn issue_tokens(&self, user: User) -> Result<Tokens, Error> {
        let mut identity = self
            .repository
            .find_by_user(&user)
            .await?
            .ok_or_else(|| Error::EntityNotFound { user })?;

        identity.issue_tokens().await?;
        self.repository.save(identity.clone()).await?;

        Ok(identity.tokens.unwrap())
    }

    pub async fn refresh_tokens(
        &self,
        user: User,
        refresh_token: RefreshToken,
    ) -> Result<Tokens, Error> {
        let mut identity = self
            .repository
            .find_by_user(&user)
            .await?
            .ok_or_else(|| Error::EntityNotFound { user })?;

        identity.verify_refresh_token(&refresh_token).await?;
        identity.issue_tokens().await?;
        self.repository.save(identity.clone()).await?;

        Ok(identity.tokens.unwrap())
    }

    pub async fn invalidate_tokens(&self, user: User) -> Result<(), Error> {
        let mut identity = self
            .repository
            .find_by_user(&user)
            .await?
            .ok_or_else(|| Error::EntityNotFound { user })?;

        identity.invalidate_tokens()?;
        self.repository.save(identity.clone()).await?;

        Ok(())
    }

    pub async fn verify_access_token(&self, access_token: &AccessToken) -> Result<User, Error> {
        Identity::verify_access_token(access_token).await
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::identity::models::entities::*;
    use crate::identity::repositories::*;
    use crate::identity::services::*;

    #[tokio::test]
    async fn identity_registration_stores_user_information_only() {
        let repository = MemoryRepository::new();
        let service = Service::new(repository.clone());

        let user = User::new(Uuid::new_v4(), Role::Member);
        service.register_identity(user.clone()).await.unwrap();

        let identity = repository.find_by_user(&user).await.unwrap().unwrap();
        assert_eq!(identity.user, user);
        assert!(identity.tokens.is_none());
    }

    #[tokio::test]
    async fn identity_registration_fails_when_existing_entry_found() {
        let repository = MemoryRepository::new();
        let service = Service::new(repository.clone());

        let user = User::new(Uuid::new_v4(), Role::Member);
        service.register_identity(user.clone()).await.unwrap();

        let result = service.register_identity(user.clone()).await;

        assert_eq!(result, Err(Error::AlreadyRegistered { user }));
    }

    #[tokio::test]
    async fn issuing_tokens_return_access_and_refresh_tokens_and_persist_refresh_token() {
        let repository = MemoryRepository::new();
        let service = Service::new(repository.clone());

        let user = User::new(Uuid::new_v4(), Role::Member);
        repository
            .save(Identity::new(user.clone(), None))
            .await
            .unwrap();

        let result = service.issue_tokens(user.clone()).await;
        assert!(result.is_ok());

        let identity = repository.find_by_user(&user).await.unwrap().unwrap();

        assert_eq!(
            identity.clone().tokens.unwrap().refresh_token,
            result.unwrap().refresh_token
        );
    }

    #[tokio::test]
    async fn issuing_tokens_fails_when_identity_not_found() {
        let repository = MemoryRepository::new();
        let service = Service::new(repository.clone());

        let user = User::new(Uuid::new_v4(), Role::Member);
        let result = service.issue_tokens(user.clone()).await;

        assert!(result.is_err());
        assert_eq!(result, Err(Error::EntityNotFound { user: user.clone() }));
    }

    #[tokio::test]
    async fn refresh_tokens_succeeds_when_requested_and_persisted_tokens_are_same() {
        let repository = MemoryRepository::new();
        let service = Service::new(repository.clone());

        let user = User::new(Uuid::new_v4(), Role::Member);
        repository
            .save(Identity::new(user.clone(), None))
            .await
            .unwrap();
        let tokens = service.issue_tokens(user.clone()).await.unwrap();

        let result = service
            .refresh_tokens(user.clone(), tokens.clone().refresh_token)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn refresh_tokens_fails_when_requested_and_persisted_tokens_are_mismatched() {
        let repository = MemoryRepository::new();
        let service = Service::new(repository.clone());

        let user = User::new(Uuid::new_v4(), Role::Member);
        repository
            .save(Identity::new(user.clone(), None))
            .await
            .unwrap();
        service.issue_tokens(user.clone()).await.unwrap();

        let error = service
            .refresh_tokens(user.clone(), "000.000.000".into())
            .await
            .unwrap_err();
        assert!(matches!(error, Error::TokenRefreshFailed { .. }));
    }

    #[tokio::test]
    async fn refresh_access_token_fails_when_no_entry_found() {
        let repository = MemoryRepository::new();
        let service = Service::new(repository.clone());

        let user = User::new(Uuid::new_v4(), Role::Member);
        let error = service
            .refresh_tokens(user.clone(), "000.000.000".into())
            .await
            .unwrap_err();

        assert!(matches!(error, Error::EntityNotFound { .. }));
    }

    #[tokio::test]
    async fn token_invalidation_removes_persisted_refresh_token() {
        let repository = MemoryRepository::new();
        let service = Service::new(repository.clone());

        let user = User::new(Uuid::new_v4(), Role::Member);
        service.register_identity(user.clone()).await.unwrap();
        service.issue_tokens(user.clone()).await.unwrap();

        service.invalidate_tokens(user.clone()).await.unwrap();

        let identity = repository.find_by_user(&user).await.unwrap().unwrap();
        assert_eq!(identity.tokens, None);
    }
}
