//! Ingestion service entrypoint.
//!
//! This binary polls external exchange-rate providers on a configurable
//! interval and persists the results into Postgres via the application
//! layer's `IngestRatesUseCase`.

mod runner;
mod setup;

use std::process::ExitCode;

use crate::setup::setup_and_launch;

#[tokio::main]
async fn main() -> ExitCode {
    setup_and_launch().await
}
