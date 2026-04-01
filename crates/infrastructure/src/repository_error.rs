use application::ports::repository_errors::RepositoryError;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn to_repository_error(e: sqlx::Error) -> RepositoryError {
    match e {
        sqlx::Error::RowNotFound => RepositoryError::NotFound(e.to_string()),
        sqlx::Error::Database(e) => {
            if e.is_unique_violation() {
                RepositoryError::Conflict(e.to_string())
            } else {
                RepositoryError::Storage(e.to_string())
            }
        }
        _ => RepositoryError::Storage(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use std::fmt;

    use sqlx::error::DatabaseError;

    use super::*;

    #[derive(Debug)]
    struct MockDatabaseError {
        message: String,
        unique_violation: bool,
    }

    impl fmt::Display for MockDatabaseError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.message)
        }
    }

    impl std::error::Error for MockDatabaseError {}

    impl DatabaseError for MockDatabaseError {
        fn message(&self) -> &str {
            &self.message
        }

        fn kind(&self) -> sqlx::error::ErrorKind {
            if self.unique_violation {
                sqlx::error::ErrorKind::UniqueViolation
            } else {
                sqlx::error::ErrorKind::Other
            }
        }

        fn as_error(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
            self
        }

        fn as_error_mut(&mut self) -> &mut (dyn std::error::Error + Send + Sync + 'static) {
            self
        }

        fn into_error(self: Box<Self>) -> Box<dyn std::error::Error + Send + Sync + 'static> {
            self
        }
    }

    #[test]
    fn test_to_repository_error_row_not_found() {
        let sqlx_error = sqlx::Error::RowNotFound;
        let repo_error = to_repository_error(sqlx_error);
        assert!(matches!(repo_error, RepositoryError::NotFound(_)));
        assert!(
            repo_error
                .to_string()
                .contains("no rows returned by a query that expected to return at least one row")
        );
    }

    #[test]
    fn test_to_repository_error_database_unique_violation() {
        let database_error = MockDatabaseError {
            message: "UNIQUE constraint failed: exchange_rates.currency_pair".to_owned(),
            unique_violation: true,
        };
        let sqlx_error = sqlx::Error::Database(Box::new(database_error));
        let repo_error = to_repository_error(sqlx_error);

        assert!(matches!(repo_error, RepositoryError::Conflict(_)));
    }

    #[test]
    fn test_to_repository_error_database_non_unique_violation() {
        let database_error = MockDatabaseError {
            message: "some generic database error".to_owned(),
            unique_violation: false,
        };
        let sqlx_error = sqlx::Error::Database(Box::new(database_error));
        let repo_error = to_repository_error(sqlx_error);

        assert!(matches!(repo_error, RepositoryError::Storage(_)));
    }

    #[test]
    fn test_to_repository_error_configuration_error() {
        // Test catch-all branch with other error types
        let sqlx_error = sqlx::Error::Configuration("Invalid configuration".into());
        let repo_error = to_repository_error(sqlx_error);

        assert!(matches!(repo_error, RepositoryError::Storage(_)));
        assert!(repo_error.to_string().contains("Invalid configuration"));
    }

    #[test]
    fn test_mock_database_error_as_error() {
        let database_error = MockDatabaseError {
            message: "test error message".to_owned(),
            unique_violation: false,
        };

        // Test as_error returns a valid error trait object reference
        let error_trait: &(dyn std::error::Error + Send + Sync + 'static) =
            database_error.as_error();
        assert_eq!(error_trait.to_string(), "test error message");
    }

    #[test]
    fn test_mock_database_error_as_error_mut() {
        let mut database_error = MockDatabaseError {
            message: "mutable test error".to_owned(),
            unique_violation: false,
        };

        // Test as_error_mut returns a valid mutable error trait object reference
        let error_trait_mut: &mut (dyn std::error::Error + Send + Sync + 'static) =
            database_error.as_error_mut();
        assert_eq!(error_trait_mut.to_string(), "mutable test error");
    }

    #[test]
    fn test_mock_database_error_into_error() {
        let database_error = MockDatabaseError {
            message: "consuming test error".to_owned(),
            unique_violation: true,
        };

        let boxed_error: Box<dyn std::error::Error + Send + Sync + 'static> =
            Box::new(database_error).into_error();
        assert_eq!(boxed_error.to_string(), "consuming test error");
    }

    #[test]
    fn test_mock_database_error_as_error_preserves_message() {
        let database_error = MockDatabaseError {
            message: "UNIQUE constraint failed: table.column".to_owned(),
            unique_violation: true,
        };

        let error_trait = database_error.as_error();
        assert_eq!(
            error_trait.to_string(),
            "UNIQUE constraint failed: table.column"
        );
    }

    #[test]
    fn test_mock_database_error_trait_object_polymorphism() {
        let database_error = MockDatabaseError {
            message: "polymorphic error test".to_owned(),
            unique_violation: false,
        };

        // Verify the error trait object can be used polymorphically
        let error_trait: &(dyn std::error::Error + Send + Sync + 'static) =
            database_error.as_error();

        // Verify we can call standard Error trait methods
        assert!(error_trait.source().is_none()); // MockDatabaseError has no source
        assert!(!error_trait.to_string().is_empty());
    }

    #[test]
    fn test_mock_database_error_message() {
        let database_error = MockDatabaseError {
            message: "message preserved test".to_owned(),
            unique_violation: false,
        };

        assert_eq!(database_error.message(), "message preserved test");
    }
}
