use rust_decimal::{Decimal, dec};

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
    let time = chrono::Utc::now();
    let rates = vec![
        ExchangeRate::new(time, rust_decimal::dec!(1.2345)),
        ExchangeRate::new(time, rust_decimal::dec!(1.2346)),
    ];
    let time_series = TimeSeries::new(pair, rates.clone());
    assert_eq!(time_series.pair().to_string(), pair_string);
    assert_eq!(time_series.rates(), &rates);
    assert_eq!(
        format!("{time_series}"),
        format!("TimeSeries({pair_string}, [{time}: 1.2345, {time}: 1.2346])")
    );
}

#[test]
fn test_time_series_add_rate() {
    let pair = CurrencyPair::new("USD".try_into().unwrap(), "EUR".try_into().unwrap()).unwrap();
    let time = chrono::Utc::now();
    let rate = ExchangeRate::new(time, rust_decimal::dec!(1.2345));
    let mut time_series = TimeSeries::new(pair, vec![]);
    time_series.add_rate(rate.clone());
    assert_eq!(time_series.rates().len(), 1);
    assert_eq!(time_series.rates(), &[rate]);
}

#[test]
fn test_calculate_rate_quality_empty() {
    let time_series = TimeSeries::new(
        crate::types::currency_pair::CurrencyPair::new(
            "USD".try_into().unwrap(),
            "EUR".try_into().unwrap(),
        )
        .unwrap(),
        vec![],
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
    let time = chrono::Utc::now();
    let time_series = TimeSeries::new(
        crate::types::currency_pair::CurrencyPair::new(
            "USD".try_into().unwrap(),
            "EUR".try_into().unwrap(),
        )
        .unwrap(),
        vec![
            crate::types::exchange_rate::ExchangeRate::new(time, dec!(1.0)),
            crate::types::exchange_rate::ExchangeRate::new(
                time + chrono::Duration::seconds(60),
                dec!(1.0),
            ),
            crate::types::exchange_rate::ExchangeRate::new(
                time + chrono::Duration::seconds(120),
                dec!(1.0),
            ),
        ],
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

    let time = chrono::Utc::now();

    let mut series = TimeSeries::new(
        crate::types::currency_pair::CurrencyPair::new(
            "USD".try_into().unwrap(),
            "EUR".try_into().unwrap(),
        )
        .unwrap(),
        vec![
            crate::types::exchange_rate::ExchangeRate::new(time, dec!(100)),
            crate::types::exchange_rate::ExchangeRate::new(
                time + chrono::Duration::seconds(60),
                dec!(101),
            ),
            crate::types::exchange_rate::ExchangeRate::new(
                time + chrono::Duration::seconds(120),
                dec!(102),
            ),
            crate::types::exchange_rate::ExchangeRate::new(
                time + chrono::Duration::seconds(180),
                dec!(103),
            ),
            crate::types::exchange_rate::ExchangeRate::new(
                time + chrono::Duration::seconds(300),
                dec!(150),
            ), // outlier
            crate::types::exchange_rate::ExchangeRate::new(
                time + chrono::Duration::seconds(360),
                dec!(104),
            ),
        ],
    );
    let result = series.calculate_rate_quality(&config);

    assert!(*result.breakdown().completeness() > dec!(85));
    assert!(*result.breakdown().gap_consistency() > dec!(74));
    assert!(*result.breakdown().outlier() > dec!(83));
    assert!(*result.breakdown().volatility() > dec!(80));
    assert!(*result.overall() > dec!(60));

    series.add_rate(crate::types::exchange_rate::ExchangeRate::new(
        time + chrono::Duration::seconds(420),
        dec!(105),
    ));

    let result2 = series.calculate_rate_quality(&config);

    assert!(result.breakdown().completeness() < result2.breakdown().completeness());
    assert!(result.breakdown().gap_consistency() < result2.breakdown().gap_consistency());
    assert!(result.breakdown().outlier() < result2.breakdown().outlier());
    assert!(result.breakdown().volatility() < result2.breakdown().volatility());
    assert!(result.overall() < result2.overall());
}
