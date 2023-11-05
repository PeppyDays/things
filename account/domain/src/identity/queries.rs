use crate::identity::errors::Error;
use crate::identity::models::{Identity, User};

use super::models::AccessToken;

#[derive(Debug)]
pub enum Query {
    GetUserFromAccessToken { access_token: String },
}

#[derive(Clone)]
pub struct QueryReader {}

impl QueryReader {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn read(&self, query: Query) -> Result<User, Error> {
        match query {
            Query::GetUserFromAccessToken { access_token } => {
                let access_token = AccessToken(access_token);
                Identity::extract_user_from_access_token(&access_token).await
            }
        }
    }
}
