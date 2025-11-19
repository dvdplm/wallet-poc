use axum::{
    Router,
    routing::{delete, get, post},
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

mod handlers;
mod state;

use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing with colored output
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    info!("Starting signing server...");

    let app_state = Arc::new(RwLock::new(AppState::new()));

    // Build router with all endpoints
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/register", post(handlers::register))
        .route("/sign", post(handlers::sign))
        .route("/forget", delete(handlers::forget))
        .with_state(app_state);

    let addr = "127.0.0.1:3000";
    let listener = tokio::net::TcpListener::bind(addr).await?;

    info!("Server listening on http://{}", addr);

    axum::serve(listener, app).await?;

    info!("Server shut down gracefully");
    Ok(())
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_health_check() {
        let result = health_check().await;
        assert_eq!(result, "OK");
    }
}
