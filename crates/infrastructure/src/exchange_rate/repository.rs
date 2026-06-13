use std::collections::{BTreeMap, HashMap};
use std::ops::RangeInclusive;

use application::ports::exchange_rate_repository::ExchangeRateRepository;
use application::ports::repository_errors::RepositoryError;
use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use domain::types::currency_pair::CurrencyPair;
use domain::types::exchange_rate::ExchangeRate;
use domain::types::time_series::TimeSeries;
use rust_decimal::Decimal;
use sqlx::PgPool;
use sqlx::migrate::MigrateError;

use super::model::ExchangeRateRow;
use crate::repository_error::to_repository_error;

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

fn normalize_timestamp(timestamp: DateTime<Utc>) -> DateTime<Utc> {
    let seconds = timestamp.timestamp();
    let nanos = timestamp.timestamp_subsec_micros() * 1_000;
    Utc.timestamp_opt(seconds, nanos)
        .single()
        .unwrap_or(timestamp)
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
        rates: HashMap<CurrencyPair, Vec<ExchangeRate>>,
    ) -> Result<(), RepositoryError> {
        if rates.is_empty() {
            return Ok(());
        }
        for (pair, rate) in rates {
            let timestamps: Vec<DateTime<Utc>> = rate
                .iter()
                .map(|r| normalize_timestamp(*r.timestamp()))
                .collect();
            let rates: Vec<Decimal> = rate.iter().map(|r| *r.rate()).collect();
            sqlx::query_file!(
                "queries/rates_insert.sql",
                pair.base().to_string(),
                pair.quote().to_string(),
                &timestamps,
                &rates,
            )
            .execute(&self.pool)
            .await
            .map_err(to_repository_error)?;
        }

        Ok(())
    }

    async fn load_rates(
        &self,
        pair: &CurrencyPair,
        range: &RangeInclusive<DateTime<Utc>>,
    ) -> Result<TimeSeries, RepositoryError> {
        let start = normalize_timestamp(*range.start());
        let end = normalize_timestamp(*range.end());

        let rows: Vec<ExchangeRateRow> = sqlx::query_file_as!(
            ExchangeRateRow,
            "queries/rates_load.sql",
            pair.base().to_string(),
            pair.quote().to_string(),
            start,
            end
        )
        .fetch_all(&self.pool)
        .await
        .map_err(to_repository_error)?;

        let rates: BTreeMap<DateTime<Utc>, Decimal> = rows
            .into_iter()
            .map(|row| (row.timestamp, row.rate))
            .collect();
        Ok(TimeSeries::new(pair.clone(), rates))
    }

    async fn exists(
        &self,
        pair: &CurrencyPair,
        range: &RangeInclusive<DateTime<Utc>>,
    ) -> Result<bool, RepositoryError> {
        let start = normalize_timestamp(*range.start());
        let end = normalize_timestamp(*range.end());

        let exists: Option<bool> = sqlx::query_file_scalar!(
            "queries/rates_exist.sql",
            pair.base().to_string(),
            pair.quote().to_string(),
            start,
            end,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(to_repository_error)?;

        Ok(exists.unwrap_or(false))
    }
}
