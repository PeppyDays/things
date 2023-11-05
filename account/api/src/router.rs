use axum::routing::{get, post};
use axum::Router;

use crate::container::Container;
use crate::handlers::check_health::check_health;
use crate::handlers::get_user::get_user;
use crate::handlers::refresh_access_token::refresh_access_token;
use crate::handlers::sign_in::sign_in_with_credential;
use crate::handlers::sign_out::sign_out;
use crate::handlers::sign_up::sign_up_with_credential;

pub fn create_router(container: Container) -> Router {
    Router::new()
        .route("/account/user/sign-out", post(sign_out))
        .route(
            "/account/identity/refresh-access-token",
            post(refresh_access_token),
        )
        .route(
            "/account/user/sign-in-with-credential",
            post(sign_in_with_credential),
        )
        .route("/account/user/get-user/:id", get(get_user))
        .route(
            "/account/user/sign-up-with-credential",
            post(sign_up_with_credential),
        )
        .with_state(container)
        .route("/account/user/check-health", get(check_health))
}
