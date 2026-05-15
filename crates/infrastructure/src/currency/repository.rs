use application::ports::currency_repository::CurrencyRepository;
use application::ports::repository_errors::RepositoryError;
use async_trait::async_trait;
use domain::types::currency_info::CurrencyInfo;
use sqlx::PgPool;

use super::model::CurrencyInfoRow;
use crate::repository_error::to_repository_error;

/// Postgres-backed implementation of [`CurrencyRepository`].
///
/// Wraps a [`PgPool`] and translates between the application port contract
/// and the `currencies` table.
#[derive(Debug, Clone)]
pub struct PostgresCurrencyRepository {
    pool: PgPool,
}

impl PostgresCurrencyRepository {
    /// Creates a new currency repository from an existing connection pool.
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CurrencyRepository for PostgresCurrencyRepository {
    async fn save_currencies(&self, currencies: &[CurrencyInfo]) -> Result<(), RepositoryError> {
        if currencies.is_empty() {
            return Ok(());
        }

        let codes: Vec<String> = currencies.iter().map(|c| c.code().to_string()).collect();
        let names: Vec<String> = currencies.iter().map(|c| c.name().to_owned()).collect();

        sqlx::query_file!("queries/currencies_save.sql", &codes, &names)
            .execute(&self.pool)
            .await
            .map_err(to_repository_error)?;

        Ok(())
    }

    async fn list_currencies(&self) -> Result<Vec<CurrencyInfo>, RepositoryError> {
        let rows: Vec<CurrencyInfoRow> =
            sqlx::query_file_as!(CurrencyInfoRow, "queries/currencies_load.sql")
                .fetch_all(&self.pool)
                .await
                .map_err(to_repository_error)?;

        rows.into_iter().map(CurrencyInfo::try_from).collect()
    }
}
