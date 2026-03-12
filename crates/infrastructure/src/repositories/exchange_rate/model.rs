use chrono::{DateTime, Utc};
use domain::types::exchange_rate::ExchangeRate;
use rust_decimal::Decimal;

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
    pub(crate) timestamp: DateTime<Utc>,
    pub(crate) rate: Decimal,
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
}
