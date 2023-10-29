use api::{container::get_container, router};
use axum::Server;

#[tokio::main]
async fn main() {
    let container = get_container().await;
    let app = router::create_router(container);

    Server::bind(&format!("127.0.0.1:8080").parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
