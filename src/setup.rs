use std::path::Path;
use std::sync::Arc;
use axum::Router;
use axum::routing::{get, post};
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::signal;
use tokio::time::sleep;
use log_service::{event, health, AppState, Event};

pub async fn init_listener() -> tokio::net::TcpListener {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 7878));
    let listener = tokio::net::TcpListener::bind(addr).await.expect("Failed to bind tokio::TcpListener");
    println!("Using IP: {}   port: {}", addr.ip(), addr.port());
    listener
}

pub fn init_router() -> Router {
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

pub async fn shutdown() {
    signal::ctrl_c().await.expect("ctrl+c error");
    println!("\nstarting shutdown");
}

async fn flush_task(state: Arc<AppState>) {
    loop {
        sleep(state.interval()).await;

        let events: Vec<Event> = { state.events.lock().await.drain(..).collect() };
        let events: Box<[u8]> = events.into_iter().map(|e| format!("{}\n", e).into_bytes()).flatten().collect();

        println!("opening file");
        let file = try_opening_file(state.filepath()).await.expect("failed to open file");

        println!("writing to file");
        try_writing_to_file(file, &events).await.expect("failed writing to file");
    }
}

async fn try_opening_file(path: &Path) -> Result<File, ()> {
    for i in 1..=3 {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await;
        match file {
            Ok(f) => return Ok(f),
            Err(_) => {
                eprintln!("failed attempt #{i} to open file. Trying again...");
            },
        }
    }
    Err(())
}

async fn try_writing_to_file(mut file: File, events: &[u8]) -> Result<(), ()> {
    for i in 1..=3 {
        let result = file.write_all(events).await;
        match result {
            Ok(_) => break,
            Err(_) => {
                eprintln!("error #{i} trying to write to file. Trying again...");
                continue;
            }
        }
    }

    for i in 1..=3 {
        let result = file.flush().await;
        match result {
            Ok(_) => return Ok(()),
            Err(_) => {
                eprintln!("error #{i} trying to flush file. Trying again...");
                continue;
            }
        }
    }

    Err(())
}
