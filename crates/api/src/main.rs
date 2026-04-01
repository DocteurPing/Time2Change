//! The main entry point for the API server.
//!
//! This module sets up the HTTP server, routes, and application state.

use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use infrastructure::currency::repository::PostgresCurrencyRepository;
use shared::config::IngestionConfig;
use sqlx::postgres::PgPoolOptions;

use crate::routes::list_currencies;
use crate::state::AppState;

mod errors;
mod routes;
mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = IngestionConfig::from_env()?;
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(config.database_url())
        .await?;

    let state = AppState {
        currency_repo: Arc::new(PostgresCurrencyRepository::new(pool)),
    };

    let app = Router::new()
        .route("/currencies", get(list_currencies))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    println!("Backend running on http://localhost:3000");

    axum::serve(listener, app).await?;
    Ok(())
}
