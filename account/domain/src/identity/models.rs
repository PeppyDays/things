use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken::{
    Algorithm as JwtAlgorithm, encode as jwt_encode, EncodingKey as JwtEncodingKey,
    Header as JwtHeader,
};
use serde::Serialize;
use uuid::Uuid;

use crate::identity::errors::Error;

const ACCESS_TOKEN_DURATION_IN_DAYS: u64 = 1;
const ACCESS_TOKEN_SECRET: &str = "ACCOUNT_ACCESS_TOKEN_SECRET";
const REFRESH_TOKEN_DURATION_IN_DAYS: u64 = 90;
const REFRESH_TOKEN_SECRET: &str = "ACCOUNT_REFRESH_TOKEN_SECRET";

#[derive(Clone, PartialEq, Debug)]
pub struct Identity {
    pub user: User,
    pub refresh_token: Option<RefreshToken>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct User {
    pub id: Uuid,
    pub role: Role,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub enum Role {
    Member,
    Administrator,
}

#[derive(PartialEq, Debug)]
pub struct AccessToken(pub String);

#[derive(Serialize)]
struct AccessTokenClaims {
    iat: u64,
    exp: u64,
    id: Uuid,
    role: Role,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RefreshToken(pub String);

#[derive(Serialize)]
struct RefreshTokenClaims {
    iat: u64,
    exp: u64,
}

impl Identity {
    pub async fn new(user: User) -> Self {
        Self {
            user,
            refresh_token: None,
        }
    }

    pub async fn issue_access_token(&mut self) -> Result<AccessToken, Error> {
        let header = JwtHeader::new(JwtAlgorithm::HS256);
        let claims = AccessTokenClaims {
            iat: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as u64,
            exp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as u64
                + (ACCESS_TOKEN_DURATION_IN_DAYS * 24 * 60 * 60),
            id: self.user.id,
            role: self.user.role.clone(),
        };
        let key = JwtEncodingKey::from_secret(ACCESS_TOKEN_SECRET.as_ref());
        let access_token =
            jwt_encode(&header, &claims, &key).map_err(|error| Error::TokenCreationFailed {
                message: error.to_string(),
            })?;

        let claims = RefreshTokenClaims {
            iat: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as u64,
            exp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as u64
                + (REFRESH_TOKEN_DURATION_IN_DAYS * 24 * 60 * 60),
        };
        let key = JwtEncodingKey::from_secret(REFRESH_TOKEN_SECRET.as_ref());
        self.refresh_token = Some(RefreshToken(jwt_encode(&header, &claims, &key).map_err(|error| {
            Error::TokenCreationFailed {
                message: error.to_string(),
            }
        })?));

        Ok(AccessToken(access_token))
    }
}
