mod setup;

#[tokio::main]
async fn main() {
    let listener = setup::init_listener().await;
    let router = setup::init_router();
    axum::serve(listener, router)
        .with_graceful_shutdown(setup::shutdown())
        .await.expect("failed to start server");
}
