use std::ops::RangeInclusive;

use application::ports::exchange_rate_repository::{ExchangeRateRepository, RepositoryError};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::types::currency_info::CurrencyInfo;
use domain::types::currency_pair::CurrencyPair;
use domain::types::exchange_rate::ExchangeRate;
use domain::types::time_series::TimeSeries;
use rust_decimal::Decimal;
use sqlx::PgPool;
use sqlx::migrate::MigrateError;

use super::model::ExchangeRateRow;
use super::queries;
use crate::exchange_rate::error::to_repository_error;
use crate::exchange_rate::model::CurrencyInfoRow;

/// Postgres-backed implementation of [`ExchangeRateRepository`].
///
/// Wraps a [`PgPool`] and translates between the application port contract
/// and the `exchange_rates` table. All type conversions are handled by
/// `sqlx`'s native `chrono` and `rust_decimal` bindings — no manual
/// string parsing occurs at runtime.
#[derive(Debug, Clone)]
pub struct PostgresExchangeRateRepository {
    pool: PgPool,
}

impl PostgresExchangeRateRepository {
    /// Creates a new repository from an existing connection pool.
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Creates the `exchange_rates` table if it does not already exist.
    ///
    /// Call this once at application startup before accepting any requests.
    ///
    /// # Errors
    ///
    /// Returns [`MigrateError`] if applying the database migrations fails.
    pub async fn migrate(&self) -> Result<(), MigrateError> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }
}

#[async_trait]
impl ExchangeRateRepository for PostgresExchangeRateRepository {
    async fn save_rates(
        &self,
        pair: &CurrencyPair,
        rates: &[ExchangeRate],
    ) -> Result<(), RepositoryError> {
        if rates.is_empty() {
            return Ok(());
        }

        let timestamp: Vec<DateTime<Utc>> = rates.iter().map(|r| *r.timestamp()).collect();
        let rate: Vec<Decimal> = rates.iter().map(|r| *r.rate()).collect();

        sqlx::query(queries::INSERT_RATE)
            .bind(pair.base().to_string())
            .bind(pair.quote().to_string())
            .bind(&timestamp)
            .bind(&rate)
            .execute(&self.pool)
            .await
            .map_err(to_repository_error)?;

        Ok(())
    }

    async fn load_rates(
        &self,
        pair: &CurrencyPair,
        range: &RangeInclusive<DateTime<Utc>>,
    ) -> Result<TimeSeries, RepositoryError> {
        let (start, end) = (range.start(), range.end());

        let rows: Vec<ExchangeRateRow> = sqlx::query_as(queries::LOAD_RATES)
            .bind(pair.base().to_string())
            .bind(pair.quote().to_string())
            .bind(start)
            .bind(end)
            .fetch_all(&self.pool)
            .await
            .map_err(to_repository_error)?;

        let exchange_rates: Vec<ExchangeRate> = rows.into_iter().map(ExchangeRate::from).collect();

        Ok(TimeSeries::new(pair.clone(), exchange_rates))
    }

    async fn exists(
        &self,
        pair: &CurrencyPair,
        range: &RangeInclusive<DateTime<Utc>>,
    ) -> Result<bool, RepositoryError> {
        let (start, end) = (range.start(), range.end());

        let exists: bool = sqlx::query_scalar(queries::EXISTS)
            .bind(pair.base().to_string())
            .bind(pair.quote().to_string())
            .bind(start)
            .bind(end)
            .fetch_one(&self.pool)
            .await
            .map_err(to_repository_error)?;

        Ok(exists)
    }

    async fn save_currencies(&self, currencies: &[CurrencyInfo]) -> Result<(), RepositoryError> {
        if currencies.is_empty() {
            return Ok(());
        }
        let codes: Vec<String> = currencies.iter().map(|c| c.code().to_string()).collect();
        let names: Vec<String> = currencies.iter().map(|c| c.name().to_owned()).collect();

        sqlx::query(queries::SAVE_CURRENCIES)
            .bind(&codes)
            .bind(&names)
            .execute(&self.pool)
            .await
            .map_err(to_repository_error)?;

        Ok(())
    }

    async fn list_currencies(&self) -> Result<Vec<CurrencyInfo>, RepositoryError> {
        let rows: Vec<CurrencyInfoRow> = sqlx::query_as(queries::LOAD_CURRENCIES)
            .fetch_all(&self.pool)
            .await
            .map_err(to_repository_error)?;

        let currencies: Vec<CurrencyInfo> = rows
            .into_iter()
            .map(CurrencyInfo::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(currencies)
    }
}
