//! Configuration for the ingestion service.
//!
//! All values are loaded from environment variables at startup.
//! Missing or invalid values cause an immediate, descriptive error
//! so that misconfiguration is caught before any work begins.

use std::env;
use std::time::Duration;

use chrono::Utc;
use thiserror::Error;

const DEFAULT_START_DATE: &str = "2026-01-01T00:00:00Z";
const DEFAULT_INTERVAL: Duration = Duration::from_millis(100);

/// Errors that can occur while loading ingestion configuration.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    /// The `DATABASE_URL` environment variable is required.
    #[error("DATABASE_URL environment variable is required")]
    MissingDatabaseUrl,

    /// The `START_DATE` environment variable is invalid.
    #[error("invalid START_DATE: {0}")]
    InvalidStartDate(String),
}

/// Ingestion service configuration loaded from the environment.
#[derive(Debug, Clone)]
pub struct IngestionConfig {
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
    pub fn from_env() -> Result<Self, ConfigError> {
        let _ = dotenvy::dotenv();
        Self::from_env_impl(|key| env::var(key))
    }

    fn from_env_impl<F>(var_fn: F) -> Result<Self, ConfigError>
    where
        F: Fn(&str) -> Result<String, env::VarError>,
    {
        let database_url = var_fn("DATABASE_URL").map_err(|_| ConfigError::MissingDatabaseUrl)?;

        let start_date_raw = var_fn("START_DATE").unwrap_or_else(|_| DEFAULT_START_DATE.to_owned());

        let start_date = start_date_raw
            .parse::<chrono::DateTime<Utc>>()
            .map_err(|e| ConfigError::InvalidStartDate(e.to_string()))?;

        let interval = DEFAULT_INTERVAL;

        Ok(Self {
            database_url,
            start_date,
            interval,
        })
    }

    /// Returns the database connection URL.
    #[must_use]
    pub fn database_url(&self) -> &str {
        &self.database_url
    }

    /// Returns the starting date for the ingestion process.
    #[must_use]
    pub const fn start_date(&self) -> &chrono::DateTime<Utc> {
        &self.start_date
    }

    /// Returns the interval between ingestion runs.
    #[must_use]
    pub const fn interval(&self) -> Duration {
        self.interval
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_var(vars: &[(&str, &str)]) -> impl Fn(&str) -> Result<String, env::VarError> {
        |key: &str| {
            vars.iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| v.to_string())
                .ok_or(env::VarError::NotPresent)
        }
    }

    #[test]
    fn test_from_env_success() {
        let vars = vec![("DATABASE_URL", "postgres://localhost")];
        let config = IngestionConfig::from_env_impl(mock_var(&vars)).unwrap();

        assert_eq!(config.database_url(), "postgres://localhost");
        assert_eq!(
            *config.start_date(),
            "2026-01-01T00:00:00Z"
                .parse::<chrono::DateTime<Utc>>()
                .unwrap()
        );
        assert_eq!(config.interval(), DEFAULT_INTERVAL);
    }

    #[test]
    fn test_from_env_with_custom_start_date() {
        let vars = vec![
            ("DATABASE_URL", "postgres://localhost"),
            ("START_DATE", "2024-06-15T12:30:00Z"),
        ];
        let config = IngestionConfig::from_env_impl(mock_var(&vars)).unwrap();

        assert_eq!(
            *config.start_date(),
            "2024-06-15T12:30:00Z"
                .parse::<chrono::DateTime<Utc>>()
                .unwrap()
        );
    }

    #[test]
    fn test_from_env_missing_database_url() {
        let vars: Vec<(&str, &str)> = vec![];
        let result = IngestionConfig::from_env_impl(mock_var(&vars));

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigError::MissingDatabaseUrl
        ));
    }

    #[test]
    fn test_from_env_invalid_start_date() {
        let vars = vec![
            ("DATABASE_URL", "postgres://localhost"),
            ("START_DATE", "not-a-date"),
        ];
        let result = IngestionConfig::from_env_impl(mock_var(&vars));

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigError::InvalidStartDate(_)
        ));
    }

    #[test]
    fn test_from_env_default() {
        let result = IngestionConfig::from_env().unwrap();
        assert_eq!(
            *result.start_date(),
            "2026-01-01T00:00:00Z"
                .parse::<chrono::DateTime<Utc>>()
                .unwrap()
        );
        assert_eq!(result.interval(), DEFAULT_INTERVAL);
    }
}
