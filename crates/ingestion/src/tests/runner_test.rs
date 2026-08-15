use std::sync::Arc;
use std::time::Duration;

use application::ports::rate_provider::RateProviderError;
use application::ports::repository_errors::RepositoryError;
use application::use_cases::ingest_rates::IngestRatesUseCase;
use chrono::naive::Days;
use chrono::{Datelike, Months, NaiveDate, Utc};

use crate::runner::{STEADY_LOOKBACK_DAYS, first_of_month, run_until};
use crate::tests::helpers::{months_ago, test_config};
use crate::tests::mocks::{CallLog, Existing, MockProvider, MockRepository};

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

/// Resolves after `secs` of (virtual) time, standing in for a shutdown signal.
async fn shutdown_after(secs: u64) {
    tokio::time::sleep(Duration::from_secs(secs)).await;
}

/// The trailing window the steady-state phase should be asking for today.
fn expected_steady_window() -> (NaiveDate, NaiveDate) {
    let today = Utc::now().date_naive();
    (
        today
            .checked_sub_days(Days::new(STEADY_LOOKBACK_DAYS))
            .unwrap(),
        today,
    )
}

// ── Date helpers ────────────────────────────────────────────────

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

// ── Guard clauses ───────────────────────────────────────────────

#[tokio::test]
async fn errors_when_no_currencies_are_configured() {
    let log = CallLog::new();
    let use_case = IngestRatesUseCase::new(
        MockRepository::new(Arc::clone(&log), Existing::Nothing),
        MockProvider::ok(Arc::clone(&log)),
    );
    let config = test_config(months_ago(1), "", "1", "3600");

    let error = run_until(&use_case, &config, shutdown_after(1))
        .await
        .unwrap_err();

    assert_eq!(
        error,
        "no currencies configured (CURRENCIES env var is empty)"
    );
    assert_eq!(log.fetch_count(), 0);
}

#[tokio::test]
async fn errors_when_currencies_are_all_malformed() {
    let log = CallLog::new();
    let use_case = IngestRatesUseCase::new(
        MockRepository::new(Arc::clone(&log), Existing::Nothing),
        MockProvider::ok(Arc::clone(&log)),
    );
    // Every entry fails `Currency` validation, so the set ends up empty.
    let config = test_config(months_ago(1), "eur,TOOLONG,12", "1", "3600");

    let error = run_until(&use_case, &config, shutdown_after(1))
        .await
        .unwrap_err();

    assert_eq!(
        error,
        "no currencies configured (CURRENCIES env var is empty)"
    );
}

// ── Backfill phase ──────────────────────────────────────────────

#[tokio::test(start_paused = true)]
async fn backfills_every_complete_month_before_the_current_one() {
    let log = CallLog::new();
    let use_case = IngestRatesUseCase::new(
        MockRepository::new(Arc::clone(&log), Existing::Nothing),
        MockProvider::ok(Arc::clone(&log)),
    );
    let config = test_config(months_ago(3), "EUR,USD", "1", "86400");

    run_until(&use_case, &config, shutdown_after(10))
        .await
        .unwrap();

    let ranges = log.ranges();
    let current_month = first_of_month(Utc::now().date_naive()).unwrap();

    // Three complete months precede the current one, each requested as a full
    // calendar month, and none of them is the current month.
    let month_ranges: Vec<_> = ranges
        .iter()
        .filter(|(start, _)| *start < current_month && start.day() == 1)
        .collect();
    assert_eq!(month_ranges.len(), 3, "ranges were {ranges:?}");

    for (start, end) in month_ranges {
        let next = start.checked_add_months(Months::new(1)).unwrap();
        assert_eq!(*end, next - Days::new(1), "{start} should end its month");
    }
}

#[tokio::test(start_paused = true)]
async fn backfills_each_month_for_every_configured_currency() {
    let log = CallLog::new();
    let use_case = IngestRatesUseCase::new(
        MockRepository::new(Arc::clone(&log), Existing::Nothing),
        MockProvider::ok(Arc::clone(&log)),
    );
    let config = test_config(months_ago(2), "EUR,USD,GBP", "1", "86400");

    run_until(&use_case, &config, shutdown_after(10))
        .await
        .unwrap();

    let current_month = first_of_month(Utc::now().date_naive()).unwrap();
    let backfill_calls = log
        .fetches()
        .into_iter()
        .filter(|call| call.start < current_month && call.start.day() == 1)
        .count();

    // 2 months x 3 base currencies.
    assert_eq!(backfill_calls, 6);
}

#[tokio::test(start_paused = true)]
async fn skips_months_that_are_already_stored() {
    let log = CallLog::new();
    let use_case = IngestRatesUseCase::new(
        MockRepository::new(Arc::clone(&log), Existing::Everything),
        MockProvider::ok(Arc::clone(&log)),
    );
    let config = test_config(months_ago(3), "EUR,USD", "1", "86400");

    run_until(&use_case, &config, shutdown_after(10))
        .await
        .unwrap();

    let current_month = first_of_month(Utc::now().date_naive()).unwrap();
    let backfill_calls = log
        .fetches()
        .into_iter()
        .filter(|call| call.start < current_month && call.start.day() == 1)
        .count();

    assert_eq!(backfill_calls, 0, "stored months must not be re-fetched");
    assert!(log.exists_checks() > 0, "the skip check must actually run");
}

#[tokio::test(start_paused = true)]
async fn fetches_anyway_when_the_existence_check_fails() {
    let log = CallLog::new();
    let use_case = IngestRatesUseCase::new(
        MockRepository::new(Arc::clone(&log), Existing::Failing),
        MockProvider::ok(Arc::clone(&log)),
    );
    let config = test_config(months_ago(2), "EUR,USD", "1", "86400");

    run_until(&use_case, &config, shutdown_after(10))
        .await
        .unwrap();

    // A failing skip check must degrade to fetching, never to silently
    // skipping data the service cannot prove it already has.
    let current_month = first_of_month(Utc::now().date_naive()).unwrap();
    let backfill_calls = log
        .fetches()
        .into_iter()
        .filter(|call| call.start < current_month && call.start.day() == 1)
        .count();

    assert_eq!(backfill_calls, 4); // 2 months x 2 currencies
}

#[tokio::test(start_paused = true)]
async fn skips_the_backfill_entirely_when_start_date_is_the_current_month() {
    let log = CallLog::new();
    let use_case = IngestRatesUseCase::new(
        MockRepository::new(Arc::clone(&log), Existing::Nothing),
        MockProvider::ok(Arc::clone(&log)),
    );
    let config = test_config(months_ago(0), "EUR,USD", "1", "86400");

    run_until(&use_case, &config, shutdown_after(10))
        .await
        .unwrap();

    let current_month = first_of_month(Utc::now().date_naive()).unwrap();
    let backfill_calls = log
        .fetches()
        .into_iter()
        .filter(|call| call.start < current_month && call.start.day() == 1)
        .count();

    assert_eq!(backfill_calls, 0);
    // It must still have reached the steady-state phase.
    assert!(log.fetch_count() > 0);
}

// ── Steady-state phase ──────────────────────────────────────────

#[tokio::test(start_paused = true)]
async fn enters_steady_state_and_refreshes_the_trailing_window() {
    let log = CallLog::new();
    let use_case = IngestRatesUseCase::new(
        MockRepository::new(Arc::clone(&log), Existing::Nothing),
        MockProvider::ok(Arc::clone(&log)),
    );
    let config = test_config(months_ago(1), "EUR,USD", "1", "3600");

    run_until(&use_case, &config, shutdown_after(60))
        .await
        .unwrap();

    let (expected_start, expected_end) = expected_steady_window();
    assert!(
        log.ranges().contains(&(expected_start, expected_end)),
        "expected the trailing window {expected_start}..={expected_end}, got {:?}",
        log.ranges()
    );
}

#[tokio::test(start_paused = true)]
async fn steady_state_polls_repeatedly_instead_of_exiting() {
    let log = CallLog::new();
    let use_case = IngestRatesUseCase::new(
        MockRepository::new(Arc::clone(&log), Existing::Nothing),
        MockProvider::ok(Arc::clone(&log)),
    );
    // Already caught up, so the run is pure steady state.
    let config = test_config(months_ago(0), "EUR", "1", "3600");

    // Five hours of virtual time at one poll per hour.
    run_until(&use_case, &config, shutdown_after(5 * 3600))
        .await
        .unwrap();

    let (start, end) = expected_steady_window();
    let refreshes = log
        .fetches()
        .into_iter()
        .filter(|call| call.start == start && call.end == end)
        .count();

    assert!(
        refreshes >= 3,
        "steady state should poll repeatedly, saw {refreshes}"
    );
}

#[tokio::test(start_paused = true)]
async fn steady_state_never_skips_its_window() {
    let log = CallLog::new();
    let use_case = IngestRatesUseCase::new(
        // Everything is "already stored" — the steady window must ignore that,
        // because a partially published window says nothing about the rest.
        MockRepository::new(Arc::clone(&log), Existing::Everything),
        MockProvider::ok(Arc::clone(&log)),
    );
    let config = test_config(months_ago(0), "EUR", "1", "3600");

    run_until(&use_case, &config, shutdown_after(2 * 3600))
        .await
        .unwrap();

    let (start, end) = expected_steady_window();
    assert!(
        log.fetches()
            .iter()
            .any(|call| call.start == start && call.end == end)
    );
}

// ── Shutdown ────────────────────────────────────────────────────

#[tokio::test(start_paused = true)]
async fn returns_ok_when_shutdown_fires_during_backfill() {
    let log = CallLog::new();
    let use_case = IngestRatesUseCase::new(
        MockRepository::new(Arc::clone(&log), Existing::Nothing),
        MockProvider::ok(Arc::clone(&log)),
    );
    // A long backfill against an immediate shutdown.
    let config = test_config(months_ago(12), "EUR", "3600", "86400");

    let result = run_until(&use_case, &config, shutdown_after(1)).await;

    assert!(result.is_ok());
    // It stopped early rather than walking all twelve months.
    assert!(log.ranges().len() < 12);
}

#[tokio::test(start_paused = true)]
async fn returns_ok_when_shutdown_fires_during_steady_state() {
    let log = CallLog::new();
    let use_case = IngestRatesUseCase::new(
        MockRepository::new(Arc::clone(&log), Existing::Nothing),
        MockProvider::ok(Arc::clone(&log)),
    );
    let config = test_config(months_ago(0), "EUR", "1", "3600");

    let result = run_until(&use_case, &config, shutdown_after(60)).await;

    assert!(result.is_ok());
}

// ── Failure tolerance ───────────────────────────────────────────

#[tokio::test(start_paused = true)]
async fn provider_failures_do_not_stop_the_loop() {
    let log = CallLog::new();
    let use_case = IngestRatesUseCase::new(
        MockRepository::new(Arc::clone(&log), Existing::Nothing),
        MockProvider::failing(Arc::clone(&log), RateProviderError::Timeout),
    );
    let config = test_config(months_ago(2), "EUR,USD", "1", "3600");

    let result = run_until(&use_case, &config, shutdown_after(2 * 3600)).await;

    assert!(result.is_ok(), "a failing provider must not abort the loop");
    // It kept going: every month, every currency, plus steady-state polls.
    assert!(log.fetch_count() > 4, "saw {} calls", log.fetch_count());
}

#[tokio::test(start_paused = true)]
async fn repository_save_failures_do_not_stop_the_loop() {
    let log = CallLog::new();
    let use_case = IngestRatesUseCase::new(
        MockRepository::failing_saves(
            Arc::clone(&log),
            RepositoryError::Storage("disk full".to_owned()),
        ),
        MockProvider::ok(Arc::clone(&log)),
    );
    let config = test_config(months_ago(2), "EUR", "1", "3600");

    let result = run_until(&use_case, &config, shutdown_after(2 * 3600)).await;

    assert!(result.is_ok());
    assert!(log.fetch_count() > 2);
}
