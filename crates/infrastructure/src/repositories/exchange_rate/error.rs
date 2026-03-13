use application::ports::exchange_rate_repository::RepositoryError;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn to_repository_error(e: sqlx::Error) -> RepositoryError {
    RepositoryError::Storage(e.to_string())
}

#[test]
fn test_to_repository_error() {
    let sqlx_error = sqlx::Error::RowNotFound;
    let repo_error = to_repository_error(sqlx_error);
    assert!(matches!(repo_error, RepositoryError::Storage(_)));
    assert!(repo_error.to_string().contains("storage failure:"));
}
