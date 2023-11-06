use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken::{
    decode as jwt_decode, encode as jwt_encode, Algorithm as JwtAlgorithm,
    DecodingKey as JwtDecodingKey, EncodingKey as JwtEncodingKey, Header as JwtHeader,
    Validation as JwtValidation,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::identity::errors::Error;

const ACCESS_TOKEN_DURATION_IN_DAYS: u64 = 1;
const ACCESS_TOKEN_SECRET: &str = "ACCOUNT_ACCESS_TOKEN_SECRET";
const REFRESH_TOKEN_DURATION_IN_DAYS: u64 = 90;
const REFRESH_TOKEN_SECRET: &str = "ACCOUNT_REFRESH_TOKEN_SECRET";

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
struct AccessTokenClaims {
    iat: u64,
    exp: u64,
    id: Uuid,
    role: Role,
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
struct RefreshTokenClaims {
    iat: u64,
    exp: u64,
}

impl Identity {
    pub fn new(user: User, tokens: Option<Tokens>) -> Self {
        Self { user, tokens }
    }
}

impl Identity {
    pub async fn issue_tokens(&mut self) -> Result<(), Error> {
        let tokens = Some(Tokens {
            access_token: self.issue_access_token().await?,
            refresh_token: Identity::issue_refresh_token().await?,
        });
        self.tokens = tokens;
        Ok(())
    }

    async fn issue_access_token(&self) -> Result<AccessToken, Error> {
        let header = JwtHeader::new(JwtAlgorithm::HS256);
        let claims = AccessTokenClaims {
            iat: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            exp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + (ACCESS_TOKEN_DURATION_IN_DAYS * 24 * 60 * 60),
            id: self.user.id,
            role: self.user.role.clone(),
        };
        let key = JwtEncodingKey::from_secret(ACCESS_TOKEN_SECRET.as_ref());

        jwt_encode(&header, &claims, &key)
            .map_err(|error| Error::TokenCreationFailed {
                message: error.to_string(),
            })
            .map(|token| AccessToken(token))
    }

    async fn issue_refresh_token() -> Result<RefreshToken, Error> {
        let header = JwtHeader::new(JwtAlgorithm::HS256);
        let claims = RefreshTokenClaims {
            iat: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            exp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + (REFRESH_TOKEN_DURATION_IN_DAYS * 24 * 60 * 60),
        };
        let key = JwtEncodingKey::from_secret(REFRESH_TOKEN_SECRET.as_ref());

        jwt_encode(&header, &claims, &key)
            .map_err(|error| Error::TokenCreationFailed {
                message: error.to_string(),
            })
            .map(|token| RefreshToken(token))
    }
}

impl Identity {
    pub async fn verify_refresh_token(&self, refresh_token: &RefreshToken) -> Result<(), Error> {
        self.tokens.as_ref().map_or_else(
            || {
                Err(Error::TokenRefreshFailed {
                    message: String::from("Persisted refresh token is not found"),
                })
            },
            |token| match &token.refresh_token == refresh_token {
                true => Ok(()),
                false => Err(Error::TokenRefreshFailed {
                    message: String::from(
                        "The given refresh token does not match with the persisted refresh token",
                    ),
                }),
            },
        )?;

        jwt_decode::<RefreshTokenClaims>(
            &refresh_token.0,
            &JwtDecodingKey::from_secret(REFRESH_TOKEN_SECRET.as_ref()),
            &JwtValidation::new(JwtAlgorithm::HS256),
        )
        .map_err(|error| Error::TokenRefreshFailed {
            message: error.to_string(),
        })?;

        Ok(())
    }

    pub fn invalidate_tokens(&mut self) -> Result<(), Error> {
        self.tokens = None;
        Ok(())
    }
}

impl Identity {
    pub async fn verify_access_token(access_token: &AccessToken) -> Result<User, Error> {
        jwt_decode::<AccessTokenClaims>(
            &access_token.0,
            &JwtDecodingKey::from_secret(ACCESS_TOKEN_SECRET.as_ref()),
            &JwtValidation::new(JwtAlgorithm::HS256),
        )
        .map_err(|error| Error::TokenValidationFailed {
            message: error.to_string(),
        })
        .map(|token| User {
            id: token.claims.id,
            role: token.claims.role,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::identity::models::*;

    #[tokio::test]
    async fn refresh_token_verification_succeeds_when_requested_and_persisted_have_same_one() {
        let mut identity = Identity::new(
            User {
                id: Uuid::new_v4(),
                role: "Member".parse().unwrap(),
            },
            None,
        );
        identity.issue_tokens().await.unwrap();
        let refresh_token = identity.tokens.as_ref().unwrap().refresh_token.clone();

        let result = identity.verify_refresh_token(&refresh_token).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn refresh_token_verification_with_different_tokens_returns_error() {
        let mut identity = Identity::new(
            User {
                id: Uuid::new_v4(),
                role: "Member".parse().unwrap(),
            },
            None,
        );
        identity.issue_tokens().await.unwrap();

        let refresh_token = RefreshToken("eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpYXQiOjE2OTkxOTMyMjAsImV4cCI6MTcwNjk2OTIyMH0.05Lyu4nsJ_eBushLs-uwhuDLPH--D2q5V6zJ3UXXx4Q".into());
        let result = identity.verify_refresh_token(&refresh_token).await;

        assert!(result.is_err());
        assert!(matches!(
            result.as_ref().unwrap_err(),
            Error::TokenRefreshFailed { .. }
        ));
        assert!(result
            .as_ref()
            .unwrap_err()
            .to_string()
            .contains("does not match"));
    }

    #[tokio::test]
    async fn refresh_token_verification_fails_when_persisted_token_not_exist() {
        let identity = Identity::new(
            User {
                id: Uuid::new_v4(),
                role: "Member".parse().unwrap(),
            },
            None,
        );
        let refresh_token = RefreshToken("000.000.000".into());

        let result = identity.verify_refresh_token(&refresh_token).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::TokenRefreshFailed { .. }
        ));
    }

    #[tokio::test]
    async fn refresh_with_expired_token_fails_validation() {
        // exp is 2023-01-01, which should be expired for now
        let tokens = Tokens {
            access_token: AccessToken("".into()),
            refresh_token: RefreshToken("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjE2NzI1MzEyMDAsImlhdCI6MTY3MjUzMTIwMH0.IafsQYx1p6X-18D6l_87mBEsvxIsyullpcuVCMm0mqQ".into()),
        };

        let identity = Identity::new(
            User {
                id: Uuid::new_v4(),
                role: "Member".parse().unwrap(),
            },
            Some(tokens.clone()),
        );

        let result = identity.verify_refresh_token(&tokens.refresh_token).await;

        assert!(result.is_err());
        assert!(matches!(
            result.as_ref().unwrap_err(),
            Error::TokenRefreshFailed { .. }
        ));
        assert!(result
            .as_ref()
            .unwrap_err()
            .to_string()
            .contains("ExpiredSignature"));
    }
}
