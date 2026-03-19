use application::ports::exchange_rate_repository::RepositoryError;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn to_repository_error(e: sqlx::Error) -> RepositoryError {
    RepositoryError::Storage(e.to_string())
}

pub(crate) fn to_invalid_error(msg: &str) -> RepositoryError {
    RepositoryError::Invalid(msg.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_repository_error() {
        let sqlx_error = sqlx::Error::RowNotFound;
        let repo_error = to_repository_error(sqlx_error);
        assert!(matches!(repo_error, RepositoryError::Storage(_)));
        assert!(repo_error.to_string().contains("storage failure:"));
    }

    #[test]
    fn test_to_invalid_error() {
        let repo_error = to_invalid_error("bad currency code: XY");
        assert!(matches!(repo_error, RepositoryError::Invalid(_)));
        assert!(repo_error.to_string().contains("bad currency code: XY"));
    }
}
