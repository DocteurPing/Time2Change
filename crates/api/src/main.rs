//! The main entry point for the API server.
//!
//! This module sets up the HTTP server, routes, and application state.

use std::sync::Arc;

use application::use_cases::analyze_pair::AnalyzePairUseCase;
use axum::Router;
use axum::routing::get;
use domain::types::rate_quality_config::RateQualityConfig;
use infrastructure::currency::repository::PostgresCurrencyRepository;
use infrastructure::exchange_rate::repository::PostgresExchangeRateRepository;
use shared::tracing::init_tracing;
use sqlx::postgres::PgPoolOptions;
use tokio::signal;
use tracing::{error, info};

use crate::config::ApiConfig;
use crate::routes::{analyze_pair, health, list_currencies};
use crate::state::AppState;

mod config;
mod dto;
mod errors;
mod routes;
mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    let config = ApiConfig::from_env()?;
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(config.database_url())
        .await?;

    let rate_quality_config = RateQualityConfig::default();
    let repository = PostgresExchangeRateRepository::new(pool.clone());

    let state = AppState::new(
        Arc::new(PostgresCurrencyRepository::new(pool)),
        Arc::new(AnalyzePairUseCase::new(repository, rate_quality_config)),
    );

    let app = Router::new()
        .route("/currencies", get(list_currencies))
        .route("/analyze", get(analyze_pair))
        .route("/health", get(health))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(config.bind_addr()).await?;

    info!("Backend running on {}", config.bind_addr());

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if signal::ctrl_c().await.is_err() {
            error!("failed to install Ctrl+C handler");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut signal) = signal::unix::signal(signal::unix::SignalKind::terminate()) {
            signal.recv().await;
        } else {
            error!("failed to install terminate signal handler");
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
    info!("Shutting down gracefully");
}
