use domain::types::currency_pair::CurrencyPair;
use rust_decimal::Decimal;

/// Recommendation for whether user should change money now or wait.
#[derive(Debug)]
pub struct ChangeRecommendation {
    pair: CurrencyPair,
    should_change_now: bool,
    confidence: Decimal, // 0.0 to 1.0
    reasoning: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// Analysis result for a single pair.
#[derive(Debug)]
pub struct PairAnalysis {
    pair: CurrencyPair,
    rate_count: usize,
    quality_score: Decimal,
    recommendation: ChangeRecommendation,
}

impl ChangeRecommendation {
    pub fn new(
        pair: CurrencyPair,
        should_change_now: bool,
        confidence: Decimal,
        reasoning: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        Self {
            pair,
            should_change_now,
            confidence,
            reasoning,
            timestamp,
        }
    }
}

impl PairAnalysis {
    pub fn new(
        pair: CurrencyPair,
        rate_count: usize,
        quality_score: Decimal,
        recommendation: ChangeRecommendation,
    ) -> Self {
        Self {
            pair,
            rate_count,
            quality_score,
            recommendation,
        }
    }
}
