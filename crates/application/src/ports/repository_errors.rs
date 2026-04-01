use thiserror::Error;

/// Errors produced by different repositories implementations.
#[derive(Error, Debug, Clone)]
pub enum RepositoryError {
    /// The requested currency pair or data slice could not be found.
    #[error("data not found: {0}")]
    NotFound(String),

    /// The requested write operation conflicts with already stored data.
    #[error("conflict: data already stored: {0}")]
    Conflict(String),

    /// The underlying storage system failed while processing the request.
    #[error("storage failure: {0}")]
    Storage(String),

    /// The caller supplied invalid input for the repository operation.
    #[error("invalid input: {0}")]
    Invalid(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_display() {
        let err = RepositoryError::NotFound("EUR-USD".into());
        assert_eq!(err.to_string(), "data not found: EUR-USD");
    }

    #[test]
    fn conflict_display() {
        let err = RepositoryError::Conflict("2024-01-01".into());
        assert_eq!(err.to_string(), "conflict: data already stored: 2024-01-01");
    }

    #[test]
    fn storage_display() {
        let err = RepositoryError::Storage("connection refused".into());
        assert_eq!(err.to_string(), "storage failure: connection refused");
    }

    #[test]
    fn invalid_display() {
        let err = RepositoryError::Invalid("empty range".into());
        assert_eq!(err.to_string(), "invalid input: empty range");
    }

    #[test]
    fn error_is_debug() {
        let err = RepositoryError::NotFound("XYZ".into());
        let debug = format!("{err:?}");
        assert!(debug.contains("NotFound"));
        assert!(debug.contains("XYZ"));
    }
}
