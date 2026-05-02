use application::responses::analyze_pair_responses::Recommendation;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub(crate) struct AnalyzePairQuery {
    base: String,
    quote: String,
    days: u32,
}

impl AnalyzePairQuery {
    #[must_use]
    pub(crate) fn base(&self) -> &str {
        &self.base
    }

    #[must_use]
    pub(crate) fn quote(&self) -> &str {
        &self.quote
    }

    #[must_use]
    pub(crate) const fn days(&self) -> u32 {
        self.days
    }
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum RecommendationDto {
    ChangeNow,
    Neutral,
    Wait,
}

#[derive(Serialize)]
pub(crate) struct PairAnalysisResponse {
    recommendation: RecommendationDto,
    reasoning: String,
}

impl PairAnalysisResponse {
    #[must_use]
    pub(crate) fn new(recommendation: Recommendation, reasoning: String) -> Self {
        Self {
            recommendation: recommendation.into(),
            reasoning,
        }
    }
}

impl From<Recommendation> for RecommendationDto {
    fn from(value: Recommendation) -> Self {
        match value {
            Recommendation::ChangeNow => Self::ChangeNow,
            Recommendation::Neutral => Self::Neutral,
            Recommendation::Wait => Self::Wait,
        }
    }
}
