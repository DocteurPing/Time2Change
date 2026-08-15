use std::env;

use chrono::{DateTime, Datelike, Months, NaiveDate, NaiveTime, Utc};

use crate::config::IngestionConfig;

/// Builds a stand-in for `env::var` backed by a fixed table, so configuration
/// never depends on the ambient process environment.
pub(crate) fn mock_var(vars: &[(&str, &str)]) -> impl Fn(&str) -> Result<String, env::VarError> {
    |key: &str| {
        vars.iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v.to_string())
            .ok_or(env::VarError::NotPresent)
    }
}

/// Returns the first instant of the month `months_back` months before now.
///
/// Anchoring on the current month keeps backfill tests deterministic: the
/// number of months the runner has to walk is fixed regardless of the date the
/// suite happens to run on.
pub(crate) fn months_ago(months_back: u32) -> DateTime<Utc> {
    let today = Utc::now().date_naive();
    let current_month = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();

    current_month
        .checked_sub_months(Months::new(months_back))
        .unwrap()
        .and_time(NaiveTime::MIN)
        .and_utc()
}

/// Builds a config for the runner tests.
///
/// `start_date` drives how many months the backfill phase has to walk, while
/// the two intervals pace it against the test's paused clock.
pub(crate) fn test_config(
    start_date: DateTime<Utc>,
    currencies: &str,
    backfill_secs: &str,
    poll_secs: &str,
) -> IngestionConfig {
    let start = start_date.to_rfc3339();
    let vars = vec![
        ("DATABASE_URL", "postgres://localhost/test"),
        ("START_DATE", start.as_str()),
        ("CURRENCIES", currencies),
        ("BACKFILL_INTERVAL_SECS", backfill_secs),
        ("POLL_INTERVAL_SECS", poll_secs),
    ];

    IngestionConfig::from_env_impl(mock_var(&vars)).unwrap()
}
