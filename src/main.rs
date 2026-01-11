mod setup;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let listener = setup::init_listener().await?;
    let (router, flush_handler) = setup::init_router();
    axum::serve(listener, router)
        .with_graceful_shutdown(setup::shutdown())
        .await.expect("failed to start server");
    
    flush_handler.await.expect("error flushing");
    // but now isnt state lost? because we ran the server with_state(state)
    
    Ok(())
}
