use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::identity::errors::Error;

#[derive(Clone, PartialEq, Debug)]
pub struct Identity {
    pub user: User,
    pub tokens: Option<Tokens>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct User {
    pub id: Uuid,
    pub role: Role,
}

impl User {
    pub fn new(id: Uuid, role: Role) -> Self {
        Self { id, role }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Role {
    Member,
    Administrator,
}

impl FromStr for Role {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Member" => Ok(Role::Member),
            "Administrator" => Ok(Role::Administrator),
            _ => Err(Error::InvalidRole {
                role: s.to_string(),
            }),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct Tokens {
    pub access_token: AccessToken,
    pub refresh_token: RefreshToken,
}

impl Tokens {
    pub fn new(access_token: AccessToken, refresh_token: RefreshToken) -> Self {
        Self {
            access_token,
            refresh_token,
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct AccessToken(pub String);

impl From<&str> for AccessToken {
    fn from(token: &str) -> Self {
        Self(token.to_string())
    }
}

impl From<String> for AccessToken {
    fn from(token: String) -> Self {
        Self(token)
    }
}

impl From<AccessToken> for String {
    fn from(token: AccessToken) -> Self {
        token.0
    }
}

#[derive(Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub iat: u64,
    pub exp: u64,
    pub id: Uuid,
    pub role: Role,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RefreshToken(pub String);

impl From<&str> for RefreshToken {
    fn from(token: &str) -> Self {
        Self(token.to_string())
    }
}

impl From<String> for RefreshToken {
    fn from(token: String) -> Self {
        Self(token)
    }
}

impl From<RefreshToken> for String {
    fn from(token: RefreshToken) -> Self {
        token.0
    }
}

#[derive(Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    pub iat: u64,
    pub exp: u64,
}

impl Identity {
    pub fn new(user: User, tokens: Option<Tokens>) -> Self {
        Self { user, tokens }
    }
}
