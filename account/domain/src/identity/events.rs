use async_trait::async_trait;
use event_sourcing::event::{DomainEvent, EventApplier};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::identity::models::{Identity, RefreshToken, Role};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Event {
    IdentityRegistered { id: Uuid, role: Role },
    AccessTokenIssued { refresh_token: RefreshToken },
    RefreshTokenRenewed { refresh_token: RefreshToken },
    RefreshTokenInvalidated,
}

impl DomainEvent for Event {
    fn get_name(&self) -> String {
        match self {
            Event::IdentityRegistered { .. } => String::from("IdentityRegistered"),
            Event::AccessTokenIssued { .. } => String::from("AccessTokenIssued"),
            Event::RefreshTokenRenewed { .. } => String::from("RefreshTokenRenewed"),
            Event::RefreshTokenInvalidated => String::from("RefreshTokenInvalidated"),
        }
    }

    fn get_version(&self) -> String {
        match self {
            Event::IdentityRegistered { .. } => String::from("1.0.0"),
            Event::AccessTokenIssued { .. } => String::from("1.0.0"),
            Event::RefreshTokenRenewed { .. } => String::from("1.0.0"),
            Event::RefreshTokenInvalidated => String::from("1.0.0"),
        }
    }
}

#[async_trait]
impl EventApplier<Identity> for Identity {
    async fn apply(&mut self, event: Event) {
        match event {
            Event::IdentityRegistered { id, role } => {
                self.id = id;
                self.role = role;
            }
            Event::AccessTokenIssued { refresh_token } => {
                self.refresh_token = Some(refresh_token);
            }
            Event::RefreshTokenRenewed { refresh_token } => {
                self.refresh_token = Some(refresh_token);
            }
            Event::RefreshTokenInvalidated => {
                self.refresh_token = Option::None;
            }
        }
    }
}
