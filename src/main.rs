use std::sync::Arc;
use tokio::sync::Notify;
use log_service::AppState;

mod setup;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app_state = Arc::new(AppState::new("logs.txt", 10_000));
    let shutdown_notifier = Arc::new(Notify::new());

    let listener = setup::init_listener().await?;
    let (router, flush_handler) = setup::init_router(app_state.clone(), shutdown_notifier.clone());
    axum::serve(listener, router)
        .with_graceful_shutdown(setup::shutdown())
        .await.expect("failed to start server");

    shutdown_notifier.notify_waiters();
    flush_handler.await?;

    println!("test");
    Ok(())
}
