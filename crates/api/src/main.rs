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
use tracing::info;

use crate::config::ApiConfig;
use crate::routes::{analyze_pair, list_currencies};
use crate::state::AppState;

mod config;
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
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(config.bind_addr()).await?;

    info!("Backend running on {}", config.bind_addr());

    axum::serve(listener, app).await?;
    Ok(())
}
