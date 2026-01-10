use std::{
    sync::Arc,
};
use axum::{
    routing::{ get, post },
    Router
};
use tokio::{
    time::sleep,
    fs::OpenOptions,
    io::AsyncWriteExt,
};
use log_service::*;

#[tokio::main]
async fn main() {
    let listener = init_listener().await;
    let router = init_router();
    axum::serve(listener, router).await.unwrap();
}

async fn init_listener() -> tokio::net::TcpListener {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 7878));
    let listener = tokio::net::TcpListener::bind(addr).await.expect("Failed to bind tokio::TcpListener");
    println!("Using IP: {}   port: {}", addr.ip(), addr.port());
    listener
}

fn init_router() -> Router {
    let state = Arc::new(AppState::new("logs.txt", 10_000));
    let router = Router::new()
        .route("/health", get(health))
        .route("/events", post(event))
        .with_state(Arc::clone(&state));

    tokio::spawn(async move {
        flush_task(Arc::clone(&state)).await;
    });

    router
}

async fn flush_task(state: Arc<AppState>) {
    loop {
        sleep(state.interval()).await;

        let events: Vec<Event> = { state.events.lock().await.drain(..).collect() };

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(state.filepath())
            .await
            .expect("error opening log file");

        for e in events {
            let e = format!("{}\n", e);
            file.write_all(e.as_bytes())
                .await
                .expect("error writing to file");
        }

        file.flush()
            .await
            .expect("error flusing file");
    }

}
