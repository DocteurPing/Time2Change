use application::ports::exchange_rate_repository::ExchangeRateRepository;
use application::ports::rate_provider::RateProvider;
use application::use_cases::ingest_rates::IngestRatesUseCase;
use chrono::naive::Days;
use chrono::{Datelike, Months, NaiveDate};
use tracing::{error, info, warn};

use crate::config::IngestionConfig;

#[allow(clippy::too_many_lines)]
pub(crate) async fn run_loop(
    use_case: &IngestRatesUseCase<impl ExchangeRateRepository, impl RateProvider>,
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

    let currencies = config.list_currencies();

    let ctrl_c = tokio::signal::ctrl_c();
    tokio::pin!(ctrl_c);

    loop {
        let next_month: Option<NaiveDate> = tokio::select! {
            biased;

            _ = &mut ctrl_c => {
                info!("Received shutdown signal");
                break;
            }

            next = async {
                interval.tick().await;

                let Some(month_end) = month_start
                    .checked_add_months(Months::new(1))
                    .map(|n| n - Days::new(1))
                else {
                    warn!(
                        month = %month_start.format("%Y-%m"),
                        "Overflow computing month end — stopping"
                    );
                    return None;
                };

                if month_start > chrono::Utc::now().date_naive() {
                    info!(
                        month = %month_start.format("%Y-%m"),
                        "End of the ingestion process, we reached the current date - stopping"
                    );
                    return None;
                }

                info!(
                    month_start = %month_start,
                    month_end   = %month_end,
                    "Ingesting month"
                );

                for currency in &currencies {
                    let span = tracing::info_span!(
                        "ingest_month",
                        currency  = %currency,
                        month = %month_start.format("%Y-%m"),
                    );
                    let _guard = span.enter();

                    match use_case.fetch_rates_for_range(&currencies, month_start, month_end, currency).await {
                        Ok(count) => info!(
                            currency  = %currency,
                            month = %month_start.format("%Y-%m"),
                            count,
                            "Month rates ingested successfully"
                        ),
                        Err(e) => warn!(
                            currency  = %currency,
                            month = %month_start.format("%Y-%m"),
                            error = %e,
                            "Failed to ingest month rates"
                        ),
                    }
                }

                month_start.checked_add_months(Months::new(1))
            } => next,
        };

        let Some(next) = next_month else {
            break;
        };
        month_start = next;
    }
}
