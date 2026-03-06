use crate::dtos::structs::{ChangeRecommendation, PairAnalysis};
use crate::ports::exchange_rate_repository::{ExchangeRateRepository, RepositoryError};
use domain::indicators::math::range_position;
use domain::indicators::math::{highest_value, lowest_value};
use domain::indicators::quality::calculate_rate_quality::calculate_rate_quality;
use domain::indicators::quality::rate_quality_config::RateQualityConfig;
use domain::types::currency_pair::CurrencyPair;
use rust_decimal::dec;
use thiserror::Error;

/// Analyze a currency pair and produce change recommendation.
pub struct AnalyzePairUseCase<R: ExchangeRateRepository> {
    repository: R,
    config: RateQualityConfig,
}

impl<R: ExchangeRateRepository> AnalyzePairUseCase<R> {
    pub fn new(repository: R, config: RateQualityConfig) -> Self {
        Self { repository, config }
    }

    pub async fn execute(
        &self,
        pair: CurrencyPair,
        lookback_days: i64,
    ) -> Result<PairAnalysis, AnalyzeError> {
        let now = chrono::Utc::now();
        let start = now - chrono::Duration::days(lookback_days);
        let time_series = self.repository.load_rates(&pair, start..=now).await?;
        let rates = time_series.rates();

        if rates.is_empty() {
            return Err(AnalyzeError::InsufficientData);
        }

        let current_rate = *rates.last().ok_or(AnalyzeError::InsufficientData)?.rate();
        let quality = calculate_rate_quality(&time_series, &self.config);
        let min_rate = lowest_value(rates).ok_or(AnalyzeError::InsufficientData)?;
        let max_rate = highest_value(rates).ok_or(AnalyzeError::InsufficientData)?;

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

#[derive(Error, Debug)]
pub enum AnalyzeError {
    #[error("repository error: {0}")]
    Repository(#[from] RepositoryError),
    #[error("insufficient historical data")]
    InsufficientData,
}
