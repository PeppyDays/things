use std::net::{Ipv4Addr, SocketAddr};

use api::{container::get_container, router};
use axum::Server;

#[tokio::main]
async fn main() {
    let container = get_container().await;
    let app = router::create_router(container);

    Server::bind(&SocketAddr::from((Ipv4Addr::UNSPECIFIED, 8080)))
        .serve(app.into_make_service())
        .await
        .unwrap();
}
