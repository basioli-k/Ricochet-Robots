//! Starts the server: configuration from the environment, SQLite storage.

use std::sync::Arc;

use server::clock::SystemClock;
use server::config::Config;
use server::sessions;
use server::{AppState, app};
use storage::SqliteStore;
use tower_sessions_core::ExpiredDeletion;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();
    let config = Config::from_vars(|key| std::env::var(key).ok())?;
    if config.google.is_none() {
        tracing::warn!("GOOGLE_CLIENT_ID/SECRET not set: only guests can play");
    }
    let store = Arc::new(SqliteStore::connect(&config.database_url).await?);
    let session_store = sessions::sqlite_store(&store).await?;
    // Purge expired sessions every hour.
    tokio::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_hours(1)),
    );
    let https = config.public_base_url.starts_with("https://");
    let state = AppState {
        players: store.clone(),
        games: store,
        clock: Arc::new(SystemClock),
    };
    let listener = tokio::net::TcpListener::bind(config.bind).await?;
    tracing::info!(address = %config.bind, "listening");
    axum::serve(listener, app(state, sessions::layer(session_store, https)))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
