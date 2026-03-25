//! Ingestion service entrypoint.
//!
//! This binary polls external exchange-rate providers on a configurable
//! interval and persists the results into Postgres via the application
//! layer's `IngestRatesUseCase`.

mod config;

use std::process::ExitCode;

use application::ports::rate_provider::RateProvider;
use application::use_cases::ingest_rates::IngestRatesUseCase;
use chrono::naive::Days;
use chrono::{Datelike, Months, NaiveDate};
use domain::types::currency_pair::CurrencyPair;
use domain::types::utils::currency_info_list_to_currency_pairs;
use infrastructure::exchange_rate::repository::PostgresExchangeRateRepository;
use infrastructure::rate_provider::frankfurter::FrankfurterClient;
use sqlx::postgres::PgPoolOptions;
use tracing::{error, info, warn};

use crate::config::IngestionConfig;

#[tokio::main]
async fn main() -> ExitCode {
    let _ = dotenvy::dotenv();

    init_tracing();

    let config = match IngestionConfig::from_env() {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, "Failed to load configuration");
            return ExitCode::FAILURE;
        }
    };

    info!("Ingestion service starting");

    // ── Database ────────────────────────────────────────────────────
    let pool = match PgPoolOptions::new()
        .max_connections(5)
        .connect(config.database_url())
        .await
    {
        Ok(p) => p,
        Err(e) => {
            error!(error = %e, "Failed to connect to database");
            return ExitCode::FAILURE;
        }
    };

    let repository = PostgresExchangeRateRepository::new(pool);
    if let Err(e) = repository.migrate().await {
        error!(error = %e, "Database migration failed");
        return ExitCode::FAILURE;
    }
    info!("Database migrations applied successfully");

    // ── Rate provider ───────────────────────────────────────────────
    let provider = match FrankfurterClient::with_default_url() {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, "Failed to create rate provider client");
            return ExitCode::FAILURE;
        }
    };

    // ── Build currency pairs ────────────────────────────────────────
    let currencies = match provider.fetch_currencies().await {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, "Failed to fetch currencies");
            return ExitCode::FAILURE;
        }
    };

    let pairs: Vec<CurrencyPair> = currency_info_list_to_currency_pairs(&currencies);

    // ── Ingestion loop ──────────────────────────────────────────────
    info!("Starting ingestion loop");
    run_loop(repository, provider, &pairs, &config).await;

    info!("Ingestion service shut down gracefully");
    ExitCode::SUCCESS
}

async fn run_loop(
    repository: PostgresExchangeRateRepository,
    provider: FrankfurterClient,
    pairs: &[CurrencyPair],
    config: &IngestionConfig,
) {
    let use_case = IngestRatesUseCase::new(repository, provider);
    let mut interval = tokio::time::interval(config.interval());

    // Normalise to the first day of the configured start month so that we
    // always request complete calendar months.
    let start = config.start_date().date_naive();
    let Some(mut month_start) = NaiveDate::from_ymd_opt(start.year(), start.month(), 1) else {
        error!("Failed to compute start of month for configured start date");
        return;
    };

    loop {
        tokio::select! {
            _ = interval.tick() => {
                // Last day of the current month: first day of next month minus 1 day.
                let month_end = if let Some(next) = month_start.checked_add_months(Months::new(1)) { next - Days::new(1) } else {
                    warn!(month = %month_start.format("%Y-%m"), "Overflow computing month end — stopping");
                    break;
                };

                info!(
                    month_start = %month_start,
                    month_end   = %month_end,
                    "Ingesting month"
                );

                for pair in pairs {
                    let span = tracing::info_span!(
                        "ingest_month",
                        pair  = %pair,
                        month = %month_start.format("%Y-%m"),
                    );
                    let _guard = span.enter();

                    match use_case.fetch_rates_for_range(pair, month_start, month_end).await {
                        Ok(count) => {
                            info!(
                                pair  = %pair,
                                month = %month_start.format("%Y-%m"),
                                count,
                                "Month rates ingested successfully"
                            );
                        }
                        Err(e) => {
                            warn!(
                                pair  = %pair,
                                month = %month_start.format("%Y-%m"),
                                error = %e,
                                "Failed to ingest month rates"
                            );
                        }
                    }
                }

                // Advance to the first day of the next month.
                if let Some(next) = month_start.checked_add_months(Months::new(1)) { month_start = next } else {
                    warn!("Overflow advancing to next month — stopping");
                    break;
                }
            }
            _ = tokio::signal::ctrl_c() => {
                info!("Received shutdown signal");
                break;
            }
        }
    }
}

fn init_tracing() {
    use tracing_subscriber::{EnvFilter, fmt};

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,sqlx=warn"));

    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();
}
