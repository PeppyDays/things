use std::env;
use std::net::{Ipv4Addr, SocketAddr};

use api::{container::get_container, router};
use axum::Server;

#[tokio::main]
async fn main() {
    let container = get_container().await;
    let app = router::create_router(container);
    let port = match env::var("ACCOUNT_APPLICATION_PORT") {
        Ok(p) => p.parse::<u16>().unwrap(),
        Err(_) => 8080,
    };

    Server::bind(&SocketAddr::from((Ipv4Addr::LOCALHOST, port)))
        .serve(app.into_make_service())
        .await
        .unwrap();
}
