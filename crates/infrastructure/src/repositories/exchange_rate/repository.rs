use std::ops::RangeInclusive;

use application::ports::exchange_rate_repository::{ExchangeRateRepository, RepositoryError};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::types::currency_pair::CurrencyPair;
use domain::types::exchange_rate::ExchangeRate;
use domain::types::time_series::TimeSeries;
use sqlx::PgPool;

use super::model::ExchangeRateRow;
use super::queries;
use crate::repositories::exchange_rate::error::to_repository_error;

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
    /// Returns [`RepositoryError::Storage`] if the DDL statement fails.
    pub async fn migrate(&self) -> Result<(), RepositoryError> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| RepositoryError::Storage(e.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl ExchangeRateRepository for PostgresExchangeRateRepository {
    /// Persists a batch of exchange rates for the given currency pair.
    ///
    /// All rates are being saved in a single transaction.
    /// Duplicate `(base, quote, timestamp)` triples are silently ignored
    /// (`ON CONFLICT DO NOTHING`).
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError::Storage`] if the transaction or any
    /// individual insert fails.
    async fn save_rates(
        &self,
        pair: &CurrencyPair,
        rates: &[ExchangeRate],
    ) -> Result<(), RepositoryError> {
        if rates.is_empty() {
            return Ok(());
        }

        let timestamp: Vec<DateTime<Utc>> = rates.iter().map(|r| *r.timestamp()).collect();
        let rate: Vec<rust_decimal::Decimal> = rates.iter().map(|r| *r.rate()).collect();

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

    /// Loads all stored rates for the pair within the inclusive time range.
    ///
    /// Rows are returned in ascending timestamp order.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError::Storage`] if the query fails.
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

    /// Returns whether at least one rate exists for the pair in the range.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError::Storage`] if the query fails.
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
}

#[cfg(test)]
mod tests {
    use application::ports::exchange_rate_repository::ExchangeRateRepository;
    use chrono::Utc;
    use domain::types::currency::Currency;
    use domain::types::currency_pair::CurrencyPair;
    use domain::types::exchange_rate::ExchangeRate;
    use rust_decimal::dec;
    use sqlx::PgPool;

    use super::PostgresExchangeRateRepository;

    fn make_pair() -> CurrencyPair {
        CurrencyPair::new(Currency::new("EUR").unwrap(), Currency::new("USD").unwrap()).unwrap()
    }

    // sqlx::test automatically spins up a temporary database,
    // runs migrations, and tears it down after the test.
    #[sqlx::test]
    async fn save_and_load_round_trip(pool: PgPool) {
        let repo = PostgresExchangeRateRepository::new(pool);
        repo.migrate().await.unwrap();

        let now = Utc::now();
        let pair = make_pair();
        let rate = ExchangeRate::new(now, dec!(1.0850));

        repo.save_rates(&pair, &[rate]).await.unwrap();

        let range = (now - chrono::Duration::seconds(1))..=(now + chrono::Duration::seconds(1));
        let series = repo.load_rates(&pair, &range).await.unwrap();

        assert_eq!(series.rates().len(), 1);
        assert_eq!(series.rates()[0].rate(), &dec!(1.0850));

        assert!(repo.exists(&pair, &range).await.unwrap());
    }

    #[sqlx::test]
    async fn exists_no_data(pool: PgPool) {
        let repo = PostgresExchangeRateRepository::new(pool);
        repo.migrate().await.unwrap();
        assert!(
            !repo
                .exists(&make_pair(), &(Utc::now()..=Utc::now()))
                .await
                .unwrap()
        );
    }

    #[sqlx::test]
    async fn save_multiple_rates(pool: PgPool) {
        let repo = PostgresExchangeRateRepository::new(pool);
        repo.migrate().await.unwrap();

        let now = Utc::now();
        let pair = make_pair();
        let rates = vec![
            ExchangeRate::new(now, dec!(1.0850)),
            ExchangeRate::new(now + chrono::Duration::seconds(60), dec!(1.0860)),
            ExchangeRate::new(now + chrono::Duration::seconds(120), dec!(1.0870)),
        ];

        repo.save_rates(&pair, &rates).await.unwrap();

        let range = (now - chrono::Duration::seconds(1))..=(now + chrono::Duration::seconds(120));
        let series = repo.load_rates(&pair, &range).await.unwrap();

        assert_eq!(series.rates().len(), 3);
        assert_eq!(series.rates()[0].rate(), &dec!(1.0850));
        assert_eq!(series.rates()[1].rate(), &dec!(1.0860));
        assert_eq!(series.rates()[2].rate(), &dec!(1.0870));
    }

    #[sqlx::test]
    async fn save_duplicate_rates(pool: PgPool) {
        let repo = PostgresExchangeRateRepository::new(pool);
        repo.migrate().await.unwrap();

        let now = Utc::now();
        let pair = make_pair();
        let rate1 = ExchangeRate::new(now, dec!(1.0850));
        let rate2 = ExchangeRate::new(now, dec!(1.0860)); // same timestamp, different rate

        repo.save_rates(&pair, std::slice::from_ref(&rate1))
            .await
            .unwrap();
        repo.save_rates(&pair, std::slice::from_ref(&rate2))
            .await
            .unwrap(); // should be ignored

        let range = (now - chrono::Duration::seconds(1))..=(now + chrono::Duration::seconds(1));
        let series = repo.load_rates(&pair, &range).await.unwrap();

        assert_eq!(series.rates().len(), 1);
        assert_eq!(series.rates()[0].rate(), &dec!(1.0850)); // original rate should be preserved
    }

    #[sqlx::test]
    async fn save_duplicate_in_batch(pool: PgPool) {
        let repo = PostgresExchangeRateRepository::new(pool);
        repo.migrate().await.unwrap();

        let now = Utc::now();
        let pair = make_pair();
        let rate1 = ExchangeRate::new(now, dec!(1.0850));
        let rate2 = ExchangeRate::new(now, dec!(1.0860)); // same timestamp, different rate

        repo.save_rates(&pair, &[rate1.clone(), rate2.clone()])
            .await
            .unwrap(); // one of them should be ignored

        let range = (now - chrono::Duration::seconds(1))..=(now + chrono::Duration::seconds(1));
        let series = repo.load_rates(&pair, &range).await.unwrap();

        assert_eq!(series.rates().len(), 1);
        assert_eq!(series.rates()[0].rate(), &dec!(1.0850)); // original rate should be preserved
    }
}
