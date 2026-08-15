use std::collections::HashSet;

use application::ports::exchange_rate_repository::ExchangeRateRepository;
use application::ports::rate_provider::RateProvider;
use application::use_cases::ingest_rates::IngestRatesUseCase;
use chrono::naive::Days;
use chrono::{Datelike, Months, NaiveDate, Utc};
use domain::types::currency::Currency;
use tracing::{Instrument, error, info, warn};

use crate::config::IngestionConfig;

/// Width of the trailing window re-fetched on every steady-state poll.
///
/// Wide enough to always span a month boundary, so newly published days and
/// late upstream corrections are picked up without special-casing the moment
/// the calendar rolls over.
const STEADY_LOOKBACK_DAYS: u64 = 35;

/// An inclusive range of calendar days to ingest.
#[derive(Debug, Clone, Copy)]
struct DateRange {
    start: NaiveDate,
    end: NaiveDate,
}

/// Runs the ingestion service until a shutdown signal is received.
///
/// The service has two phases:
///
/// 1. **Backfill** — walks complete calendar months from the configured start
///    date up to the current month. Months already present in storage are
///    skipped, so a restart does not re-download history it already holds.
/// 2. **Steady state** — once the backfill has caught up, repeatedly re-fetches
///    a trailing window so newly published rates are picked up. This phase does
///    not terminate on its own.
///
/// # Errors
///
/// Returns an error string when the service is misconfigured and cannot make
/// progress. A clean shutdown returns `Ok(())`.
pub(crate) async fn run_loop<R: ExchangeRateRepository, C: RateProvider>(
    use_case: &IngestRatesUseCase<R, C>,
    config: &IngestionConfig,
) -> Result<(), String> {
    let currencies = config.list_currencies();
    if currencies.is_empty() {
        return Err("no currencies configured (CURRENCIES env var is empty)".to_owned());
    }

    // Normalise to the first day of the configured start month so that we
    // always request complete calendar months.
    let Some(mut month_start) = first_of_month(config.start_date().date_naive()) else {
        return Err("failed to compute start of month for the configured start date".to_owned());
    };

    let shutdown = shutdown_signal();
    tokio::pin!(shutdown);

    // ── Phase 1: backfill complete past months ──────────────────────
    let mut backfill = tokio::time::interval(config.backfill_interval());

    loop {
        let Some(current_month) = first_of_month(Utc::now().date_naive()) else {
            return Err("failed to compute the start of the current month".to_owned());
        };

        // Only complete months are backfilled; the current one belongs to the
        // steady-state phase, which keeps refreshing it as new days publish.
        if month_start >= current_month {
            break;
        }

        tokio::select! {
            biased;
            () = &mut shutdown => {
                info!("Received shutdown signal");
                return Ok(());
            }
            _ = backfill.tick() => {}
        }

        let Some(next) = backfill_month(use_case, &currencies, month_start).await else {
            warn!(
                month = %month_start.format("%Y-%m"),
                "Calendar overflow computing month bounds — ending backfill"
            );
            break;
        };
        month_start = next;
    }

    info!(
        poll_interval_secs = config.poll_interval().as_secs(),
        lookback_days = STEADY_LOOKBACK_DAYS,
        "Backfill complete — entering steady-state polling"
    );

    // ── Phase 2: keep the trailing window fresh ─────────────────────
    let mut poll = tokio::time::interval(config.poll_interval());

    loop {
        tokio::select! {
            biased;
            () = &mut shutdown => {
                info!("Received shutdown signal");
                return Ok(());
            }
            _ = poll.tick() => {}
        }

        refresh_recent(use_case, &currencies).await;
    }
}

/// Re-fetches the trailing window ending today.
///
/// The window is never skipped: it is partially published by definition, so
/// the presence of some rates says nothing about the rest.
async fn refresh_recent<R: ExchangeRateRepository, C: RateProvider>(
    use_case: &IngestRatesUseCase<R, C>,
    currencies: &HashSet<Currency>,
) {
    let today = Utc::now().date_naive();
    let range = DateRange {
        start: today
            .checked_sub_days(Days::new(STEADY_LOOKBACK_DAYS))
            .unwrap_or(today),
        end: today,
    };

    info!(
        window_start = %range.start,
        window_end   = %range.end,
        "Refreshing recent rates"
    );

    ingest_range(use_case, currencies, range, false).await;
}

/// Returns the first day of the calendar month containing `date`.
fn first_of_month(date: NaiveDate) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(date.year(), date.month(), 1)
}

/// Resolves once the process is asked to terminate.
///
/// Handles `SIGINT` and, on Unix, `SIGTERM` — the signal container runtimes
/// send on `stop`. Without the latter a long-running worker is killed
/// abruptly once the grace period expires instead of shutting down cleanly.
async fn shutdown_signal() {
    let ctrl_c = async {
        if tokio::signal::ctrl_c().await.is_err() {
            error!("failed to install Ctrl+C handler");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut signal) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
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
}

/// Backfills the calendar month beginning at `month_start`.
///
/// Returns the first day of the following month, or `None` when the calendar
/// arithmetic overflows.
async fn backfill_month<R: ExchangeRateRepository, C: RateProvider>(
    use_case: &IngestRatesUseCase<R, C>,
    currencies: &HashSet<Currency>,
    month_start: NaiveDate,
) -> Option<NaiveDate> {
    let next_month = month_start.checked_add_months(Months::new(1))?;
    let range = DateRange {
        start: month_start,
        end: next_month - Days::new(1),
    };

    info!(
        month_start = %range.start,
        month_end   = %range.end,
        "Backfilling month"
    );

    ingest_range(use_case, currencies, range, true).await;

    Some(next_month)
}

/// Ingests `range` for every configured currency used as a base.
///
/// When `skip_if_present` is set, a base whose pairs are already fully stored
/// for the range is skipped instead of re-queried upstream. Per-currency
/// failures are logged and do not abort the remaining currencies.
async fn ingest_range<R: ExchangeRateRepository, C: RateProvider>(
    use_case: &IngestRatesUseCase<R, C>,
    currencies: &HashSet<Currency>,
    range: DateRange,
    skip_if_present: bool,
) {
    for currency in currencies {
        let span = tracing::info_span!(
            "ingest_range",
            currency = %currency,
            start    = %range.start,
            end      = %range.end,
        );

        ingest_one(use_case, currencies, range, currency, skip_if_present)
            .instrument(span)
            .await;
    }
}

/// Ingests `range` for a single base currency.
async fn ingest_one<R: ExchangeRateRepository, C: RateProvider>(
    use_case: &IngestRatesUseCase<R, C>,
    currencies: &HashSet<Currency>,
    range: DateRange,
    currency: &Currency,
    skip_if_present: bool,
) {
    if skip_if_present {
        match use_case
            .range_already_ingested(currencies, range.start, range.end, currency)
            .await
        {
            Ok(true) => {
                info!("Range already ingested — skipping");
                return;
            }
            Ok(false) => {}
            Err(e) => warn!(
                error = %e,
                "Could not determine whether the range was already ingested — fetching anyway"
            ),
        }
    }

    match use_case
        .fetch_rates_for_range(currencies, range.start, range.end, currency)
        .await
    {
        Ok(count) => info!(count, "Rates ingested successfully"),
        Err(e) => error!(error = %e, "Failed to ingest rates"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    #[test]
    fn first_of_month_normalises_to_day_one() {
        assert_eq!(first_of_month(date(2026, 8, 15)), Some(date(2026, 8, 1)));
    }

    #[test]
    fn first_of_month_is_idempotent() {
        let first = date(2026, 8, 1);
        assert_eq!(first_of_month(first), Some(first));
    }

    #[test]
    fn first_of_month_handles_year_boundaries() {
        assert_eq!(first_of_month(date(2026, 12, 31)), Some(date(2026, 12, 1)));
        assert_eq!(first_of_month(date(2026, 1, 1)), Some(date(2026, 1, 1)));
    }

    #[test]
    fn month_bounds_cover_the_whole_month() {
        // The same arithmetic `backfill_month` uses to derive its range.
        let month_start = date(2024, 2, 1);
        let next_month = month_start.checked_add_months(Months::new(1)).unwrap();

        // 2024 is a leap year: February must end on the 29th.
        assert_eq!(next_month - Days::new(1), date(2024, 2, 29));
    }

    #[test]
    fn steady_window_spans_a_month_boundary() {
        // The lookback must be wide enough that the window always reaches back
        // into the previous month, whatever the day of the month.
        let first_of_march = date(2026, 3, 1);
        let window_start = first_of_march
            .checked_sub_days(Days::new(STEADY_LOOKBACK_DAYS))
            .unwrap();

        assert!(window_start < date(2026, 2, 1));
    }
}
