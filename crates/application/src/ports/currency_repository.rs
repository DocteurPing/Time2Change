use domain::types::currency_info::CurrencyInfo;

use crate::ports::repository_errors::RepositoryError;

/// Repository port for persisting and retrieving available currencies.
///
/// This trait isolates currency-catalog persistence concerns from
/// exchange-rate time series persistence, enabling dedicated use cases
/// and infrastructure adapters for each responsibility.
#[async_trait::async_trait]
pub trait CurrencyRepository: Send + Sync {
    /// Persists the list of currencies available from the upstream provider.
    ///
    /// Implementations should generally use idempotent semantics
    /// (e.g. upsert by currency code) to support periodic re-sync.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError::Storage`] if the persistence
    /// operation fails.
    async fn save_currencies(&self, currencies: &[CurrencyInfo]) -> Result<(), RepositoryError>;

    /// Returns the list of persisted currencies.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError::Storage`] if reading from storage
    /// fails.
    async fn list_currencies(&self) -> Result<Vec<CurrencyInfo>, RepositoryError>;
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn not_found_display() {
        let err = RepositoryError::NotFound("EUR".into());
        assert_eq!(err.to_string(), "data not found: EUR");
    }

    #[test]
    fn conflict_display() {
        let err = RepositoryError::Conflict("USD".into());
        assert_eq!(err.to_string(), "conflict: data already stored: USD");
    }

    #[test]
    fn storage_display() {
        let err = RepositoryError::Storage("connection refused".into());
        assert_eq!(err.to_string(), "storage failure: connection refused");
    }

    #[test]
    fn invalid_display() {
        let err = RepositoryError::Invalid("empty currency list".into());
        assert_eq!(err.to_string(), "invalid input: empty currency list");
    }

    #[test]
    fn error_is_debug() {
        let err = RepositoryError::NotFound("JPY".into());
        let debug = format!("{err:?}");
        assert!(debug.contains("NotFound"));
        assert!(debug.contains("JPY"));
    }
}
