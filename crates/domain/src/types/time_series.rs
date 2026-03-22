use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use rust_decimal::{Decimal, dec};

use crate::indicators::math::{average, clamp_0_100, median_i64, standard_deviation, z_score};
use crate::types::currency_pair::CurrencyPair;
use crate::types::exchange_rate::ExchangeRate;
use crate::types::rate_quality::{RateQuality, RateQualityBreakdown};
use crate::types::rate_quality_config::RateQualityConfig;

/// Represents a time series of exchange rates for a specific currency pair.
///
/// A `TimeSeries` groups a [`CurrencyPair`] with its observed rate values
/// stored in a [`BTreeMap`] keyed by [`DateTime<Utc>`].
///
/// Using a `BTreeMap` as the backing store provides three guarantees that a
/// plain `Vec` cannot:
///
/// - **Uniqueness** – each timestamp can appear at most once; inserting a rate
///   for an already-present timestamp silently overwrites the previous value.
/// - **Chronological order** – iteration always proceeds from the oldest to
///   the newest observation without a separate sort step.
/// - **Efficient range queries** – a contiguous slice of the timeline can be
///   retrieved in O(log n) time via [`BTreeMap::range`].
#[derive(Debug, Clone)]
pub struct TimeSeries {
    pair: CurrencyPair,
    rates: BTreeMap<DateTime<Utc>, Decimal>,
}

impl TimeSeries {
    /// Creates a new time series from a pre-built map of timestamp → rate.
    ///
    /// The map is stored as-is; all ordering and uniqueness guarantees come
    /// from the [`BTreeMap`] itself.
    #[must_use]
    pub const fn new(pair: CurrencyPair, rates: BTreeMap<DateTime<Utc>, Decimal>) -> Self {
        Self { pair, rates }
    }

    /// Creates a new time series from an iterator of [`ExchangeRate`] values.
    ///
    /// Each item is decomposed into its `(timestamp, rate)` pair and inserted
    /// into the backing map. If the iterator yields multiple entries with the
    /// same timestamp, only the last one is retained.
    #[must_use]
    pub fn from_exchange_rates(
        pair: CurrencyPair,
        rates: impl IntoIterator<Item = ExchangeRate>,
    ) -> Self {
        let map = rates
            .into_iter()
            .map(|r| (*r.timestamp(), *r.rate()))
            .collect();
        Self { pair, rates: map }
    }

    /// Returns the currency pair associated with this series.
    #[must_use]
    pub const fn pair(&self) -> &CurrencyPair {
        &self.pair
    }

    /// Returns all exchange-rate observations as a map of timestamp → rate,
    /// ordered chronologically from oldest to newest.
    #[must_use]
    pub const fn rates(&self) -> &BTreeMap<DateTime<Utc>, Decimal> {
        &self.rates
    }

    /// Inserts or overwrites the rate at the given timestamp.
    ///
    /// If a rate already exists for `timestamp` it is replaced silently.
    pub fn add_rate(&mut self, timestamp: DateTime<Utc>, rate: Decimal) {
        self.rates.insert(timestamp, rate);
    }

    /// Extends the series with a slice of [`ExchangeRate`] observations.
    ///
    /// Existing entries whose timestamps collide with incoming ones are
    /// overwritten.
    pub fn extend_rates(&mut self, rates: &[ExchangeRate]) {
        for r in rates {
            self.rates.insert(*r.timestamp(), *r.rate());
        }
    }

    /// Calculates a quality score for the time series.
    ///
    /// The resulting [`RateQuality`] combines four dimensions:
    ///
    /// - completeness: how close the observed number of samples is to the
    ///   expected count inferred from the typical gap between observations
    /// - gap consistency: how regular the spacing between timestamps is
    /// - outlier score: how many observations deviate strongly from the
    ///   distribution of values
    /// - volatility score: how stable the percentage returns are
    ///
    /// Each component is normalized to a `0..=100` range and then combined
    /// using the weights and thresholds from `config`.
    ///
    /// If the series has no rates, a zeroed [`RateQuality`] is returned.
    #[allow(clippy::too_many_lines)]
    #[must_use]
    pub fn calculate_rate_quality(&self, config: &RateQualityConfig) -> RateQuality {
        if self.rates.is_empty() {
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

        // Collect values in chronological order (BTreeMap guarantees this).
        let series_values: Vec<Decimal> = self.rates.values().copied().collect();

        // Collect timestamps for gap analysis.
        let timestamps: Vec<DateTime<Utc>> = self.rates.keys().copied().collect();

        let mut gaps_seconds: Vec<i64> = Vec::new();
        for window in timestamps.windows(2) {
            let gap = (window[1] - window[0]).num_seconds().abs();
            gaps_seconds.push(gap);
        }

        let completeness = if gaps_seconds.is_empty() {
            dec!(100)
        } else {
            let total_duration = match (self.rates.keys().next(), self.rates.keys().next_back()) {
                (Some(&first_ts), Some(&last_ts)) => {
                    last_ts.signed_duration_since(first_ts).num_seconds().abs()
                }
                _ => 0,
            };

            let typical_gap = median_i64(&gaps_seconds).unwrap_or(0);
            if total_duration == 0 || typical_gap <= 0 {
                dec!(100)
            } else {
                let expected_count = total_duration / typical_gap + 1;
                let expected_count_dec = Decimal::from(expected_count);
                let observed_count_dec = Decimal::from(self.rates.len());
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
                            z_score(**v, mean, std_dev).is_some_and(|z| {
                                z.abs() > config.thresholds().outlier_z_threshold()
                            })
                        })
                        .count();
                    let outlier_ratio =
                        Decimal::from(outliers) / Decimal::from(series_values.len());
                    clamp_0_100((Decimal::ONE - outlier_ratio) * dec!(100))
                }
                _ => dec!(100),
            }
        };

        // Compute percentage returns between consecutive observations.
        let mut returns: Vec<Decimal> = Vec::new();
        for window in series_values.windows(2) {
            let prev = window[0];
            let curr = window[1];
            if prev != Decimal::ZERO {
                returns.push((curr - prev) / prev);
            }
        }

        let volatility = if returns.is_empty() {
            dec!(100) // No returns to measure -> consider perfectly stable
        } else {
            let std_returns = standard_deviation(&returns).unwrap_or(Decimal::ZERO);
            clamp_0_100(
                dec!(100)
                    / (Decimal::ONE + config.thresholds().max_allowed_volatility() * std_returns),
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

    /// Returns the lowest exchange-rate value in the series.
    ///
    /// Compares only the numeric rate values; timestamps are ignored.
    ///
    /// Returns `None` when the series is empty.
    #[must_use]
    pub fn lowest_value(&self) -> Option<&Decimal> {
        self.rates.values().min()
    }

    /// Returns the highest exchange-rate value in the series.
    ///
    /// Compares only the numeric rate values; timestamps are ignored.
    ///
    /// Returns `None` when the series is empty.
    #[must_use]
    pub fn highest_value(&self) -> Option<&Decimal> {
        self.rates.values().max()
    }
}

impl std::fmt::Display for TimeSeries {
    /// Formats the time series as `TimeSeries(PAIR, [ts1: rate1, ts2: rate2, ...])`.
    ///
    /// Entries are always printed in chronological order because the backing
    /// store is a [`BTreeMap`].
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TimeSeries({}, [", self.pair)?;
        for (i, (ts, rate)) in self.rates.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{ts}: {rate}")?;
        }
        write!(f, "])")
    }
}
