use axum::routing::{get, post};
use axum::Router;

use crate::container::Container;
use crate::handlers::check_health;
use crate::handlers::get_user;
use crate::handlers::refresh_access_token;
use crate::handlers::sign_in;
use crate::handlers::sign_out;
use crate::handlers::sign_up;

pub fn create_router(container: Container) -> Router {
    Router::new()
        .route("/account/user/sign-out", post(sign_out::handle))
        .route(
            "/account/identity/refresh-access-token",
            post(refresh_access_token::handle),
        )
        .route(
            "/account/user/sign-in-with-credential",
            post(sign_in::handle),
        )
        .route("/account/user/get-user/:id", get(get_user::handle))
        .route(
            "/account/user/sign-up-with-credential",
            post(sign_up::handle),
        )
        .with_state(container)
        .route("/account/user/check-health", get(check_health::handle))
}
