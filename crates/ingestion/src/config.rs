//! Configuration for the ingestion service.
//!
//! All values are loaded from environment variables at startup.
//! Missing or invalid values cause an immediate, descriptive error
//! so that misconfiguration is caught before any work begins.

use std::collections::HashSet;
use std::env;
use std::time::Duration;

use chrono::{DateTime, Utc};
use domain::types::currency::Currency;

const DEFAULT_START_DATE: &str = "2026-01-01T00:00:00Z";

/// Delay between two historical months during the backfill phase.
///
/// Backfill walks a bounded number of months once per process start, so a
/// short delay is enough to stay polite towards the upstream provider without
/// making the initial catch-up take hours.
const DEFAULT_BACKFILL_INTERVAL_SECS: u64 = 1;

/// Delay between two steady-state polls once the backfill has caught up.
///
/// Reference rates are published at most once per business day, so polling
/// every six hours is ample and keeps upstream traffic negligible.
const DEFAULT_POLL_INTERVAL_SECS: u64 = 6 * 60 * 60;

/// Ingestion service configuration loaded from the environment.
#[derive(Debug, Clone)]
pub(crate) struct IngestionConfig {
    /// Postgres connection string.
    database_url: String,
    /// Starting date for the ingestion process.
    start_date: DateTime<Utc>,
    /// Delay between two historical months while backfilling.
    backfill_interval: Duration,
    /// Delay between two steady-state polls once backfill has caught up.
    poll_interval: Duration,
    /// List of currencies to ingest.
    list_currencies: HashSet<Currency>,
}

/// Reads a positive duration, in seconds, from the environment.
///
/// Falls back to `default_secs` when the variable is absent. A value of zero
/// is rejected because it would make the service spin without pause.
fn interval_from_env<F>(var_fn: &F, key: &str, default_secs: u64) -> Result<Duration, String>
where
    F: Fn(&str) -> Result<String, env::VarError>,
{
    let seconds = match var_fn(key) {
        Ok(raw) => raw
            .trim()
            .parse::<u64>()
            .map_err(|e| format!("invalid {key}: {e}"))?,
        Err(_) => default_secs,
    };

    if seconds == 0 {
        return Err(format!("invalid {key}: must be greater than zero"));
    }

    Ok(Duration::from_secs(seconds))
}

impl IngestionConfig {
    /// Loads configuration from environment variables.
    ///
    /// # Required variables
    ///
    /// - `DATABASE_URL` — Postgres connection string
    ///
    /// # Errors
    ///
    /// Returns an error string if any required variable is missing or malformed.
    pub(crate) fn from_env() -> Result<Self, String> {
        let _ = dotenvy::dotenv();
        Self::from_env_impl(|key| env::var(key))
    }

    fn from_env_impl<F>(var_fn: F) -> Result<Self, String>
    where
        F: Fn(&str) -> Result<String, env::VarError>,
    {
        let database_url =
            var_fn("DATABASE_URL").map_err(|_| "DATABASE_URL environment variable is required")?;

        let start_date = var_fn("START_DATE")
            .unwrap_or_else(|_| DEFAULT_START_DATE.to_owned())
            .parse::<DateTime<Utc>>()
            .map_err(|e| format!("invalid START_DATE: {e}"))?;

        let backfill_interval = interval_from_env(
            &var_fn,
            "BACKFILL_INTERVAL_SECS",
            DEFAULT_BACKFILL_INTERVAL_SECS,
        )?;

        let poll_interval =
            interval_from_env(&var_fn, "POLL_INTERVAL_SECS", DEFAULT_POLL_INTERVAL_SECS)?;

        let list_currencies = var_fn("CURRENCIES")
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .filter_map(|s| Currency::new(s).ok())
            .collect::<HashSet<Currency>>();

        Ok(Self {
            database_url,
            start_date,
            backfill_interval,
            poll_interval,
            list_currencies,
        })
    }

    /// Returns the database connection URL.
    #[must_use]
    pub(crate) fn database_url(&self) -> &str {
        &self.database_url
    }

    /// Returns the starting date for the ingestion process.
    #[must_use]
    pub(crate) const fn start_date(&self) -> &DateTime<Utc> {
        &self.start_date
    }

    /// Returns the delay between two historical months while backfilling.
    #[must_use]
    pub(crate) const fn backfill_interval(&self) -> Duration {
        self.backfill_interval
    }

    /// Returns the delay between two steady-state polls.
    #[must_use]
    pub(crate) const fn poll_interval(&self) -> Duration {
        self.poll_interval
    }

    /// Returns the list of currency pairs to ingest.
    #[must_use]
    pub(crate) fn list_currencies(&self) -> HashSet<Currency> {
        self.list_currencies.clone()
    }
}
