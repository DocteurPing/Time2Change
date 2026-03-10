use chrono::{DateTime, Duration, Utc};
use domain::types::currency::Currency;
use domain::types::currency_pair::CurrencyPair;
use domain::types::exchange_rate::ExchangeRate;

pub(crate) fn make_pair() -> CurrencyPair {
    CurrencyPair::new(Currency::new("EUR").unwrap(), Currency::new("USD").unwrap()).unwrap()
}

pub(crate) fn make_rate(ts: DateTime<Utc>, rate: rust_decimal::Decimal) -> ExchangeRate {
    ExchangeRate::new(ts, rate)
}

/// Build a series of evenly-spaced rates starting `days_ago` days before now.
pub(crate) fn build_rates(values: &[rust_decimal::Decimal], days_ago: i64) -> Vec<ExchangeRate> {
    let now = Utc::now();
    let start = now - Duration::days(days_ago);
    let step = if values.len() <= 1 {
        Duration::hours(1)
    } else {
        Duration::days(days_ago) / i32::try_from(values.len() - 1).unwrap_or_default()
    };

    values
        .iter()
        .enumerate()
        .map(|(i, v)| make_rate(start + step * i32::try_from(i).unwrap_or_default(), *v))
        .collect()
}
