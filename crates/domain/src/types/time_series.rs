use rust_decimal::{Decimal, dec};

use crate::indicators::math::{average, clamp_0_100, median_i64, standard_deviation, z_score};
use crate::types::currency_pair::CurrencyPair;
use crate::types::exchange_rate::ExchangeRate;
use crate::types::rate_quality::{RateQuality, RateQualityBreakdown};
use crate::types::rate_quality_config::RateQualityConfig;

pub struct TimeSeries {
    pair: CurrencyPair,
    rates: Vec<ExchangeRate>,
}

impl TimeSeries {
    pub const fn new(pair: CurrencyPair, rates: Vec<ExchangeRate>) -> Self {
        Self { pair, rates }
    }

    pub const fn pair(&self) -> &CurrencyPair {
        &self.pair
    }

    pub fn rates(&self) -> &[ExchangeRate] {
        &self.rates
    }

    pub fn add_rate(&mut self, rate: ExchangeRate) {
        self.rates.push(rate);
    }

    pub fn calculate_rate_quality(&self, config: &RateQualityConfig) -> RateQuality {
        if self.rates().is_empty() {
            return RateQuality::new(
                Decimal::ZERO,
                RateQualityBreakdown::new(
                    Decimal::ZERO,
                    Decimal::ZERO,
                    Decimal::ZERO,
                    Decimal::ZERO,
                ),
            );
        }

        let series_values: Vec<Decimal> = self.rates().iter().map(|r| *r.rate()).collect();

        let mut gaps_seconds: Vec<i64> = Vec::new();
        for window in self.rates().windows(2) {
            let gap = (*window[1].timestamp() - *window[0].timestamp())
                .num_seconds()
                .abs();
            gaps_seconds.push(gap);
        }

        let completeness = if gaps_seconds.is_empty() {
            dec!(100)
        } else {
            let total_duration = (*self.rates().last().unwrap().timestamp()
                - *self.rates().first().unwrap().timestamp())
            .num_seconds()
            .abs();
            let typical_gap = median_i64(gaps_seconds.clone()).unwrap_or(0);
            if total_duration == 0 || typical_gap <= 0 {
                dec!(100)
            } else {
                let expected_count = total_duration / typical_gap + 1;
                let expected_count_dec = Decimal::from(expected_count);
                let observed_count_dec = Decimal::from(self.rates().len());
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
                                .map(|z| z.abs() > config.threshold().outlier_z_threshold())
                                .unwrap_or(false)
                        })
                        .count();
                    let outlier_ratio =
                        Decimal::from(outliers) / Decimal::from(series_values.len());
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
            clamp_0_100(
                dec!(100)
                    / (Decimal::ONE + config.threshold().max_allowed_volatility() * std_returns),
            )
        };

        let overall = clamp_0_100(
            config.weights().completeness() * completeness
                + config.weights().gap_consistency() * gap_consistency
                + config.weights().outlier() * outlier
                + config.weights().volatility() * volatility,
        );

        RateQuality::new(
            overall,
            RateQualityBreakdown::new(completeness, gap_consistency, outlier, volatility),
        )
    }
}

impl std::fmt::Display for TimeSeries {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TimeSeries({}, [", self.pair)?;
        for (i, rate) in self.rates.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{rate}")?;
        }
        write!(f, "])")
    }
}
