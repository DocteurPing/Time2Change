use std::collections::BTreeMap;

use chrono::Utc;
use rust_decimal::{Decimal, dec};

use crate::types::currency::Currency;
use crate::types::currency_pair::CurrencyPair;
use crate::types::exchange_rate::ExchangeRate;
use crate::types::rate_quality_config::{
    RateQualityConfig, RateQualityThresholds, RateQualityWeights,
};
use crate::types::time_series::TimeSeries;

#[test]
fn test_time_series_display() {
    let pair = CurrencyPair::new("USD".try_into().unwrap(), "EUR".try_into().unwrap()).unwrap();
    let pair_string = pair.to_string();
    let time = Utc::now();
    let time2 = time + chrono::Duration::milliseconds(1);
    let rates = BTreeMap::from([
        (time, rust_decimal::dec!(1.2345)),
        (time2, rust_decimal::dec!(1.2346)),
    ]);
    let time_series = TimeSeries::new(pair, rates.clone());
    assert_eq!(time_series.pair().to_string(), pair_string);
    assert_eq!(time_series.rates(), &rates);
    assert_eq!(
        format!("{time_series}"),
        format!("TimeSeries({pair_string}, [{time}: 1.2345, {time2}: 1.2346])")
    );
}

#[test]
fn test_time_series_add_rate() {
    let pair = CurrencyPair::new("USD".try_into().unwrap(), "EUR".try_into().unwrap()).unwrap();
    let time = Utc::now();
    let mut time_series = TimeSeries::new(pair, BTreeMap::new());
    time_series.add_rate(time, rust_decimal::dec!(1.2345));
    assert_eq!(time_series.rates().len(), 1);
    assert_eq!(time_series.rates().get(&time), Some(&dec!(1.2345)));
}

#[test]
fn test_calculate_rate_quality_empty() {
    let time_series = TimeSeries::new(
        CurrencyPair::new("USD".try_into().unwrap(), "EUR".try_into().unwrap()).unwrap(),
        BTreeMap::new(),
    );
    let config = RateQualityConfig::default();
    let quality = time_series.calculate_rate_quality(&config);
    assert_eq!(*quality.overall(), Decimal::ZERO);
    assert_eq!(*quality.breakdown().completeness(), Decimal::ZERO);
    assert_eq!(*quality.breakdown().gap_consistency(), Decimal::ZERO);
    assert_eq!(*quality.breakdown().outlier(), Decimal::ZERO);
    assert_eq!(*quality.breakdown().volatility(), Decimal::ZERO);
}

#[test]
fn test_calculate_rate_quality_perfect() {
    let time = Utc::now();
    let time_series = TimeSeries::new(
        CurrencyPair::new("USD".try_into().unwrap(), "EUR".try_into().unwrap()).unwrap(),
        BTreeMap::from([
            (time, dec!(1.0)),
            (time + chrono::Duration::seconds(60), dec!(1.0)),
            (time + chrono::Duration::seconds(120), dec!(1.0)),
        ]),
    );
    let config = RateQualityConfig::default();
    let quality = time_series.calculate_rate_quality(&config);
    assert_eq!(*quality.overall(), dec!(100));
    assert_eq!(*quality.breakdown().completeness(), dec!(100));
    assert_eq!(*quality.breakdown().gap_consistency(), dec!(100));
    assert_eq!(*quality.breakdown().outlier(), dec!(100));
    assert_eq!(*quality.breakdown().volatility(), dec!(100));
}

#[test]
fn test_rate_quality_with_gap_and_outlier() {
    let config = RateQualityConfig::new(
        RateQualityWeights::default(),
        RateQualityThresholds::new(dec!(1.0), dec!(1.0)).unwrap(),
    );

    let time = Utc::now();

    let mut series = TimeSeries::new(
        CurrencyPair::new("USD".try_into().unwrap(), "EUR".try_into().unwrap()).unwrap(),
        BTreeMap::from([
            (time, dec!(100)),
            (time + chrono::Duration::seconds(60), dec!(101)),
            (time + chrono::Duration::seconds(120), dec!(102)),
            (time + chrono::Duration::seconds(180), dec!(103)),
            (time + chrono::Duration::seconds(300), dec!(150)), // outlier + gap
            (time + chrono::Duration::seconds(360), dec!(104)),
        ]),
    );
    let result = series.calculate_rate_quality(&config);

    assert!(*result.breakdown().completeness() > dec!(85));
    assert!(*result.breakdown().gap_consistency() > dec!(74));
    assert!(*result.breakdown().outlier() > dec!(83));
    assert!(*result.breakdown().volatility() > dec!(80));
    assert!(*result.overall() > dec!(60));

    series.add_rate(time + chrono::Duration::seconds(420), dec!(105));

    let result2 = series.calculate_rate_quality(&config);

    assert!(result.breakdown().completeness() < result2.breakdown().completeness());
    assert!(result.breakdown().gap_consistency() < result2.breakdown().gap_consistency());
    assert!(result.breakdown().outlier() < result2.breakdown().outlier());
    assert!(result.breakdown().volatility() < result2.breakdown().volatility());
    assert!(result.overall() < result2.overall());
}

#[test]
fn test_lowest_value_non_empty() {
    let time = Utc::now();
    let values = BTreeMap::from([
        (time, dec!(5)),
        (time + chrono::Duration::seconds(1), dec!(2)),
        (time + chrono::Duration::seconds(2), dec!(8)),
    ]);
    let currency_pair = CurrencyPair::new(
        Currency::try_from("USD").unwrap(),
        Currency::try_from("EUR").unwrap(),
    )
    .unwrap();
    let time_series = TimeSeries::new(currency_pair, values);
    let result = time_series.lowest_value();
    assert_eq!(result, Some(&dec!(2)));
}

#[test]
fn test_lowest_value_empty() {
    let currency_pair = CurrencyPair::new(
        Currency::try_from("USD").unwrap(),
        Currency::try_from("EUR").unwrap(),
    )
    .unwrap();
    let time_series = TimeSeries::new(currency_pair, BTreeMap::new());
    let result = time_series.lowest_value();
    assert_eq!(result, None);
}

#[test]
fn test_lowest_value_all_equal() {
    let time = Utc::now();
    let values = BTreeMap::from([
        (time, dec!(3)),
        (time + chrono::Duration::seconds(1), dec!(3)),
        (time + chrono::Duration::seconds(2), dec!(3)),
    ]);
    let currency_pair = CurrencyPair::new(
        Currency::try_from("USD").unwrap(),
        Currency::try_from("EUR").unwrap(),
    )
    .unwrap();
    let time_series = TimeSeries::new(currency_pair, values);
    let result = time_series.lowest_value();
    assert_eq!(result, Some(&dec!(3)));
}

#[test]
fn test_highest_value_non_empty() {
    let time = Utc::now();
    let values = BTreeMap::from([
        (time, dec!(5)),
        (time + chrono::Duration::seconds(1), dec!(2)),
        (time + chrono::Duration::seconds(2), dec!(8)),
    ]);
    let currency_pair = CurrencyPair::new(
        Currency::try_from("USD").unwrap(),
        Currency::try_from("EUR").unwrap(),
    )
    .unwrap();
    let time_series = TimeSeries::new(currency_pair, values);
    let result = time_series.highest_value();
    assert_eq!(result, Some(&dec!(8)));
}

#[test]
fn test_highest_value_empty() {
    let currency_pair = CurrencyPair::new(
        Currency::try_from("USD").unwrap(),
        Currency::try_from("EUR").unwrap(),
    )
    .unwrap();
    let time_series = TimeSeries::new(currency_pair, BTreeMap::new());
    let result = time_series.highest_value();
    assert_eq!(result, None);
}

#[test]
fn test_highest_value_all_equal() {
    let time = Utc::now();
    let values = BTreeMap::from([
        (time, dec!(3)),
        (time + chrono::Duration::seconds(1), dec!(3)),
        (time + chrono::Duration::seconds(2), dec!(3)),
    ]);
    let currency_pair = CurrencyPair::new(
        Currency::try_from("USD").unwrap(),
        Currency::try_from("EUR").unwrap(),
    )
    .unwrap();
    let time_series = TimeSeries::new(currency_pair, values);
    let result = time_series.highest_value();
    assert_eq!(result, Some(&dec!(3)));
}

#[test]
fn extend_rates() {
    let time = Utc::now();
    let values = BTreeMap::from([
        (time, dec!(5)),
        (time + chrono::Duration::seconds(1), dec!(2)),
    ]);
    let currency_pair = CurrencyPair::new(
        Currency::try_from("USD").unwrap(),
        Currency::try_from("EUR").unwrap(),
    )
    .unwrap();
    let mut time_series = TimeSeries::new(currency_pair, values);
    time_series.extend_rates(&[
        ExchangeRate::new(time + chrono::Duration::seconds(2), dec!(8)),
        ExchangeRate::new(time + chrono::Duration::seconds(3), dec!(3)),
    ]);
    assert_eq!(time_series.rates().len(), 4);
    let result = time_series.highest_value();
    assert_eq!(result, Some(&dec!(8)));
}
