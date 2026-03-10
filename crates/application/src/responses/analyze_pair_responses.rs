//! Response models produced by the pair-analysis use case.
//!
//! These types provide a stable, application-level representation of the
//! analysis outcome without exposing internal implementation details from
//! the domain or infrastructure layers.
use domain::types::currency_pair::CurrencyPair;
use rust_decimal::Decimal;

/// Recommendation describing whether a user should exchange funds now.
///
/// The recommendation combines a boolean decision with a confidence score,
/// a human-readable explanation, and the timestamp at which the decision
/// was produced.
#[derive(Debug)]
pub struct ChangeRecommendation {
    pair: CurrencyPair,
    should_change_now: bool,
    confidence: Decimal, // 0.0 to 1.0
    reasoning: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

/// Aggregate analysis result for a single currency pair.
///
/// This structure contains the analyzed pair, the number of historical rates
/// considered, the resulting quality score, and the final recommendation.
#[derive(Debug)]
pub struct PairAnalysis {
    pair: CurrencyPair,
    rate_count: usize,
    quality_score: Decimal,
    recommendation: ChangeRecommendation,
}

impl ChangeRecommendation {
    /// Creates a new recommendation for a currency pair.
    ///
    /// `confidence` is expected to be normalized between `0` and `1`, where
    /// higher values indicate stronger confidence in the recommendation.
    #[must_use]
    pub const fn new(
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

    /// Returns the currency pair this recommendation applies to.
    #[must_use]
    pub const fn pair(&self) -> &CurrencyPair {
        &self.pair
    }

    /// Returns whether the analysis recommends exchanging funds now.
    #[must_use]
    pub const fn should_change_now(&self) -> bool {
        self.should_change_now
    }

    /// Returns the confidence score associated with the recommendation.
    #[must_use]
    pub const fn confidence(&self) -> &Decimal {
        &self.confidence
    }

    /// Returns the human-readable reasoning behind the recommendation.
    #[must_use]
    pub fn reasoning(&self) -> &str {
        &self.reasoning
    }

    /// Returns the timestamp at which the recommendation was generated.
    #[must_use]
    pub const fn timestamp(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.timestamp
    }
}

impl PairAnalysis {
    /// Creates a new analysis result for a currency pair.
    #[must_use]
    pub const fn new(
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

    /// Returns the analyzed currency pair.
    #[must_use]
    pub const fn pair(&self) -> &CurrencyPair {
        &self.pair
    }

    /// Returns the number of rates used to produce the analysis.
    #[must_use]
    pub const fn rate_count(&self) -> usize {
        self.rate_count
    }

    /// Returns the overall quality score for the analyzed data set.
    #[must_use]
    pub const fn quality_score(&self) -> &Decimal {
        &self.quality_score
    }

    /// Returns the recommendation derived from the analysis.
    #[must_use]
    pub const fn recommendation(&self) -> &ChangeRecommendation {
        &self.recommendation
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use domain::types::currency::Currency;
    use domain::types::currency_pair::CurrencyPair;
    use rust_decimal::dec;

    use super::*;

    fn make_pair() -> CurrencyPair {
        let base = Currency::new("EUR").unwrap();
        let quote = Currency::new("USD").unwrap();
        CurrencyPair::new(base, quote).unwrap()
    }

    // ── ChangeRecommendation ────────────────────────────────────────

    #[test]
    fn change_recommendation_stores_pair() {
        let pair = make_pair();
        let now = Utc::now();
        let rec = ChangeRecommendation::new(pair.clone(), true, dec!(0.9), "good".into(), now);

        assert_eq!(rec.pair(), &pair);
    }

    #[test]
    fn change_recommendation_should_change_now_true() {
        let now = Utc::now();
        let rec = ChangeRecommendation::new(make_pair(), true, dec!(0.8), "reason".into(), now);

        assert!(rec.should_change_now());
    }

    #[test]
    fn change_recommendation_should_change_now_false() {
        let now = Utc::now();
        let rec = ChangeRecommendation::new(make_pair(), false, dec!(0.3), "wait".into(), now);

        assert!(!rec.should_change_now());
    }

    #[test]
    fn change_recommendation_confidence() {
        let now = Utc::now();
        let rec = ChangeRecommendation::new(make_pair(), true, dec!(0.75), "msg".into(), now);

        assert_eq!(rec.confidence(), &dec!(0.75));
    }

    #[test]
    fn change_recommendation_reasoning() {
        let now = Utc::now();
        let rec =
            ChangeRecommendation::new(make_pair(), true, dec!(0.5), "favorable moment".into(), now);

        assert_eq!(rec.reasoning(), "favorable moment");
    }

    #[test]
    fn change_recommendation_timestamp() {
        let now = Utc::now();
        let rec = ChangeRecommendation::new(make_pair(), false, dec!(0.1), "t".into(), now);

        assert_eq!(rec.timestamp(), &now);
    }

    #[test]
    fn change_recommendation_zero_confidence() {
        let now = Utc::now();
        let rec = ChangeRecommendation::new(make_pair(), false, dec!(0), "no signal".into(), now);

        assert_eq!(rec.confidence(), &dec!(0));
    }

    #[test]
    fn change_recommendation_max_confidence() {
        let now = Utc::now();
        let rec = ChangeRecommendation::new(make_pair(), true, dec!(1.0), "max signal".into(), now);

        assert_eq!(rec.confidence(), &dec!(1.0));
    }

    #[test]
    fn change_recommendation_empty_reasoning() {
        let now = Utc::now();
        let rec = ChangeRecommendation::new(make_pair(), true, dec!(0.5), String::new(), now);

        assert_eq!(rec.reasoning(), "");
    }

    // ── PairAnalysis ────────────────────────────────────────────────

    #[test]
    fn pair_analysis_stores_pair() {
        let pair = make_pair();
        let now = Utc::now();
        let rec = ChangeRecommendation::new(pair.clone(), true, dec!(0.9), "r".into(), now);
        let analysis = PairAnalysis::new(pair.clone(), 100, dec!(85), rec);

        assert_eq!(analysis.pair(), &pair);
    }

    #[test]
    fn pair_analysis_rate_count() {
        let now = Utc::now();
        let rec = ChangeRecommendation::new(make_pair(), true, dec!(0.5), "r".into(), now);
        let analysis = PairAnalysis::new(make_pair(), 42, dec!(90), rec);

        assert_eq!(analysis.rate_count(), 42);
    }

    #[test]
    fn pair_analysis_rate_count_zero() {
        let now = Utc::now();
        let rec = ChangeRecommendation::new(make_pair(), false, dec!(0), "r".into(), now);
        let analysis = PairAnalysis::new(make_pair(), 0, dec!(0), rec);

        assert_eq!(analysis.rate_count(), 0);
    }

    #[test]
    fn pair_analysis_quality_score() {
        let now = Utc::now();
        let rec = ChangeRecommendation::new(make_pair(), true, dec!(0.7), "r".into(), now);
        let analysis = PairAnalysis::new(make_pair(), 50, dec!(95.5), rec);

        assert_eq!(analysis.quality_score(), &dec!(95.5));
    }

    #[test]
    fn pair_analysis_recommendation_delegates() {
        let now = Utc::now();
        let rec = ChangeRecommendation::new(make_pair(), true, dec!(0.88), "go now".into(), now);
        let analysis = PairAnalysis::new(make_pair(), 10, dec!(80), rec);

        assert!(analysis.recommendation().should_change_now());
        assert_eq!(analysis.recommendation().confidence(), &dec!(0.88));
        assert_eq!(analysis.recommendation().reasoning(), "go now");
        assert_eq!(analysis.recommendation().timestamp(), &now);
    }

    #[test]
    fn pair_analysis_recommendation_wait() {
        let now = Utc::now();
        let rec = ChangeRecommendation::new(make_pair(), false, dec!(0.2), "wait".into(), now);
        let analysis = PairAnalysis::new(make_pair(), 5, dec!(60), rec);

        assert!(!analysis.recommendation().should_change_now());
        assert_eq!(analysis.recommendation().confidence(), &dec!(0.2));
    }

    #[test]
    fn pair_analysis_debug_impl() {
        let now = Utc::now();
        let rec = ChangeRecommendation::new(make_pair(), true, dec!(0.5), "r".into(), now);
        let analysis = PairAnalysis::new(make_pair(), 1, dec!(50), rec);

        // Verify Debug is implemented and doesn't panic
        let debug_str = format!("{analysis:?}");
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn change_recommendation_debug_impl() {
        let now = Utc::now();
        let rec = ChangeRecommendation::new(make_pair(), true, dec!(0.5), "reason".into(), now);

        let debug_str = format!("{rec:?}");
        assert!(debug_str.contains("ChangeRecommendation"));
    }
}
