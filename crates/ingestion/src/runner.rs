use application::ports::exchange_rate_repository::ExchangeRateRepository;
use application::ports::rate_provider::RateProvider;
use application::use_cases::ingest_rates::IngestRatesUseCase;
use chrono::naive::Days;
use chrono::{Datelike, Months, NaiveDate};
use domain::types::currency_pair::CurrencyPair;
use tracing::{error, info, warn};

use crate::config::IngestionConfig;

pub(crate) async fn run_loop(
    use_case: &IngestRatesUseCase<impl ExchangeRateRepository, impl RateProvider>,
    pairs: &[CurrencyPair],
    config: &IngestionConfig,
) {
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
