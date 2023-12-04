use std::env;
use std::net::{Ipv4Addr, SocketAddr};

use tokio::net::TcpListener;

use api::{container::get_container, router};

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

    axum::serve(
        TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, port)))
            .await
            .unwrap(),
        app.into_make_service(),
    )
    .await
    .unwrap();
}
