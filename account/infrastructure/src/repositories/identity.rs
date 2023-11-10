use async_trait::async_trait;
use sqlx::mysql::MySqlRow;
use sqlx::{query, MySql, Pool, Row};

use domain::identity::models::entities::{Identity, Tokens, User};
use domain::identity::repositories::Repository;

#[derive(Clone)]
pub struct MySqlRepository {
    pool: Pool<MySql>,
}

impl MySqlRepository {
    pub fn new(pool: Pool<MySql>) -> Self {
        Self { pool }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to execute a query: {0}")]
    QueryExecutionFailed(#[source] sqlx::Error),
}

impl From<Error> for domain::identity::errors::Error {
    fn from(error: Error) -> Self {
        Self::DatabaseOperationFailed(error.into())
    }
}

#[async_trait]
impl Repository for MySqlRepository {
    type Error = Error;

    async fn save(&self, identity: Identity) -> Result<(), Error> {
        query("INSERT INTO identities (user_id, user_role, refresh_token) VALUES (?, ?, ?) ON DUPLICATE KEY UPDATE refresh_token = ?")
            .bind(identity.user.id)
            .bind(Into::<&str>::into(identity.user.role))
            .bind(&identity.tokens.clone().map(|tokens| Into::<String>::into(tokens.refresh_token)))
            .bind(&identity.tokens.clone().map(|tokens| Into::<String>::into(tokens.refresh_token)))
            .execute(&self.pool)
            .await
            .map_err(Error::QueryExecutionFailed)?;

        Ok(())
    }

    async fn find_by_user(&self, user: &User) -> Result<Option<Identity>, Error> {
        let identity =
            query("SELECT refresh_token FROM identities WHERE user_id = ? and user_role = ?")
                .bind(user.id)
                .bind(Into::<&str>::into(user.role.clone()))
                .map(|row: MySqlRow| Identity {
                    user: user.clone(),
                    tokens: row
                        .get::<Option<&str>, &str>("refresh_token")
                        .map(|refresh_token| Tokens::new("".into(), refresh_token.into())),
                })
                .fetch_optional(&self.pool)
                .await
                .map_err(Error::QueryExecutionFailed)?;

        Ok(identity)
    }
}
