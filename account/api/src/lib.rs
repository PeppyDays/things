use axum::Server;

mod container;
mod router;

pub async fn run() {
    let config = container::get_config().await;
    let app = router::create_router();

    Server::bind(&format!("127.0.0.1:{}", config.port).parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
