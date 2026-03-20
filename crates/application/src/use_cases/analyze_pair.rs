use domain::indicators::math::range_position;
use domain::types::currency_pair::CurrencyPair;
use domain::types::rate_quality_config::RateQualityConfig;
use rust_decimal::dec;
use thiserror::Error;

use crate::ports::exchange_rate_repository::{ExchangeRateRepository, RepositoryError};
use crate::responses::analyze_pair_responses::{ChangeRecommendation, PairAnalysis};

/// Application use case for analyzing a currency pair over a historical window.
///
/// This workflow loads previously stored exchange-rate data for a pair,
/// computes a quality score for the available time series, determines where the
/// latest rate sits within the observed range, and produces a human-readable
/// recommendation about whether exchanging now appears favorable.
#[derive(Debug)]
pub struct AnalyzePairUseCase<R: ExchangeRateRepository> {
    repository: R,
    config: RateQualityConfig,
}

impl<R: ExchangeRateRepository> AnalyzePairUseCase<R> {
    /// Creates a new analyze-pair use case.
    ///
    /// The provided repository is used to load historical rates, while
    /// `config` controls how time-series quality is evaluated during analysis.
    #[must_use]
    pub const fn new(repository: R, config: RateQualityConfig) -> Self {
        Self { repository, config }
    }

    /// Analyzes the given currency pair using the specified historical lookback.
    ///
    /// The method:
    /// - loads stored rates for the pair over the last `lookback_days`,
    /// - computes the time-series quality score,
    /// - finds the latest rate's position inside the observed min/max range,
    /// - derives a recommendation and confidence score.
    ///
    /// # Errors
    ///
    /// Returns [`AnalyzeError::Repository`] if loading rates fails, or
    /// [`AnalyzeError::InsufficientData`] if the available series cannot support
    /// analysis.
    pub async fn execute(
        &self,
        pair: CurrencyPair,
        lookback_days: u32,
    ) -> Result<PairAnalysis, AnalyzeError> {
        let now = chrono::Utc::now();
        let start = now - chrono::Duration::days(i64::from(lookback_days));
        let time_series = self.repository.load_rates(&pair, &(start..=now)).await?;
        let rates = time_series.rates();

        if rates.is_empty() {
            return Err(AnalyzeError::InsufficientData);
        }

        let current_rate = *rates.last().ok_or(AnalyzeError::InsufficientData)?.rate();
        let quality = time_series.calculate_rate_quality(&self.config);
        let min_rate = time_series
            .lowest_value()
            .ok_or(AnalyzeError::InsufficientData)?;
        let max_rate = time_series
            .highest_value()
            .ok_or(AnalyzeError::InsufficientData)?;

        let position = range_position(current_rate, *max_rate, *min_rate)
            .ok_or(AnalyzeError::InsufficientData)?;

        // Decision logic
        let should_change_now = position >= dec!(0.85);

        // Confidence combines signal strength + data quality
        let signal_strength = (position - dec!(0.5)).abs() * dec!(2);
        let confidence = signal_strength * *quality.overall();

        // Human readable explanation
        let reasoning = if should_change_now {
            format!(
                "Current rate {} is near the top of the {}-day range (position {:.2}). \
                     Data quality {:.2}. Favorable moment to exchange.",
                current_rate,
                lookback_days,
                position,
                quality.overall()
            )
        } else {
            format!(
                "Current rate {} sits in the middle/lower part of the {}-day range \
                     (position {:.2}). Data quality {:.2}. Waiting may yield a better rate.",
                current_rate,
                lookback_days,
                position,
                quality.overall()
            )
        };

        let recommendation =
            ChangeRecommendation::new(pair.clone(), should_change_now, confidence, reasoning, now);

        Ok(PairAnalysis::new(
            pair,
            rates.len(),
            *quality.overall(),
            recommendation,
        ))
    }
}

/// Errors that can occur while analyzing a currency pair.
#[derive(Error, Debug)]
pub enum AnalyzeError {
    /// Wraps a repository failure that occurred while loading historical data.
    #[error("repository error: {0}")]
    Repository(#[from] RepositoryError),

    /// Indicates that the available historical data is missing or too limited to
    /// produce a meaningful analysis.
    #[error("insufficient historical data")]
    InsufficientData,
}
