use std::collections::HashSet;
use std::time::Duration;

use chrono::{DateTime, Utc};
use domain::types::currency::Currency;

use crate::config::{DEFAULT_BACKFILL_INTERVAL_SECS, DEFAULT_POLL_INTERVAL_SECS, IngestionConfig};
use crate::tests::helpers::mock_var;

// ── Required values ─────────────────────────────────────────────

#[test]
fn from_env_reads_all_values() {
    let vars = vec![
        ("DATABASE_URL", "postgres://localhost"),
        ("CURRENCIES", "EUR,GBP"),
    ];
    let config = IngestionConfig::from_env_impl(mock_var(&vars)).unwrap();

    assert_eq!(config.database_url(), "postgres://localhost");
    assert_eq!(
        *config.start_date(),
        "2026-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap()
    );
    assert_eq!(
        config.list_currencies(),
        [Currency::new("EUR").unwrap(), Currency::new("GBP").unwrap()]
            .into_iter()
            .collect::<HashSet<_>>()
    );
}

#[test]
fn from_env_requires_database_url() {
    let result = IngestionConfig::from_env_impl(mock_var(&[]));

    assert_eq!(
        result.unwrap_err(),
        "DATABASE_URL environment variable is required"
    );
}

// ── Defaults ────────────────────────────────────────────────────

#[test]
fn from_env_applies_defaults_when_optional_vars_are_absent() {
    let vars = vec![("DATABASE_URL", "postgres://localhost")];
    let config = IngestionConfig::from_env_impl(mock_var(&vars)).unwrap();

    assert_eq!(
        *config.start_date(),
        "2026-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap()
    );
    assert_eq!(
        config.backfill_interval(),
        Duration::from_secs(DEFAULT_BACKFILL_INTERVAL_SECS)
    );
    assert_eq!(
        config.poll_interval(),
        Duration::from_secs(DEFAULT_POLL_INTERVAL_SECS)
    );
    assert!(config.list_currencies().is_empty());
}

#[test]
fn from_env_reads_the_process_environment() {
    // Smoke-covers the dotenv wiring. The outcome depends on the ambient
    // environment, so only the call itself is exercised — every value-level
    // assertion lives in the hermetic tests above.
    let _ = IngestionConfig::from_env();
}

// ── START_DATE ──────────────────────────────────────────────────

#[test]
fn from_env_accepts_a_custom_start_date() {
    let vars = vec![
        ("DATABASE_URL", "postgres://localhost"),
        ("START_DATE", "2024-06-15T12:30:00Z"),
    ];
    let config = IngestionConfig::from_env_impl(mock_var(&vars)).unwrap();

    assert_eq!(
        *config.start_date(),
        "2024-06-15T12:30:00Z".parse::<DateTime<Utc>>().unwrap()
    );
    assert!(config.list_currencies().is_empty());
}

#[test]
fn from_env_rejects_a_malformed_start_date() {
    let vars = vec![
        ("DATABASE_URL", "postgres://localhost"),
        ("START_DATE", "not-a-date"),
    ];
    let result = IngestionConfig::from_env_impl(mock_var(&vars));

    assert!(result.unwrap_err().contains("invalid START_DATE"));
}

// ── Intervals ───────────────────────────────────────────────────

#[test]
fn from_env_accepts_custom_intervals() {
    let vars = vec![
        ("DATABASE_URL", "postgres://localhost"),
        ("BACKFILL_INTERVAL_SECS", "5"),
        ("POLL_INTERVAL_SECS", " 905 "),
    ];
    let config = IngestionConfig::from_env_impl(mock_var(&vars)).unwrap();

    assert_eq!(config.backfill_interval(), Duration::from_secs(5));
    assert_eq!(config.poll_interval(), Duration::from_secs(905));
}

#[test]
fn from_env_rejects_a_zero_poll_interval() {
    let vars = vec![
        ("DATABASE_URL", "postgres://localhost"),
        ("POLL_INTERVAL_SECS", "0"),
    ];
    let result = IngestionConfig::from_env_impl(mock_var(&vars));

    assert_eq!(
        result.unwrap_err(),
        "invalid POLL_INTERVAL_SECS: must be greater than zero"
    );
}

#[test]
fn from_env_rejects_a_zero_backfill_interval() {
    let vars = vec![
        ("DATABASE_URL", "postgres://localhost"),
        ("BACKFILL_INTERVAL_SECS", "0"),
    ];
    let result = IngestionConfig::from_env_impl(mock_var(&vars));

    assert_eq!(
        result.unwrap_err(),
        "invalid BACKFILL_INTERVAL_SECS: must be greater than zero"
    );
}

#[test]
fn from_env_rejects_a_non_numeric_interval() {
    let vars = vec![
        ("DATABASE_URL", "postgres://localhost"),
        ("BACKFILL_INTERVAL_SECS", "soon"),
    ];
    let result = IngestionConfig::from_env_impl(mock_var(&vars));

    assert!(
        result
            .unwrap_err()
            .contains("invalid BACKFILL_INTERVAL_SECS")
    );
}

// ── CURRENCIES parsing ──────────────────────────────────────────

#[test]
fn from_env_trims_and_drops_blank_currency_entries() {
    let vars = vec![
        ("DATABASE_URL", "postgres://localhost"),
        ("CURRENCIES", " EUR , , GBP ,"),
    ];
    let config = IngestionConfig::from_env_impl(mock_var(&vars)).unwrap();

    assert_eq!(
        config.list_currencies(),
        [Currency::new("EUR").unwrap(), Currency::new("GBP").unwrap()]
            .into_iter()
            .collect::<HashSet<_>>()
    );
}

#[test]
fn from_env_silently_drops_malformed_currency_codes() {
    let vars = vec![
        ("DATABASE_URL", "postgres://localhost"),
        ("CURRENCIES", "EUR,eur,EURO,12,USD"),
    ];
    let config = IngestionConfig::from_env_impl(mock_var(&vars)).unwrap();

    assert_eq!(
        config.list_currencies(),
        [Currency::new("EUR").unwrap(), Currency::new("USD").unwrap()]
            .into_iter()
            .collect::<HashSet<_>>()
    );
}

#[test]
fn from_env_deduplicates_currencies() {
    let vars = vec![
        ("DATABASE_URL", "postgres://localhost"),
        ("CURRENCIES", "EUR,EUR,EUR"),
    ];
    let config = IngestionConfig::from_env_impl(mock_var(&vars)).unwrap();

    assert_eq!(config.list_currencies().len(), 1);
}
