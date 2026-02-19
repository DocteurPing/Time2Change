use rust_decimal::{Decimal, dec};

use crate::{
    indicators::{
        math::{average, clamp_0_100, median_i64, standard_deviation, z_score},
        quality::{
            rate_quality::{RateQuality, RateQualityBreakdown},
            rate_quality_config::RateQualityConfig,
        },
    },
    types::time_series::TimeSeries,
};

pub fn calculate_rate_quality(values: &TimeSeries, config: &RateQualityConfig) -> RateQuality {
    let rates = values.rates();

    if rates.is_empty() {
        return RateQuality {
            overall: Decimal::ZERO,
            breakdown: RateQualityBreakdown {
                completeness: Decimal::ZERO,
                gap_consistency: Decimal::ZERO,
                outlier: Decimal::ZERO,
                volatility: Decimal::ZERO,
            },
        };
    }

    let series_values: Vec<Decimal> = rates.iter().map(|r| *r.rate()).collect();

    let mut gaps_seconds: Vec<i64> = Vec::new();
    for window in rates.windows(2) {
        let gap = (*window[1].timestamp() - *window[0].timestamp())
            .num_seconds()
            .abs();
        gaps_seconds.push(gap);
    }

    let completeness = if gaps_seconds.is_empty() {
        dec!(100)
    } else {
        let total_duration = (*rates.last().unwrap().timestamp()
            - *rates.first().unwrap().timestamp())
        .num_seconds()
        .abs();
        let typical_gap = median_i64(gaps_seconds.clone()).unwrap_or(0);
        if total_duration == 0 || typical_gap <= 0 {
            dec!(100)
        } else {
            let expected_count = total_duration / typical_gap + 1;
            let expected_count_dec = Decimal::from(expected_count);
            let observed_count_dec = Decimal::from(rates.len() as i64);
            clamp_0_100(observed_count_dec / expected_count_dec * dec!(100))
        }
    };

    let gap_consistency = if gaps_seconds.is_empty() {
        dec!(100)
    } else {
        let gaps_dec: Vec<Decimal> = gaps_seconds.iter().map(|g| Decimal::from(*g)).collect();
        let mean_gap = average(&gaps_dec).unwrap_or(Decimal::ZERO);
        let std_gap = standard_deviation(&gaps_dec).unwrap_or(Decimal::ZERO);
        if mean_gap == Decimal::ZERO {
            dec!(100)
        } else {
            clamp_0_100(mean_gap / (mean_gap + std_gap) * dec!(100))
        }
    };

    let outlier = if series_values.is_empty() {
        Decimal::ZERO
    } else {
        let mean = average(&series_values);
        let std_dev = standard_deviation(&series_values);
        match (mean, std_dev) {
            (Some(mean), Some(std_dev)) if std_dev != Decimal::ZERO => {
                let outliers = series_values
                    .iter()
                    .filter(|v| {
                        z_score(**v, mean, std_dev)
                            .map(|z| z.abs() > config.outlier_z_threshold)
                            .unwrap_or(false)
                    })
                    .count();
                let outlier_ratio =
                    Decimal::from(outliers as i64) / Decimal::from(series_values.len() as i64);
                clamp_0_100((Decimal::ONE - outlier_ratio) * dec!(100))
            }
            _ => dec!(100),
        }
    };

    // Compute percentage returns
    let mut returns: Vec<Decimal> = Vec::new();
    for window in series_values.windows(2) {
        let prev = window[0];
        let curr = window[1];
        if prev != Decimal::ZERO {
            returns.push((curr - prev) / prev);
        }
    }

    let volatility = if returns.is_empty() {
        dec!(100) // No returns to measure → consider perfectly stable
    } else {
        let std_returns = standard_deviation(&returns).unwrap_or(Decimal::ZERO);
        clamp_0_100(dec!(100) / (Decimal::ONE + config.max_allowed_volatility * std_returns))
    };

    let overall = clamp_0_100(
        config.w_completeness * completeness
            + config.w_gap_consistency * gap_consistency
            + config.w_outlier * outlier
            + config.w_volatility * volatility,
    );

    RateQuality {
        overall,
        breakdown: RateQualityBreakdown {
            completeness,
            gap_consistency,
            outlier,
            volatility,
        },
    }
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
    let quality = calculate_rate_quality(&time_series, &config);
    assert_eq!(quality.overall, Decimal::ZERO);
    assert_eq!(quality.breakdown.completeness, Decimal::ZERO);
    assert_eq!(quality.breakdown.gap_consistency, Decimal::ZERO);
    assert_eq!(quality.breakdown.outlier, Decimal::ZERO);
    assert_eq!(quality.breakdown.volatility, Decimal::ZERO);
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
    let quality = calculate_rate_quality(&time_series, &config);
    assert_eq!(quality.overall, dec!(100));
    assert_eq!(quality.breakdown.completeness, dec!(100));
    assert_eq!(quality.breakdown.gap_consistency, dec!(100));
    assert_eq!(quality.breakdown.outlier, dec!(100));
    assert_eq!(quality.breakdown.volatility, dec!(100));
}

#[test]
fn test_rate_quality_with_gap_and_outlier() {
    let config = RateQualityConfig {
        w_completeness: dec!(0.25),
        w_gap_consistency: dec!(0.25),
        w_outlier: dec!(0.25),
        w_volatility: dec!(0.25),
        outlier_z_threshold: dec!(1.0),
        max_allowed_volatility: dec!(1.0),
    };

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
    let result = calculate_rate_quality(&series, &config);

    assert!(result.breakdown.completeness > dec!(85));
    assert!(result.breakdown.gap_consistency > dec!(74));
    assert!(result.breakdown.outlier > dec!(83));
    assert!(result.breakdown.volatility > dec!(80));
    assert!(result.overall > dec!(60));

    series.add_rate(crate::types::exchange_rate::ExchangeRate::new(
        time + chrono::Duration::seconds(420),
        dec!(105),
    ));

    let result2 = calculate_rate_quality(&series, &config);

    assert!(result.breakdown.completeness < result2.breakdown.completeness);
    assert!(result.breakdown.gap_consistency < result2.breakdown.gap_consistency);
    assert!(result.breakdown.outlier < result2.breakdown.outlier);
    assert!(result.breakdown.volatility < result2.breakdown.volatility);
    assert!(result.overall < result2.overall);
}
