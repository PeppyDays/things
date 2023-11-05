use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken::{
    decode as jwt_decode, encode as jwt_encode, Algorithm as JwtAlgorithm,
    DecodingKey as JwtDecodingKey, EncodingKey as JwtEncodingKey, Header as JwtHeader, TokenData,
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
    pub refresh_token: Option<RefreshToken>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct User {
    pub id: Uuid,
    pub role: Role,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Role {
    Member,
    Administrator,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Tokens {
    pub access_token: AccessToken,
    pub refresh_token: RefreshToken,
}

#[derive(PartialEq, Debug, Clone)]
pub struct AccessToken(pub String);

#[derive(Serialize, Deserialize)]
struct AccessTokenClaims {
    iat: u64,
    exp: u64,
    id: Uuid,
    role: Role,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RefreshToken(pub String);

#[derive(Serialize, Deserialize)]
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

    pub async fn issue_access_and_refresh_tokens(&mut self) -> Result<Tokens, Error> {
        let access_token = self.issue_access_token().await?;
        let refresh_token = self.issue_refresh_token().await?;
        self.refresh_token = Some(refresh_token.clone());

        Ok(Tokens {
            access_token,
            refresh_token,
        })
    }

    async fn issue_access_token(&self) -> Result<AccessToken, Error> {
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

        jwt_encode(&header, &claims, &key)
            .map_err(|error| Error::TokenCreationFailed {
                message: error.to_string(),
            })
            .map(|token| AccessToken(token))
    }

    async fn issue_refresh_token(&self) -> Result<RefreshToken, Error> {
        let header = JwtHeader::new(JwtAlgorithm::HS256);
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

        jwt_encode(&header, &claims, &key)
            .map_err(|error| Error::TokenCreationFailed {
                message: error.to_string(),
            })
            .map(|token| RefreshToken(token))
    }

    pub async fn validate_refresh_token(&self, refresh_token: &RefreshToken) -> Result<(), Error> {
        self.refresh_token.as_ref().map_or_else(
            || {
                Err(Error::TokenRefreshFailed {
                    message: String::from("Persisted refresh token is not found"),
                })
            },
            |token| match token == refresh_token {
                true => Ok(()),
                false => Err(Error::TokenRefreshFailed {
                    message: String::from(
                        "The given refresh token does not match with the persisted refresh token",
                    ),
                }),
            },
        )?;

        self.decode_refresh_token(&refresh_token).await?;

        Ok(())
    }

    async fn decode_refresh_token(
        &self,
        refresh_token: &RefreshToken,
    ) -> Result<TokenData<RefreshTokenClaims>, Error> {
        jwt_decode::<RefreshTokenClaims>(
            &refresh_token.0,
            &JwtDecodingKey::from_secret(REFRESH_TOKEN_SECRET.as_ref()),
            &JwtValidation::new(JwtAlgorithm::HS256),
        )
        .map_err(|error| Error::TokenRefreshFailed {
            message: error.to_string(),
        })
    }

    pub fn clear_refresh_token(&mut self) -> Result<(), Error> {
        self.refresh_token = None;
        Ok(())
    }

    pub async fn extract_user_from_access_token(access_token: &AccessToken) -> Result<User, Error> {
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
    async fn issuing_access_and_refresh_tokens_return_tokens() {
        let mut identity = Identity {
            user: User {
                id: Uuid::new_v4(),
                role: Role::Member,
            },
            refresh_token: None,
        };
        let _tokens = identity.issue_access_and_refresh_tokens().await;
        assert!(identity.refresh_token.is_some());
    }

    #[tokio::test]
    async fn refresh_token_validation_succeeds_when_requested_and_persisted_have_same_one() {
        let mut identity = Identity {
            user: User {
                id: Uuid::new_v4(),
                role: Role::Member,
            },
            refresh_token: None,
        };
        let refresh_token = identity
            .issue_access_and_refresh_tokens()
            .await
            .unwrap()
            .refresh_token;

        let result = identity.validate_refresh_token(&refresh_token).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn refresh_token_validation_with_different_tokens_returns_error() {
        let mut identity = Identity {
            user: User {
                id: Uuid::new_v4(),
                role: Role::Member,
            },
            refresh_token: None,
        };
        identity
            .issue_access_and_refresh_tokens()
            .await
            .unwrap()
            .refresh_token;
        let refresh_token = RefreshToken(String::from("eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpYXQiOjE2OTkxOTMyMjAsImV4cCI6MTcwNjk2OTIyMH0.05Lyu4nsJ_eBushLs-uwhuDLPH--D2q5V6zJ3UXXx4Q"));

        assert_ne!(&refresh_token, identity.refresh_token.as_ref().unwrap());

        let result = identity.validate_refresh_token(&refresh_token).await;

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
    async fn refresh_token_validation_fails_when_persisted_token_not_exist() {
        let identity = Identity {
            user: User {
                id: Uuid::new_v4(),
                role: Role::Member,
            },
            refresh_token: None,
        };
        let refresh_token = RefreshToken(String::from("000.000.000"));

        let result = identity.validate_refresh_token(&refresh_token).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::TokenRefreshFailed { .. }
        ));
    }

    #[tokio::test]
    async fn refresh_with_expired_token_fails_validation() {
        let refresh_token = RefreshToken(String::from("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjE2NzI1MzEyMDAsImlhdCI6MTY3MjUzMTIwMH0.IafsQYx1p6X-18D6l_87mBEsvxIsyullpcuVCMm0mqQ"));
        let identity = Identity {
            user: User {
                id: Uuid::new_v4(),
                role: Role::Member,
            },
            // exp is 2023-01-01, which should be expired for now
            refresh_token: Some(refresh_token.clone()),
        };

        let result = identity.validate_refresh_token(&refresh_token).await;

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
