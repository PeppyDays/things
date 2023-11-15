use std::env;
use std::net::{Ipv4Addr, SocketAddr};

use api::{container::get_container, router};
use axum::Server;

#[tokio::main]
async fn main() {
    env_logger::init();

    let container = get_container().await;
    let app = router::create_router(container);
    let port = match env::var("ACCOUNT_APPLICATION_PORT") {
        Ok(p) => p.parse::<u16>().unwrap(),
        Err(_) => 8080,
    };

    log::info!(
        "Listening on {}",
        &SocketAddr::from((Ipv4Addr::LOCALHOST, port))
    );

    Server::bind(&SocketAddr::from((Ipv4Addr::LOCALHOST, port)))
        .serve(app.into_make_service())
        .await
        .unwrap();
}
