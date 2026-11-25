//! Starts the server: configuration from the environment, SQLite storage.

use std::sync::Arc;

use domain::fixed_board;
use server::clock::{Clock, SystemClock};
use server::config::Config;
use server::game_socket::Heartbeat;
use server::rate_limit::SlidingWindow;
use server::rooms::actor::RandomShuffler;
use server::rooms::manager::RoomManager;
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
    let store = Arc::new(SqliteStore::connect(&config.database_url).await?);
    let session_store = sessions::sqlite_store(&store).await?;
    // Purge expired sessions every hour. The library's task stops at its
    // first error (e.g. a busy database), so restart it after logging.
    let purger = session_store.clone();
    tokio::spawn(async move {
        loop {
            if let Err(e) = purger
                .clone()
                .continuously_delete_expired(tokio::time::Duration::from_hours(1))
                .await
            {
                tracing::warn!(error = %e, "purging expired sessions failed; retrying");
            }
            tokio::time::sleep(tokio::time::Duration::from_mins(1)).await;
        }
    });
    let https = config.secure_cookies();
    let clock: Arc<dyn Clock> = Arc::new(SystemClock::default());
    let rooms = Arc::new(RoomManager::new(
        config.rooms,
        || (fixed_board::board(), fixed_board::starting_robots()),
        clock.clone(),
        Arc::new(RandomShuffler),
        Arc::new(SlidingWindow::new(
            config.room_creates,
            config.room_create_window_secs.saturating_mul(1000),
        )),
        store.clone(),
    ));
    let idle_rooms = rooms.clone();
    tokio::spawn(async move {
        let mut every = tokio::time::interval(tokio::time::Duration::from_mins(1));
        loop {
            every.tick().await;
            // Each sweep runs as its own task, so a panic in one is logged
            // and the next minute's sweep still runs.
            let sweep = idle_rooms.clone();
            match tokio::spawn(async move { sweep.close_idle().await }).await {
                Ok(0) => {}
                Ok(closed) => tracing::info!(closed, "closed idle rooms"),
                Err(e) => tracing::error!(error = %e, "closing idle rooms failed"),
            }
        }
    });
    let state = AppState {
        players: store.clone(),
        games: store,
        clock,
        guest_limiter: Arc::new(SlidingWindow::new(
            config.guest_starts,
            config.guest_window_secs.saturating_mul(1000),
        )),
        rooms,
        chat_limiter: Arc::new(SlidingWindow::new(config.chat_per_sec, 1000)),
        action_limiter: Arc::new(SlidingWindow::new(config.actions_per_sec, 1000)),
        public_origin: config.public_origin(),
        heartbeat: Heartbeat::default(),
    };
    let listener = tokio::net::TcpListener::bind(config.bind).await?;
    tracing::info!(address = %config.bind, "listening");
    // Connection info gives handlers the client IP for rate limits.
    let service = app(state, sessions::layer(session_store, https))
        .into_make_service_with_connect_info::<std::net::SocketAddr>();
    axum::serve(listener, service)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

/// Ctrl-C, or SIGTERM as sent by container runtimes on stop.
async fn shutdown_signal() {
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(e) => {
                tracing::warn!(error = %e, "can't listen for SIGTERM");
                std::future::pending::<()>().await;
            }
        }
    };
    let interrupt = async {
        if let Err(e) = tokio::signal::ctrl_c().await {
            // Without a handler, wait for SIGTERM rather than stopping now.
            tracing::warn!(error = %e, "can't listen for Ctrl-C");
            std::future::pending::<()>().await;
        }
    };
    tokio::select! {
        () = interrupt => {}
        () = terminate => {}
    }
    tracing::info!("shutting down");
}
