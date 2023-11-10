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

impl TryFrom<&str> for Role {
    type Error = Error;

    fn try_from(role: &str) -> Result<Self, Self::Error> {
        match role {
            "Member" => Ok(Role::Member),
            "Administrator" => Ok(Role::Administrator),
            _ => Err(Error::InvalidRole(role.to_string())),
        }
    }
}

impl From<Role> for &str {
    fn from(role: Role) -> Self {
        match role {
            Role::Member => "Member",
            Role::Administrator => "Administrator",
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
