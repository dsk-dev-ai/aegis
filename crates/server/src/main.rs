use std::net::SocketAddr;

use tracing_subscriber::EnvFilter;
use veil_server::{app, AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let addr: SocketAddr = std::env::var("VEIL_RELAY_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8080".to_string())
        .parse()?;

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("veil relay listening on http://{addr}");

    axum::serve(listener, app(AppState::new())).await?;
    Ok(())
}
