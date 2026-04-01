//! Configuration for the api serice.
//!
//! All values are loaded from environment variables at startup.
//! Missing or invalid values cause an immediate, descriptive error
//! so that misconfiguration is caught before any work begins.

use std::env;

use thiserror::Error;

/// Errors that can occur while loading API configuration.
#[derive(Debug, Error, PartialEq, Eq)]
pub(crate) enum ConfigError {
    #[error("DATABASE_URL environment variable is required")]
    MissingDatabaseUrl,
}

/// API service configuration loaded from the environment.
#[derive(Debug, Clone)]
pub(crate) struct ApiConfig {
    /// Postgres connection string.
    database_url: String,
}

impl ApiConfig {
    /// Loads configuration from environment variables.
    ///
    /// # Required variables
    ///
    /// - `DATABASE_URL` — Postgres connection string
    ///
    /// # Errors
    ///
    /// Returns a `ConfigError` if any required variable is missing or malformed.
    pub(crate) fn from_env() -> Result<Self, ConfigError> {
        let _ = dotenvy::dotenv();
        Self::from_env_impl(|key| env::var(key))
    }

    fn from_env_impl<F>(var_fn: F) -> Result<Self, ConfigError>
    where
        F: Fn(&str) -> Result<String, env::VarError>,
    {
        let database_url = var_fn("DATABASE_URL").map_err(|_| ConfigError::MissingDatabaseUrl)?;

        Ok(Self { database_url })
    }

    /// Returns the database connection URL.
    #[must_use]
    pub(crate) fn database_url(&self) -> &str {
        &self.database_url
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
        let config = ApiConfig::from_env_impl(mock_var(&vars)).unwrap();

        assert_eq!(config.database_url(), "postgres://localhost");
    }

    #[test]
    fn test_from_env_missing_database_url() {
        let vars: Vec<(&str, &str)> = vec![];
        let result = ApiConfig::from_env_impl(mock_var(&vars));

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ConfigError::MissingDatabaseUrl);
    }

    #[test]
    fn test_from_env_default() {
        let result = ApiConfig::from_env();
        assert!(result.is_ok());
    }
}
