use std::path::Path;
use std::sync::Arc;
use axum::{
    Router,
    routing::{get, post},
};
use tokio::{
    fs::{File, OpenOptions},
    io::AsyncWriteExt,
    signal,
    time::sleep,
    sync::Notify,
    task::JoinHandle,
};
use log_service::{event, health, AppState, Event};

pub async fn init_listener() -> Result<tokio::net::TcpListener, std::io::Error> {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 7878));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Using IP: {}   port: {}", addr.ip(), addr.port());
    Ok(listener)
}

pub fn init_router() -> (Router, JoinHandle<()>) {
    let state = Arc::new(AppState::new("logs.txt", 10_000));
    let shutdown_notifier = Arc::new(Notify::new());

    let router = Router::new()
        .route("/health", get(health))
        .route("/events", post(event))
        .with_state(Arc::clone(&state));

    let handler = tokio::spawn(async move {
        flush_task(Arc::clone(&state), Arc::clone(&shutdown_notifier)).await;
    });

    (router, handler)
}

pub async fn shutdown() {
    signal::ctrl_c().await.expect("ctrl+c error");
    println!("\nstarting shutdown");
}

async fn flush_task(state: Arc<AppState>, notifier: Arc<Notify>) {
    loop {
        tokio::select! {
            _ = sleep(state.interval()) => { flush_buffer(&state).await.expect("error writing to file"); }
            _ = notifier.notified() => { flush_buffer(&state).await.expect("error writing to file"); break; }
        }
    }
}

async fn flush_buffer(state: &Arc<AppState>) -> Result<(), std::io::Error> {
    let events: Vec<Event> = {
        let mut guard = state.events.lock().await;
        std::mem::take(&mut *guard)
    };
    let events: Box<[u8]> = events.into_iter()
        .map(|e| format!("{}\n", e).into_bytes())
        .flatten()
        .collect();

    let file = try_opening_file(state.filepath()).await?;
    try_writing_to_file(file, &events).await?;
    Ok(())
}

async fn try_opening_file(path: &Path) -> Result<File, std::io::Error> {
    let mut i: u8 = 1;

    loop {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await;

        match file {
            Ok(f) => break Ok(f),
            Err(e) => if i == 4 {
                break Err(e);
            } else {
                eprintln!("failed attempt #{i} to open file. Trying again...");
            },
        }

        i += 1;
    }
}

async fn try_writing_to_file(mut file: File, events: &[u8]) -> Result<(), std::io::Error> {
    for i in 1..=4 {
        let result = file.write_all(events).await;
        match result {
            Ok(_) => break,
            Err(e) => if i == 4 {
                return Err(e);
            } else {
                eprintln!("error #{i} in file write. Trying again...");
            }
        }
    }

    for i in 1..=4 {
        let result = file.flush().await;
        match result {
            Ok(_) => break,
            Err(e) => if i == 4 {
                return Err(e);
            } else {
                eprintln!("error #{i} in file flush. Trying again...");
            }
        }
    }

    Ok(())
}
