use application::ports::exchange_rate_repository::RepositoryError;
use chrono::{DateTime, Utc};
use domain::types::currency::Currency;
use domain::types::currency_info::CurrencyInfo;
use domain::types::exchange_rate::ExchangeRate;
use rust_decimal::Decimal;

use crate::repositories::exchange_rate::error::to_invalid_error;

/// Raw database row returned by the `exchange_rates` table.
///
/// The field types map directly to the Postgres column types:
/// - `timestamp` is `TIMESTAMPTZ` — sqlx decodes it as `DateTime<Utc>` natively
/// - `rate` is `NUMERIC` — sqlx decodes it as `Decimal` natively
///
/// The `base` and `quote` columns are intentionally **not** included here:
/// they are used as query filters but are already known at the call site
/// (from the [`CurrencyPair`] argument), so we avoid redundantly re-parsing
/// them for every row.
#[derive(Debug, sqlx::FromRow)]
pub struct ExchangeRateRow {
    timestamp: DateTime<Utc>,
    rate: Decimal,
}

/// Raw database row returned by the `currencies` table.
///
/// Both columns are decoded as plain `String` values by sqlx — validation
/// into domain types happens at the infrastructure boundary via [`TryFrom`],
/// keeping the `domain` crate free of any sqlx dependency.
///
/// The field types map directly to the Postgres column types:
/// - `currency` is `TEXT` — sqlx decodes it as `String` natively
/// - `name` is `TEXT` — sqlx decodes it as `String` natively
#[derive(Debug, sqlx::FromRow)]
pub struct CurrencyInfoRow {
    currency: String,
    name: String,
}

impl TryFrom<CurrencyInfoRow> for CurrencyInfo {
    type Error = RepositoryError;

    /// Converts a raw DB row into the domain [`CurrencyInfo`] type.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryError::Invalid`] if the stored currency code
    /// fails [`Currency`] validation (not a 3-letter uppercase ASCII string).
    /// This should never happen with well-formed data but is handled
    /// explicitly so that corrupt rows surface as a clear error rather than
    /// a panic.
    fn try_from(row: CurrencyInfoRow) -> Result<Self, Self::Error> {
        let code = Currency::new(&row.currency).map_err(|e| to_invalid_error(&e.to_string()))?;
        Ok(Self::new(code, row.name))
    }
}

impl From<ExchangeRateRow> for ExchangeRate {
    /// Converts a raw DB row into the domain [`ExchangeRate`] type.
    ///
    /// This conversion is infallible because `sqlx` has already validated
    /// the types when decoding the row — `TIMESTAMPTZ` → `DateTime<Utc>`
    /// and `NUMERIC` → `Decimal` are handled by the driver.
    fn from(row: ExchangeRateRow) -> Self {
        Self::new(row.timestamp, row.rate)
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::dec;

    use super::*;

    #[test]
    fn exchange_rate_row_converts_to_domain_type() {
        let now = Utc::now();
        let row = ExchangeRateRow {
            timestamp: now,
            rate: dec!(1.0850),
        };
        let rate = ExchangeRate::from(row);
        assert_eq!(rate.timestamp(), &now);
        assert_eq!(rate.rate(), &dec!(1.0850));
    }

    #[test]
    fn currency_info_row_converts_to_domain_type() {
        let row = CurrencyInfoRow {
            currency: "EUR".to_owned(),
            name: "Euro".to_owned(),
        };
        let info = CurrencyInfo::try_from(row).unwrap();
        assert_eq!(info.code().to_string(), "EUR");
        assert_eq!(info.name(), "Euro");
    }

    #[test]
    fn currency_info_row_invalid_code_returns_error() {
        let row = CurrencyInfoRow {
            currency: "invalid".to_owned(),
            name: "Bad Currency".to_owned(),
        };
        let result = CurrencyInfo::try_from(row);
        assert!(result.is_err());
    }
}
