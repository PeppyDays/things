use async_trait::async_trait;
use sqlx::{MySql, Pool, query, Row};
use sqlx::mysql::MySqlRow;

use domain::identity::errors::Error;
use domain::identity::models::{Identity, RefreshToken, Role};
use domain::identity::models::User;
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

#[async_trait]
impl Repository for MySqlRepository {
    async fn save(&self, identity: Identity) -> Result<(), Error> {
        query("INSERT INTO identities (user_id, user_role, refresh_token) VALUES (?, ?, ?) ON DUPLICATE KEY UPDATE refresh_token = ?")
            .bind(identity.user.id)
            .bind(match &identity.user.role {
                Role::Member => "Member",
                Role::Administrator => "Administrator",
            })
            .bind(&identity.refresh_token.clone().map(|t| t.0))
            .bind(&identity.refresh_token.clone().map(|t| t.0))
            .execute(&self.pool)
            .await
            .map_err(|error| Error::Database { message: error.to_string() })?;

        Ok(())
    }

    async fn find_by_user(&self, user: &User) -> Result<Option<Identity>, Error> {
        let identity = query("SELECT user_id, user_role, refresh_token FROM identities WHERE user_id = ? and user_role = ?")
            .bind(user.id)
            .bind(match &user.role {
                Role::Member => "Member",
                Role::Administrator => "Administrator",
            })
            .map(|row: MySqlRow| Identity {
                user: User {
                    id: row.get("user_id"),
                    role: match row.get("user_role") {
                        "Member" => Role::Member,
                        "Administrator" => Role::Administrator,
                        _ => panic!("Invalid role"),
                    },
                },
                refresh_token: match row.get("refresh_token") {
                    Some(token) => Some(RefreshToken(token)),
                    None => None,
                },
            })
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| Error::Database { message: error.to_string() })?;

        Ok(identity)
    }
}
