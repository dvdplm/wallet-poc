use axum::{
    Router,
    routing::{delete, get, post},
};
use axum_server::tls_rustls::RustlsConfig;
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
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
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

    // Load TLS configuration
    let config = RustlsConfig::from_pem_file(
        "signingserver/certs/cert.pem",
        "signingserver/certs/key.pem",
    )
    .await?;

    let addr = "127.0.0.1:3443";

    info!("Server listening on https://{}", addr);
    info!("Note: Using self-signed certificate.");

    axum_server::bind_rustls(addr.parse()?, config)
        .serve(app.into_make_service())
        .await?;

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
