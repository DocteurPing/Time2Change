use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

#[derive(Debug, PartialEq, Clone)]
pub struct ExchangeRate {
    timestamp: DateTime<Utc>,
    rate: Decimal,
}

impl ExchangeRate {
    pub const fn new(timestamp: DateTime<Utc>, rate: Decimal) -> Self {
        Self { timestamp, rate }
    }

    pub const fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }

    pub const fn rate(&self) -> &Decimal {
        &self.rate
    }
}

impl std::fmt::Display for ExchangeRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.timestamp, self.rate)
    }
}

#[test]
fn test_exchange_rate_display() {
    let timestamp = Utc::now();
    let rate = rust_decimal::dec!(1.2345);
    let exchange_rate = ExchangeRate::new(timestamp, rate);
    assert_eq!(exchange_rate.timestamp(), &timestamp);
    assert_eq!(exchange_rate.rate(), &rate);
    assert_eq!(format!("{exchange_rate}"), format!("{timestamp}: {rate}"));
}
