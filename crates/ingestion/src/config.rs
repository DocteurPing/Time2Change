//! Configuration for the ingestion service.
//!
//! All values are loaded from environment variables at startup.
//! Missing or invalid values cause an immediate, descriptive error
//! so that misconfiguration is caught before any work begins.

use std::env;
use std::time::Duration;

use chrono::Utc;

const DEFAULT_START_DATE: &str = "2026-01-01T00:00:00Z";
const DEFAULT_INTERVAL: Duration = Duration::from_millis(100);

/// Ingestion service configuration loaded from the environment.
#[derive(Debug, Clone)]
pub(crate) struct IngestionConfig {
    /// Postgres connection string.
    database_url: String,
    /// Starting date for the ingestion process.
    start_date: chrono::DateTime<Utc>,
    /// Interval between ingestion runs.
    interval: Duration,
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
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL environment variable is required")?;

        let start_date = env::var("START_DATE")
            .unwrap_or_else(|_| DEFAULT_START_DATE.to_owned())
            .parse::<chrono::DateTime<Utc>>()
            .map_err(|e| format!("invalid START_DATE: {e}"))?;

        let interval = DEFAULT_INTERVAL;

        Ok(Self {
            database_url,
            start_date,
            interval,
        })
    }

    /// Returns the database connection URL.
    #[must_use]
    pub(crate) fn database_url(&self) -> &str {
        &self.database_url
    }

    /// Returns the starting date for the ingestion process.
    #[must_use]
    pub(crate) const fn start_date(&self) -> &chrono::DateTime<Utc> {
        &self.start_date
    }

    /// Returns the interval between ingestion runs.
    #[must_use]
    pub(crate) const fn interval(&self) -> Duration {
        self.interval
    }
}
