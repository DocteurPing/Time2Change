use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

/// A timestamped foreign exchange rate observation.
///
/// This value object represents the price of one unit of a base currency
/// expressed in the quote currency at a precise moment in time.
///
/// Instances of this type are immutable after creation and are intended to be
/// used as atomic data points inside higher-level aggregates such as time
/// series.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ExchangeRate {
    timestamp: DateTime<Utc>,
    rate: Decimal,
}

impl ExchangeRate {
    /// Creates a new exchange rate observation.
    ///
    /// `timestamp` identifies when the rate was observed and `rate` stores the
    /// numeric exchange value captured at that moment.
    #[must_use]
    pub const fn new(timestamp: DateTime<Utc>, rate: Decimal) -> Self {
        Self { timestamp, rate }
    }

    /// Returns the timestamp associated with this rate observation.
    #[must_use]
    pub const fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }

    /// Returns the exchange rate value.
    #[must_use]
    pub const fn rate(&self) -> &Decimal {
        &self.rate
    }

    /// Returns the timestamp and rate as separate values.
    pub fn into_parts(self) -> (DateTime<Utc>, Decimal) {
        (self.timestamp, self.rate)
    }
}

impl std::fmt::Display for ExchangeRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.timestamp, self.rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exchange_rate_display() {
        let timestamp = Utc::now();
        let rate = rust_decimal::dec!(1.2345);
        let exchange_rate = ExchangeRate::new(timestamp, rate);
        assert_eq!(exchange_rate.timestamp(), &timestamp);
        assert_eq!(exchange_rate.rate(), &rate);
        assert_eq!(format!("{exchange_rate}"), format!("{timestamp}: {rate}"));
    }

    #[test]
    fn test_into_parts() {
        let timestamp = Utc::now();
        let rate = rust_decimal::dec!(1.2345);
        let exchange_rate = ExchangeRate::new(timestamp, rate);
        assert_eq!(exchange_rate.into_parts(), (timestamp, rate));
    }
}
