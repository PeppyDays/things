use axum::middleware::from_fn_with_state;
use axum::routing::{get, post};
use axum::Router;

use crate::container::Container;
use crate::handlers::*;
use crate::middleware::*;

pub fn create_router(container: Container) -> Router {
    Router::new()
        .route("/account/user/sign-out", post(sign_out::handle))
        .route_layer(from_fn_with_state(
            container.clone(),
            require_authentication,
        ))
        .route(
            "/account/identity/refresh-tokens",
            post(refresh_tokens::handle),
        )
        .route("/account/user/sign-in", post(sign_in::handle))
        .route("/account/user/get-user/:id", get(get_user::handle))
        .route("/account/user/sign-up", post(sign_up::handle))
        .with_state(container)
        .route("/account/user/check-health", get(check_health::handle))
}
