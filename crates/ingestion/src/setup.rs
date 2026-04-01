use std::process::ExitCode;

use application::use_cases::ingest_rates::IngestRatesUseCase;
use application::use_cases::sync_currencies::SyncCurrenciesUseCase;
use domain::types::currency_pair::CurrencyPair;
use domain::types::utils::currency_info_list_to_currency_pairs;
use infrastructure::currency::repository::PostgresCurrencyRepository;
use infrastructure::exchange_rate::repository::PostgresExchangeRateRepository;
use infrastructure::rate_provider::frankfurter::FrankfurterClient;
use sqlx::postgres::PgPoolOptions;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt};

use crate::config::IngestionConfig;
use crate::runner::run_loop;

#[allow(clippy::too_many_lines)]
pub(crate) async fn setup_and_launch() -> ExitCode {
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

    let exchange_rate_repository = PostgresExchangeRateRepository::new(pool.clone());
    if let Err(e) = exchange_rate_repository.migrate().await {
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

    // ── Currency sync ───────────────────────────────────────────────
    let currency_repository = PostgresCurrencyRepository::new(pool);
    let currency_sync_use_case = SyncCurrenciesUseCase::new(currency_repository, provider.clone());
    let fetched_count = match currency_sync_use_case.execute().await {
        Ok(count) => count,
        Err(e) => {
            error!(error = %e, "Failed to sync currencies");
            return ExitCode::FAILURE;
        }
    };

    info!(
        fetched = fetched_count,
        persisted = fetched_count,
        "Currency sync completed successfully"
    );

    // ── Build currency pairs from persisted currencies ──────────────
    let currencies = match currency_sync_use_case.list_currencies().await {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, "Failed to load currencies from database");
            return ExitCode::FAILURE;
        }
    };

    let pairs: Vec<CurrencyPair> = currency_info_list_to_currency_pairs(&currencies);

    let ingest_use_case = IngestRatesUseCase::new(exchange_rate_repository, provider);

    // ── Ingestion loop ──────────────────────────────────────────────
    info!(pair_count = pairs.len(), "Starting ingestion loop");

    run_loop(&ingest_use_case, &pairs, &config).await;

    info!("Ingestion service shut down gracefully");
    ExitCode::SUCCESS
}

fn init_tracing() {
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
