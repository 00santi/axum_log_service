use axum::routing::{ get, post };
use axum::{ Router };
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
    let router = Router::new()
        .route("/health", get(health))
        .route("/events", post(event));
    router
}
