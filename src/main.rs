mod setup;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let listener = setup::init_listener().await?;
    let router = setup::init_router();
    axum::serve(listener, router)
        .with_graceful_shutdown(setup::shutdown())
        .await.expect("failed to start server");
    
    Ok(())
}
